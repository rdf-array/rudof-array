use serde::{Deserialize, Serialize};

use super::{
    dependency_graph::{DependencyGraph, PosNeg},
    node_constraint::NodeConstraint,
    shape::Shape,
};
use crate::{Expr, Pred, ShapeLabelIdx, ir::schema_ir::SchemaIR};
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    vec,
};

#[derive(Debug, PartialEq, Clone, Default, Serialize, Deserialize)]
pub enum ShapeExpr {
    ShapeOr {
        exprs: Vec<ShapeLabelIdx>,
    },
    ShapeAnd {
        exprs: Vec<ShapeLabelIdx>,
    },
    ShapeNot {
        expr: ShapeLabelIdx,
    },
    NodeConstraint(Box<NodeConstraint>),
    Shape(Box<Shape>),
    External {},
    Ref {
        idx: ShapeLabelIdx,
    },
    /// rdf-array: a sequence whose members, in order, match the regular
    /// expression `expr`.
    ListShape {
        expr: Box<ListExpr>,
    },

    #[default]
    Empty,
}

/// rdf-array: the regular expression of a [`ShapeExpr::ListShape`] over the
/// shape expressions of its members, each with an index of its own.
/// `max` of `None` is unbounded; `EachOf` is ordered.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum ListExpr {
    Member {
        expr: ShapeLabelIdx,
        min: usize,
        max: Option<usize>,
    },
    EachOf {
        exprs: Vec<ListExpr>,
        min: usize,
        max: Option<usize>,
    },
    OneOf {
        exprs: Vec<ListExpr>,
        min: usize,
        max: Option<usize>,
    },
}

impl ListExpr {
    pub fn min_max(&self) -> (usize, Option<usize>) {
        match self {
            ListExpr::Member { min, max, .. }
            | ListExpr::EachOf { min, max, .. }
            | ListExpr::OneOf { min, max, .. } => (*min, *max),
        }
    }

    /// The indexes of the members' shape expressions, in order.
    pub fn members(&self) -> Vec<ShapeLabelIdx> {
        match self {
            ListExpr::Member { expr, .. } => vec![*expr],
            ListExpr::EachOf { exprs, .. } | ListExpr::OneOf { exprs, .. } => {
                exprs.iter().flat_map(ListExpr::members).collect()
            },
        }
    }

    /// The fewest and the most members a sequence matching this expression can
    /// have (`None`: unbounded).
    pub fn length_bounds(&self) -> (usize, Option<usize>) {
        let (one_min, one_max) = match self {
            ListExpr::Member { .. } => (1, Some(1)),
            ListExpr::EachOf { exprs, .. } => exprs.iter().map(ListExpr::length_bounds).fold(
                (0usize, Some(0usize)),
                |(acc_min, acc_max), (min, max)| {
                    (
                        acc_min.saturating_add(min),
                        acc_max.zip(max).map(|(a, b)| a.saturating_add(b)),
                    )
                },
            ),
            ListExpr::OneOf { exprs, .. } => exprs
                .iter()
                .map(ListExpr::length_bounds)
                .reduce(|(acc_min, acc_max), (min, max)| (acc_min.min(min), acc_max.zip(max).map(|(a, b)| a.max(b))))
                .unwrap_or((0, Some(0))),
        };
        let (min, max) = self.min_max();
        // Repeating an expression that takes no members takes no members.
        let total_max = match (max, one_max) {
            (_, Some(0)) | (Some(0), _) => Some(0),
            (Some(max), Some(one_max)) => Some(max.saturating_mul(one_max)),
            _ => None,
        };
        (min.saturating_mul(one_min), total_max)
    }

