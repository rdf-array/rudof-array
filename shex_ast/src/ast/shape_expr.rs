use prefixmap::error::DerefError;
use prefixmap::{DerefIri, IriRef, PrefixMap};
use rudof_iri::IriS;
use serde::{Deserialize, Serialize, Serializer};
use std::str::FromStr;

use super::serde_string_or_struct::SerializeStringOrStruct;
use crate::Annotation;
use crate::ast::serde_string_or_struct::*;
use crate::{NodeConstraint, RefError, Shape, ShapeExprLabel};

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
#[serde(transparent)]
pub struct ShapeExprWrapper {
    #[serde(
        serialize_with = "serialize_string_or_struct",
        deserialize_with = "deserialize_string_or_struct"
    )]
    pub se: ShapeExpr,
}

impl DerefIri for ShapeExprWrapper {
    fn deref_iri(self, base: Option<&IriS>, prefixmap: Option<&PrefixMap>) -> Result<Self, DerefError> {
        let se = self.se.deref_iri(base, prefixmap)?;
        let sew = ShapeExprWrapper { se };
        Ok(sew)
    }
}

impl From<ShapeExpr> for ShapeExprWrapper {
    fn from(shape_expr: ShapeExpr) -> Self {
        Self { se: shape_expr }
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
#[serde(tag = "type")]
pub enum ShapeExpr {
    ShapeOr {
        #[serde(rename = "shapeExprs")]
        shape_exprs: Vec<ShapeExprWrapper>,
    },
    ShapeAnd {
        #[serde(rename = "shapeExprs")]
        shape_exprs: Vec<ShapeExprWrapper>,
    },
    ShapeNot {
        #[serde(rename = "shapeExpr")]
        shape_expr: Box<ShapeExprWrapper>,
    },

    NodeConstraint(NodeConstraint),

    Shape(Shape),

    #[serde(rename = "ShapeExternal")]
    External,

    /// rdf-array: the LIST shape expression, `LIST ( regex )` or
    /// `LIST card x`, which matches a sequence whose members, in order, match
    /// the regular expression `expression`.
    ListShape {
        expression: Box<ListExpr>,
    },

    Ref(ShapeExprLabel),
}

/// rdf-array: the regular expression of a [`ShapeExpr::ListShape`], over
/// shape expressions. `min` and `max` are as on triple expressions: absent
/// means 1, and -1 is unbounded. `ListEachOf` is ordered, unlike a triple
/// expression's `EachOf`. `Undef` (rdf-undef) matches an empty slot of the
/// sequence and nothing else, and it is not a shape expression.
#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
#[serde(tag = "type")]
pub enum ListExpr {
    Member {
        #[serde(rename = "shapeExpr")]
        shape_expr: Box<ShapeExprWrapper>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        min: Option<i32>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        max: Option<i32>,
    },
    Undef {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        min: Option<i32>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        max: Option<i32>,
    },
    ListEachOf {
        expressions: Vec<ListExpr>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        min: Option<i32>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        max: Option<i32>,
    },
    ListOneOf {
        expressions: Vec<ListExpr>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        min: Option<i32>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        max: Option<i32>,
    },
}

impl ListExpr {
    /// A member with no cardinality of its own: exactly one sequence member.
    pub fn member(se: ShapeExpr) -> ListExpr {
        ListExpr::Member {
            shape_expr: Box::new(ShapeExprWrapper { se }),
            min: None,
            max: None,
        }
    }

    pub fn min_max(&self) -> (Option<i32>, Option<i32>) {
        match self {
            ListExpr::Member { min, max, .. }
            | ListExpr::Undef { min, max }
            | ListExpr::ListEachOf { min, max, .. }
            | ListExpr::ListOneOf { min, max, .. } => (*min, *max),
        }
    }

    /// Sets this expression's cardinality; one that has a cardinality already,
    /// as in `( x {2} ) {3}`, is wrapped in a `ListEachOf` of one first.
    pub fn with_cardinality(self, new_min: Option<i32>, new_max: Option<i32>) -> ListExpr {
        match self {
            ListExpr::Member {
                shape_expr,
                min: None,
                max: None,
            } => ListExpr::Member {
                shape_expr,
                min: new_min,
                max: new_max,
            },
            ListExpr::Undef { min: None, max: None } => ListExpr::Undef {
                min: new_min,
                max: new_max,
            },
            ListExpr::ListEachOf {
                expressions,
                min: None,
                max: None,
            } => ListExpr::ListEachOf {
                expressions,
                min: new_min,
                max: new_max,
            },
            ListExpr::ListOneOf {
                expressions,
                min: None,
                max: None,
            } => ListExpr::ListOneOf {
                expressions,
                min: new_min,
                max: new_max,
            },
            other => ListExpr::ListEachOf {
                expressions: vec![other],
                min: new_min,
                max: new_max,
            },
        }
    }

    /// The shape expressions of the expression's members, in order.
    pub fn member_shape_exprs(&self) -> Vec<&ShapeExpr> {
        match self {
            ListExpr::Member { shape_expr, .. } => vec![&shape_expr.se],
            ListExpr::Undef { .. } => vec![],
            ListExpr::ListEachOf { expressions, .. } | ListExpr::ListOneOf { expressions, .. } => {
                expressions.iter().flat_map(ListExpr::member_shape_exprs).collect()
            },
        }
    }

    /// The shape expressions of the expression's members, mutably, in order.
    pub fn member_shape_exprs_mut(&mut self) -> Vec<&mut ShapeExpr> {
        match self {
            ListExpr::Member { shape_expr, .. } => vec![&mut shape_expr.se],
            ListExpr::Undef { .. } => vec![],
            ListExpr::ListEachOf { expressions, .. } | ListExpr::ListOneOf { expressions, .. } => expressions
                .iter_mut()
                .flat_map(ListExpr::member_shape_exprs_mut)
                .collect(),
        }
    }
}

impl DerefIri for ListExpr {
    fn deref_iri(self, base: Option<&IriS>, prefixmap: Option<&PrefixMap>) -> Result<Self, DerefError> {
        let deref_all = |expressions: Vec<ListExpr>| {
            expressions
                .into_iter()
                .map(|e| e.deref_iri(base, prefixmap))
                .collect::<Result<Vec<_>, _>>()
        };
        match self {
            ListExpr::Member { shape_expr, min, max } => Ok(ListExpr::Member {
                shape_expr: shape_expr.deref_iri(base, prefixmap)?,
                min,
                max,
            }),
            ListExpr::Undef { min, max } => Ok(ListExpr::Undef { min, max }),
            ListExpr::ListEachOf { expressions, min, max } => Ok(ListExpr::ListEachOf {
                expressions: deref_all(expressions)?,
                min,
                max,
            }),
            ListExpr::ListOneOf { expressions, min, max } => Ok(ListExpr::ListOneOf {
                expressions: deref_all(expressions)?,
                min,
                max,
            }),
        }
    }
}

impl FromStr for ShapeExpr {
    type Err = RefError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let ref_ = ShapeExprLabel::from_str(s)?;
        Ok(ShapeExpr::Ref(ref_))
    }
}

impl SerializeStringOrStruct for ShapeExpr {
    fn serialize_string_or_struct<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match &self {
            ShapeExpr::Ref(r) => r.serialize(serializer),
            _ => self.serialize(serializer),
        }
    }
}

