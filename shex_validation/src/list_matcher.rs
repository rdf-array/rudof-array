//! rdf-array: matching a sequence against a LIST shape expression.
//!
//! `rbe` works on bags, and a list's members are ordered, so this is a matcher
//! of its own: it runs the regular expression over the sequence as a set of
//! positions, the positions at which each part of the expression can stop. That
//! is backtracking without its cost, since a member that may belong to more
//! than one part keeps every reading alive at once, and `(position, shape)` is
//! tested at most once. Each position keeps one way of reaching it, which gives
//! the results of a match, one per member in order.

use crate::typing::ValidationResult;
use crate::validator_error::ValidatorError;
use crate::{Reason, Reasons, ValidatorErrors};
use either::Either;
use shex_ast::ir::shape_expr::{ListExpr, ShapeExpr};
use shex_ast::{Node, ShapeLabelIdx};
use std::collections::{BTreeMap, HashMap};

type Result<T> = std::result::Result<T, ValidatorError>;

/// The positions reached, each with the last step of one way of reaching it
/// (`None`: no member taken yet).
type Positions = BTreeMap<usize, Option<usize>>;

/// One member taken: its position, the shape it passed, the step before it.
struct Step {
    pos: usize,
    idx: ShapeLabelIdx,
    previous: Option<usize>,
}

struct Matcher<'a, F> {
    members: &'a [Node],
    check_member: F,
    /// The result of every `(position, shape)` tried.
    tested: HashMap<(usize, ShapeLabelIdx), ValidationResult>,
    /// The order the pairs were tried in, for the report of a failure.
    tried: Vec<(usize, ShapeLabelIdx)>,
    steps: Vec<Step>,
}

impl<F> Matcher<'_, F>
where
    F: FnMut(&Node, &ShapeLabelIdx) -> Result<ValidationResult>,
{
    fn passes(&mut self, pos: usize, idx: ShapeLabelIdx) -> Result<bool> {
        if let Some(result) = self.tested.get(&(pos, idx)) {
            return Ok(result.is_right());
        }
        let result = (self.check_member)(&self.members[pos], &idx)?;
        let passed = result.is_right();
        self.tested.insert((pos, idx), result);
        self.tried.push((pos, idx));
        Ok(passed)
    }

    /// The positions at which `expr`, with its cardinality, can stop when it
    /// starts at any of `starts`.
    fn ends(&mut self, expr: &ListExpr, starts: &Positions) -> Result<Positions> {
        let (min, max) = expr.min_max();
        let mut result = if min == 0 { starts.clone() } else { Positions::new() };
        let mut current = starts.clone();
        let mut count = 0;
        while max != Some(count) {
            let next = self.ends_once(expr, &current)?;
            count += 1;
            if next.is_empty() {
                break;
            }
            // The same positions again give the same positions forever. Taking
            // members moves the set up until it is empty, and a nullable part
            // can only make it grow, so one of the two always comes, however
            // large the cardinality.
            let stable = next.keys().eq(current.keys());
            if count >= min || stable {
                for (pos, step) in &next {
                    result.entry(*pos).or_insert(*step);
                }
            }
            if stable {
                break;
            }
            current = next;
        }
        Ok(result)
    }

    /// As [`Self::ends`], for one pass through `expr`.
    fn ends_once(&mut self, expr: &ListExpr, starts: &Positions) -> Result<Positions> {
        match expr {
            ListExpr::Member { expr: idx, .. } => {
                let mut result = Positions::new();
                for (pos, previous) in starts {
                    if *pos < self.members.len() && self.passes(*pos, *idx)? {
                        let step = self.steps.len();
                        self.steps.push(Step {
                            pos: *pos,
                            idx: *idx,
                            previous: *previous,
                        });
                        result.entry(pos + 1).or_insert(Some(step));
                    }
                }
                Ok(result)
            },
            ListExpr::EachOf { exprs, .. } => {
                let mut current = starts.clone();
                for e in exprs {
                    current = self.ends(e, &current)?;
                    if current.is_empty() {
                        break;
                    }
                }
                Ok(current)
            },
            ListExpr::OneOf { exprs, .. } => {
                let mut result = Positions::new();
                for e in exprs {
                    for (pos, step) in self.ends(e, starts)? {
                        result.entry(pos).or_insert(step);
                    }
                }
                Ok(result)
            },
        }
    }

    /// The members' reasons along the way that ends with step `last`, in order.
    fn reasons(&self, mut last: Option<usize>) -> Vec<Reason> {
        let mut path = Vec::new();
        while let Some(step) = last {
            let step = &self.steps[step];
            path.push((step.pos, step.idx));
            last = step.previous;
        }
        path.into_iter()
            .rev()
            .filter_map(|key| self.tested.get(&key))
            .filter_map(|result| result.as_ref().right().cloned())
            .flatten()
            .collect()
    }

    /// The first member that passed none of the shapes tried on it, and the
    /// errors of those shapes.
    fn first_failure(&self) -> Option<(usize, Vec<ValidatorError>)> {
        let mut failing: BTreeMap<usize, Vec<ValidatorError>> = BTreeMap::new();
        let mut passed = vec![false; self.members.len()];
        for key in &self.tried {
            match self.tested.get(key) {
                Some(Either::Right(_)) => passed[key.0] = true,
                Some(Either::Left(errors)) => failing.entry(key.0).or_default().extend(errors.iter().cloned()),
                None => {},
            }
        }
        failing.into_iter().find(|(pos, _)| !passed[*pos])
    }
}

/// Checks `node` against the LIST shape expression `se`, whose regular
/// expression is `expr`: `node` must be a sequence whose members, in order,
/// match `expr`. `check_member` tests one member, a focus node of its own,
/// against one member shape.
pub(crate) fn check_list_shape(
    node: &Node,
    se: &ShapeExpr,
    expr: &ListExpr,
    check_member: impl FnMut(&Node, &ShapeLabelIdx) -> Result<ValidationResult>,
) -> Result<ValidationResult> {
    let failure = |error: ValidatorError| {
        Ok(Either::Left(vec![ValidatorError::ListShapeFailure {
            shape_expr: Box::new(se.clone()),
            node: Box::new(node.clone()),
            errors: ValidatorErrors::new(vec![error]),
        }]))
    };
    let Some(elements) = node.as_object().sequence_elements() else {
        return failure(ValidatorError::SequenceExpected {
            shape_expr: Box::new(se.clone()),
            node: Box::new(node.clone()),
        });
    };
    let members: Vec<Node> = elements.iter().cloned().map(Node::from).collect();
    let mut matcher = Matcher {
        members: &members,
        check_member,
        tested: HashMap::new(),
        tried: Vec::new(),
        steps: Vec::new(),
    };
    let ends = matcher.ends(expr, &Positions::from([(0, None)]))?;
    if let Some(last) = ends.get(&members.len()) {
        return Ok(Either::Right(vec![Reason::ListShapeResults {
            node: node.clone(),
            shape_expr: se.clone(),
            reasons: Reasons::new(matcher.reasons(*last)),
        }]));
    }
    match matcher.first_failure() {
        Some((pos, errors)) => failure(ValidatorError::ListMemberFailure {
            node: Box::new(node.clone()),
            index: pos + 1,
            member: Box::new(members[pos].clone()),
            errors: ValidatorErrors::new(errors),
        }),
        None => {
            let (min, max) = expr.length_bounds();
            failure(ValidatorError::ListLengthViolation {
                shape_expr: Box::new(se.clone()),
                node: Box::new(node.clone()),
                length: members.len(),
                min,
                max,
            })
        },
    }
}