    /// Spells the expression in ShExC, showing a member's shape with `show`.
    pub fn show(&self, show: &dyn Fn(&ShapeLabelIdx) -> String) -> String {
        let junction = |exprs: &[ListExpr], sep: &str| {
            let body = exprs.iter().map(|e| e.show(show)).collect::<Vec<_>>().join(sep);
            format!("( {body} )")
        };
        let (body, (min, max)) = match self {
            ListExpr::Member { expr, .. } => (show(expr), self.min_max()),
            ListExpr::EachOf { exprs, .. } => (junction(exprs, " ; "), self.min_max()),
            ListExpr::OneOf { exprs, .. } => (junction(exprs, " | "), self.min_max()),
        };
        match (min, max) {
            (1, Some(1)) => body,
            (0, Some(1)) => format!("{body} ?"),
            (0, None) => format!("{body} *"),
            (1, None) => format!("{body} +"),
            (min, Some(max)) if min == max => format!("{body} {{{min}}}"),
            (min, Some(max)) => format!("{body} {{{min},{max}}}"),
            (min, None) => format!("{body} {{{min},*}}"),
        }
    }
}

impl ShapeExpr {
    pub fn mk_ref(idx: ShapeLabelIdx) -> ShapeExpr {
        ShapeExpr::Ref { idx }
    }

    pub fn direct_references(&self) -> Vec<ShapeLabelIdx> {
        match self {
            ShapeExpr::ShapeOr { exprs, .. } => exprs.to_vec(),
            ShapeExpr::ShapeAnd { exprs, .. } => exprs.to_vec(),
            ShapeExpr::ShapeNot { expr, .. } => vec![*expr],
            ShapeExpr::NodeConstraint(_) => vec![],
            ShapeExpr::Shape(_s) => vec![],
            ShapeExpr::External {} => vec![],
            ShapeExpr::Ref { idx } => vec![*idx],
            // A member is another node, like a triple constraint's value, so a
            // list refers to its members' shapes no more directly than a shape
            // does: `<Tree> LIST ( @<Tree> * )` is not a circular reference.
            ShapeExpr::ListShape { .. } => vec![],
            ShapeExpr::Empty => vec![],
        }
    }

    pub fn get_triple_exprs(&self, schema: &SchemaIR) -> Vec<Expr> {
        match self {
            ShapeExpr::ShapeOr { .. } => Vec::new(), // Should be an error? extending from OR
            ShapeExpr::ShapeAnd { exprs, .. } => exprs
                .iter()
                .flat_map(|e| {
                    let info = schema.find_shape_idx(e).unwrap();
                    info.expr().get_triple_exprs(schema)
                })
                .collect(),
            ShapeExpr::ShapeNot { .. } => Vec::new(), /*schema
            .find_shape_idx(expr)
            .map(|info| info.expr().get_triple_exprs(schema))
            .unwrap_or_default() */
            ShapeExpr::NodeConstraint(_nc) => vec![],
            ShapeExpr::Shape(s) => vec![s.triple_expr().clone()],
            ShapeExpr::External {} => vec![],
            ShapeExpr::Ref { idx } => {
                let info = schema.find_shape_idx(idx).unwrap();
                info.expr().get_triple_exprs(schema)
            },
            ShapeExpr::ListShape { .. } => vec![],
            ShapeExpr::Empty => vec![],
        }
    }

    /// Get the references in this shape expression.
    pub fn references(&self, schema: &SchemaIR) -> HashMap<Pred, Vec<ShapeLabelIdx>> {
        let mut visited = HashSet::new();
        self.references_visited(schema, &mut visited)
    }

