use crate::rdf_core::{
    FocusRDF, RDFError,
    parser::rdf_node_parser::{RDFNodeParse, utils::parse_sequence},
};
use std::marker::PhantomData;

/// Parser that extracts all elements from an RDF list.
///
/// In rdf-array a list is one sequence term, `(e1 e2 …)`, and its elements are
/// returned in order; `rdf:first`/`rdf:rest` ladders are ordinary triples.
#[derive(Debug, Clone)]
pub struct ListParser<RDF> {
    _marker: PhantomData<RDF>,
}

impl<RDF> ListParser<RDF> {
    pub fn new() -> Self {
        ListParser { _marker: PhantomData }
    }
}

impl<RDF> Default for ListParser<RDF> {
    fn default() -> Self {
        Self::new()
    }
}

impl<RDF> RDFNodeParse<RDF> for ListParser<RDF>
where
    RDF: FocusRDF,
{
    type Output = Vec<RDF::Term>;

    fn parse_focused(&self, rdf: &mut RDF) -> Result<Self::Output, RDFError> {
        parse_sequence::<RDF>(rdf)
    }
}
