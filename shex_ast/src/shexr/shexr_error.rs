use rudof_iri::IriS;
use rudof_rdf::rdf_core::RDFError;
use std::fmt::Display;
use thiserror::Error;

use crate::Node;

#[derive(Debug, Error)]
pub enum ShExRError {
    #[error("RDF parse error: {err}")]
    RDFParseError {
        #[from]
        err: RDFError,
    },

    /// rdf-array: a LIST's expression must be a Member, ListEachOf or ListOneOf.
    #[error("Node {node} should be an sx:Member, sx:ListEachOf or sx:ListOneOf, but its type is {found:?}")]
    UnexpectedListExprType { node: String, found: Option<String> },

    #[error("No nodes with `rdf:type sx:Schema`")]
    NoSchemaNodes,

    #[error("More than one nodes with `rdf:type sx:Schema`")]
    MoreThanOneSchema { nodes: Nodes },

    #[error("Shape Label can not be a literal {term}")]
    ShapeExprLabelLiteral { term: String },

    #[error("Unexpected value for nodeKind: {iri}")]
    UnexpectedNodeKind { iri: IriS },

    #[error("Expected RDFNode parsing node shape, found: {term}")]
    TermToRDFNodeFailed { term: String },
}

#[derive(Debug)]

pub struct Nodes {
    values: Vec<Node>,
}

impl Nodes {
    pub fn new(ns: Vec<Node>) -> Nodes {
        Nodes { values: ns }
    }
}

impl Display for Nodes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for n in self.values.iter() {
            write!(f, "{n} ")?;
        }
        writeln!(f)?;
        Ok(())
    }
}