    pub(crate) fn references_visited(
        &self,
        schema: &SchemaIR,
        visited: &mut HashSet<ShapeLabelIdx>,
    ) -> HashMap<Pred, Vec<ShapeLabelIdx>> {
        match self {
            ShapeExpr::ShapeOr { exprs, .. } => exprs.iter().fold(HashMap::new(), |mut acc, expr| {
                let refs = if visited.insert(*expr) {
                    schema
                        .find_shape_idx(expr)
                        .map(|info| info.expr().references_visited(schema, visited))
                        .unwrap_or_default()
                } else {
                    HashMap::new()
                };
                for (p, v) in refs {
                    acc.entry(p).or_default().extend(v);
                }
                acc
            }),
            ShapeExpr::ShapeAnd { exprs, .. } => exprs.iter().fold(HashMap::new(), |mut acc, expr| {
                let refs = if visited.insert(*expr) {
                    schema
                        .find_shape_idx(expr)
                        .map(|info| info.expr().references_visited(schema, visited))
                        .unwrap_or_default()
                } else {
                    HashMap::new()
                };
                for (p, v) in refs {
                    acc.entry(p).or_default().extend(v);
                }
                acc
            }),
            ShapeExpr::ShapeNot { expr, .. } => {
                if visited.insert(*expr) {
                    schema
                        .find_shape_idx(expr)
                        .map(|info| info.expr().references_visited(schema, visited))
                        .unwrap_or_default()
                } else {
                    HashMap::new()
                }
            },
            ShapeExpr::NodeConstraint(_nc) => HashMap::new(),
            ShapeExpr::Shape(s) => s.references_visited(schema, visited),
            ShapeExpr::External {} => HashMap::new(),
            ShapeExpr::Ref { idx } => {
                let mut map = HashMap::new();
                map.insert(Pred::default(), vec![*idx]);
                // Also collect the references of the referenced shape expression, so that
                // value-reference dependencies behind `<B> @<A>` indirections (eg the
                // branches of an extended ShapeOr) are scheduled for typing.
                if visited.insert(*idx)
                    && let Some(info) = schema.find_shape_idx(idx)
                {
                    for (p, v) in info.expr().references_visited(schema, visited) {
                        map.entry(p).or_default().extend(v);
                    }
                }
                map
            },
            // The members of a sequence are not neighbours of the node: they are
            // proved when the list is checked.
            ShapeExpr::ListShape { .. } => HashMap::new(),
            ShapeExpr::Empty => HashMap::new(),
        }
    }

    /// Get the predicates used in this shape expression.
    /// For ShapeOr, ShapeAnd and ShapeNot, it will recursively get the predicates from
    /// the referenced shapes.
    /// For Shape, it will get the predicates from the shape.
    /// For NodeConstraint and External, it will return an empty set.
    /// For Ref, it will get the predicates from the referenced shape.
    /// For Empty, it will return an empty set.
    /// This will used the direct predicates of the shape, not the ones from extends.
    /// To get the predicates including extends, use the method preds_extends of `SchemaIR`
    pub fn preds(&self, schema: &SchemaIR) -> HashSet<Pred> {
        let mut visited = HashSet::new();
        self.preds_visited(schema, &mut visited)
    }

    /// `visited` holds the shapes already visited, so that a shape that refers
    /// to itself (e.g. `:A @:A AND { :p . }`) is visited only once
    fn preds_visited(&self, schema: &SchemaIR, visited: &mut HashSet<ShapeLabelIdx>) -> HashSet<Pred> {
        let mut preds = HashSet::new();
        let mut visit = |idx: &ShapeLabelIdx, preds: &mut HashSet<Pred>| {
            if visited.insert(*idx) {
                let info = schema.find_shape_idx(idx).unwrap();
                preds.extend(info.expr().preds_visited(schema, visited));
            }
        };
        match self {
            ShapeExpr::ShapeOr { exprs, .. } | ShapeExpr::ShapeAnd { exprs, .. } => {
                for e in exprs {
                    visit(e, &mut preds);
                }
            },
            ShapeExpr::ShapeNot { expr, .. } => visit(expr, &mut preds),
            ShapeExpr::NodeConstraint(_nc) => {},
            ShapeExpr::Shape(s) => {
                preds.extend(s.preds());
            },
            ShapeExpr::External {} => {},
            ShapeExpr::Ref { idx } => visit(idx, &mut preds),
            ShapeExpr::ListShape { .. } => {},
            ShapeExpr::Empty => {},
        }
        preds
    }