impl ShapeExpr {
    pub fn empty_shape() -> ShapeExpr {
        ShapeExpr::Shape(Shape::default())
    }

    pub fn external() -> ShapeExpr {
        ShapeExpr::External
    }

    pub fn shape_not(se: ShapeExpr) -> ShapeExpr {
        ShapeExpr::ShapeNot {
            shape_expr: Box::new(ShapeExprWrapper { se }),
        }
    }

    pub fn or(ses: Vec<ShapeExpr>) -> ShapeExpr {
        /*let shape_exprs = ses
        .into_iter()
        .map(|shape_expression| Box::new(shape_expression.into()))
        .collect(); */
        let mut shape_exprs = Vec::new();
        for se in ses {
            shape_exprs.push(se.into())
        }
        ShapeExpr::ShapeOr { shape_exprs }
    }

    pub fn and(ses: Vec<ShapeExpr>) -> ShapeExpr {
        /* let shape_exprs = ses
        .into_iter()
        .map(|shape_expression| Box::new(shape_expression.into()))
        .collect(); */
        let mut shape_exprs = Vec::new();
        for se in ses {
            shape_exprs.push(se.into())
        }
        ShapeExpr::ShapeAnd { shape_exprs }
    }

    /// rdf-array: the LIST shape expression over the regular expression `expression`.
    pub fn list_shape(expression: ListExpr) -> ShapeExpr {
        ShapeExpr::ListShape {
            expression: Box::new(expression),
        }
    }

    pub fn node_constraint(nc: NodeConstraint) -> ShapeExpr {
        ShapeExpr::NodeConstraint(nc)
    }

    pub fn iri_ref(iri_ref: IriRef) -> ShapeExpr {
        ShapeExpr::Ref(ShapeExprLabel::iri_ref(iri_ref))
    }

    pub fn shape_ref(label: ShapeExprLabel) -> ShapeExpr {
        ShapeExpr::Ref(label)
    }

    pub fn any() -> ShapeExpr {
        ShapeExpr::default()
    }

    pub fn shape(shape: Shape) -> ShapeExpr {
        ShapeExpr::Shape(shape)
    }

    pub fn add_annotation(&mut self, annotation: Annotation) {
        match self {
            Self::Shape(s) => s.add_annotation(annotation),
            _ => todo!(),
        };
    }

    pub fn has_annotations(&self) -> bool {
        match self {
            Self::Shape(s) => s.has_annotations(),
            _ => todo!(),
        }
    }

