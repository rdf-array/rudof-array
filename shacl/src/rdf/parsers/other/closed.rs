use crate::ast::ASTComponent;
use rudof_iri::IriS;
use rudof_rdf::rdf_core::parser::rdf_node_parser::constructors::{SingleBoolPropertyParser, SingleValuePropertyParser};
use rudof_rdf::rdf_core::parser::rdf_node_parser::{ParserExt, RDFNodeParse};
use rudof_rdf::rdf_core::term::{Iri, Term};
use rudof_rdf::rdf_core::vocabs::ShaclVocab;
use rudof_rdf::rdf_core::{FocusRDF, RDFError};
use std::collections::HashSet;

pub(crate) fn closed<RDF: FocusRDF>() -> impl RDFNodeParse<RDF, Output = Vec<ASTComponent>> {
    SingleBoolPropertyParser::new(ShaclVocab::sh_closed())
        .optional()
        .then(move |maybe_closed| {
            ignored_properties().map(move |is| {
                maybe_closed.map_or(vec![], |b| {
                    vec![ASTComponent::Closed {
                        is_closed: b,
                        ignored_properties: is,
                    }]
                })
            })
        })
}

fn ignored_properties<RDF: FocusRDF>() -> impl RDFNodeParse<RDF, Output = HashSet<IriS>> {
    // The value is read apart from its members, since `optional` would swallow the error of
    // a slot among them: a list of a shapes graph may not hold one (rdf-undef).
    SingleValuePropertyParser::new(ShaclVocab::sh_ignored_properties())
        .optional()
        .flat_map(|node| {
            let mut hs = HashSet::new();
            let Some(elements) = node.as_ref().and_then(|n: &RDF::Term| n.sequence_elements()) else {
                return Ok(hs);
            };
            for element in elements {
                let v = element.ok_or_else(|| RDFError::SlotInListError {
                    node: node.as_ref().map(ToString::to_string).unwrap_or_default(),
                })?;
                if let Ok(iri) = RDF::term_as_iri(&v) {
                    let iri: RDF::IRI = iri;
                    let iri_string = iri.as_str();
                    let iri_s = IriS::new_unchecked(iri_string);
                    hs.insert(iri_s);
                } else {
                    return Err(RDFError::ExpectedIRIError { term: v.to_string() });
                }
            }
            Ok(hs)
        })
}