    /// Adds PosNeg edges to the dependency graph.
    pub(crate) fn add_edges<'a>(
        &'a self,
        source: ShapeLabelIdx,
        graph: &mut DependencyGraph,
        pos_neg: PosNeg,
        schema: &'a SchemaIR,
        visited: &mut Vec<&'a ShapeExpr>,
    ) {
        match self {
            ShapeExpr::ShapeOr { exprs, .. } => {
                for expr in exprs {
                    if let Some(info) = schema.find_shape_idx(expr) {
                        let expr = info.expr();
                        if visited.contains(&expr) {
                            continue;
                        } else {
                            visited.push(expr);
                            expr.add_edges(source, graph, pos_neg, schema, visited);
                        }
                    }
                }
            },
            ShapeExpr::ShapeAnd { exprs, .. } => {
                for expr in exprs {
                    if let Some(info) = schema.find_shape_idx(expr) {
                        let expr = info.expr();
                        if visited.contains(&expr) {
                            continue;
                        } else {
                            visited.push(expr);
                            expr.add_edges(source, graph, pos_neg, schema, visited);
                        }
                    }
                }
            },
            ShapeExpr::ShapeNot { expr, .. } => {
                if let Some(info) = schema.find_shape_idx(expr) {
                    let expr = info.expr();
                    if visited.contains(&expr) {
                    } else {
                        visited.push(expr);
                        expr.add_edges(source, graph, pos_neg.change(), schema, visited);
                    }
                }
            },
            ShapeExpr::NodeConstraint(_) => {},
            ShapeExpr::Shape(s) => s.add_edges(source, graph, pos_neg),
            ShapeExpr::External {} => {},
            ShapeExpr::Ref { idx } => {
                graph.add_edge(source, *idx, pos_neg);
            },
            // Unlike a circular reference, stratification does look through a
            // list: `<S> LIST ( NOT @<S> * )` negates <S> in <S>.
            ShapeExpr::ListShape { expr } => {
                for member in expr.members() {
                    graph.add_edge(source, member, pos_neg);
                }
            },
            ShapeExpr::Empty => {},
        }
    }

    // TODO: Improve the visualization of shape expressions
    /*pub fn show_qualified(&self, prefixmap: &prefixmap::PrefixMap) -> String {
        match self {
            ShapeExpr::ShapeOr { exprs, .. } => format!(
                "({})",
                exprs
                    .iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<_>>()
                    .join(" OR ")
            ),
            ShapeExpr::ShapeAnd { exprs, .. } => format!(
                "({})",
                exprs
                    .iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<_>>()
                    .join(" AND ")
            ),
            ShapeExpr::ShapeNot { expr, .. } => format!("(NOT {})", expr.to_string()),
            ShapeExpr::NodeConstraint(nc) => nc.to_string(),
            ShapeExpr::Shape(shape) => shape.show_qualified(prefixmap),
            ShapeExpr::External {} => "External".to_string(),
            ShapeExpr::Ref { idx } => format!("@{}", idx),
            ShapeExpr::Empty => "{}".to_string(),
        }
    }*/
}

impl Display for ShapeExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShapeExpr::ShapeOr { exprs, .. } => write!(
                f,
                "OR({})",
                exprs.iter().map(|e| format!("{e}")).collect::<Vec<_>>().join(", ")
            ),
            ShapeExpr::ShapeAnd { exprs, .. } => write!(
                f,
                "AND({})",
                exprs.iter().map(|e| format!("{e}")).collect::<Vec<_>>().join(", ")
            ),
            ShapeExpr::ShapeNot { expr, .. } => write!(f, "NOT {expr}"),
            ShapeExpr::NodeConstraint(nc) => write!(f, "{nc}"),
            ShapeExpr::Shape(shape) => write!(f, "{shape}"),
            ShapeExpr::External {} => write!(f, "External"),
            ShapeExpr::Ref { idx } => write!(f, "@{idx}"),
            ShapeExpr::ListShape { expr } => write!(f, "LIST ( {} )", expr.show(&|e| format!("{e}"))),
            ShapeExpr::Empty => write!(f, "<Empty>"),
        }
    }
}