    pub fn annotations(&self) -> Option<impl Iterator<Item = &Annotation>> {
        match self {
            Self::Shape(s) => s.annotations(),
            _ => todo!(),
        }
    }

    pub fn is_ref(&self) -> bool {
        matches!(self, Self::Ref(_))
    }

    pub fn is_datatype(&self) -> bool {
        matches!(self, Self::NodeConstraint(nc) if nc.is_datatype())
    }
}

impl Default for ShapeExpr {
    fn default() -> Self {
        ShapeExpr::Shape(Shape::default())
    }
}

impl DerefIri for ShapeExpr {
    fn deref_iri(self, base: Option<&IriS>, prefixmap: Option<&PrefixMap>) -> Result<Self, DerefError> {
        match self {
            ShapeExpr::External => Ok(ShapeExpr::External),
            ShapeExpr::ShapeAnd { shape_exprs } => {
                let shape_exprs = shape_exprs.deref_iri(base, prefixmap)?;
                Ok(ShapeExpr::ShapeAnd {
                    shape_exprs: shape_exprs.clone(),
                })
            },
            ShapeExpr::ShapeOr { shape_exprs } => {
                let shape_exprs = shape_exprs.deref_iri(base, prefixmap)?;
                Ok(ShapeExpr::ShapeOr {
                    shape_exprs: shape_exprs.clone(),
                })
            },
            ShapeExpr::ShapeNot { shape_expr } => Ok(ShapeExpr::ShapeNot {
                shape_expr: shape_expr.deref_iri(base, prefixmap)?,
            }),
            ShapeExpr::Shape(shape) => {
                let shape = shape.deref_iri(base, prefixmap)?;
                Ok(ShapeExpr::Shape(shape))
            },
            ShapeExpr::Ref(ref_) => {
                let ref_ = ref_.deref_iri(base, prefixmap)?;
                Ok(ShapeExpr::Ref(ref_))
            },
            ShapeExpr::NodeConstraint(nc) => {
                let nc = nc.deref_iri(base, prefixmap)?;
                Ok(ShapeExpr::NodeConstraint(nc))
            },
            ShapeExpr::ListShape { expression } => Ok(ShapeExpr::ListShape {
                expression: expression.deref_iri(base, prefixmap)?,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Pattern, StringFacet, XsFacet};

    use super::*;

    #[test]
    fn test_serde_xsfacet_pattern() {
        let facets: Vec<XsFacet> = vec![XsFacet::StringFacet(StringFacet::Pattern(Pattern::new("o*")))];
        let nc = NodeConstraint::new().with_xsfacets(facets);
        let se = ShapeExpr::NodeConstraint(nc);
        let json_nc = serde_json::to_string(&se).unwrap();
        assert_eq!(json_nc, "{\"type\":\"NodeConstraint\",\"pattern\":\"o*\"}");
    }

    #[test]
    fn test_serde_list_shape() {
        // One Member with a cardinality, which is a Collection's old shape.
        let json = r#"{"type":"ListShape","expression":{"type":"Member","shapeExpr":{"type":"NodeConstraint","nodeKind":"iri"},"min":1,"max":-1}}"#;
        let se: ShapeExpr = serde_json::from_str(json).unwrap();
        let ShapeExpr::ListShape { expression } = &se else {
            panic!("expected a ListShape, found {se:?}");
        };
        assert_eq!(expression.min_max(), (Some(1), Some(-1)));
        assert_eq!(serde_json::to_string(&se).unwrap(), json);

        // Nested groups, a reference as a member, and absent cardinalities.
        let json = r#"{"type":"ListShape","expression":{"type":"ListOneOf","expressions":[{"type":"ListEachOf","expressions":[{"type":"Member","shapeExpr":"http://a.example/T"},{"type":"Member","shapeExpr":{"type":"Shape"}}],"min":0,"max":-1},{"type":"Member","shapeExpr":"http://a.example/U"}]}}"#;
        let se: ShapeExpr = serde_json::from_str(json).unwrap();
        assert_eq!(serde_json::to_string(&se).unwrap(), json);

        assert!(
            serde_json::from_str::<ShapeExpr>(r#"{"type":"Collection","shapeExpr":"http://a.example/T"}"#).is_err()
        );
    }

    #[test]
    fn list_expr_wraps_a_second_cardinality() {
        let x = ListExpr::member(ShapeExpr::any());
        let twice = x.with_cardinality(Some(2), Some(2));
        assert!(matches!(twice, ListExpr::Member { min: Some(2), .. }));
        let thrice = twice.with_cardinality(Some(3), Some(3));
        match thrice {
            ListExpr::ListEachOf {
                expressions,
                min: Some(3),
                max: Some(3),
            } => assert_eq!(expressions.len(), 1),
            other => panic!("expected a ListEachOf of one, found {other:?}"),
        }
    }
}
