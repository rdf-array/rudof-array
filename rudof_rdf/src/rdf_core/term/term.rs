use std::{
    fmt::{Debug, Display},
    hash::Hash,
};

/// Represents any RDF term that can appear in an RDF graph.
///
/// In RDF, terms are the fundamental building blocks of triples. A term can be
/// an IRI (Internationalized Resource Identifier), a blank node, a literal value,
/// a quoted triple (in RDF-star) or a sequence of terms (in rdf-array).
pub trait Term: Debug + Clone + Display + PartialEq + Eq + Hash {
    /// Returns the kind of RDF term this represents.
    ///
    /// This method allows distinguishing between different types of RDF terms
    /// at runtime.
    fn kind(&self) -> TermKind;

    /// Returns `true` if this term is an IRI.
    fn is_iri(&self) -> bool {
        self.kind() == TermKind::Iri
    }

    /// Returns `true` if this term is a blank node.
    fn is_blank_node(&self) -> bool {
        self.kind() == TermKind::BlankNode
    }

    /// Returns `true` if this term is a literal.
    fn is_literal(&self) -> bool {
        self.kind() == TermKind::Literal
    }

    /// Returns `true` if this term is a quoted triple (RDF-star).
    fn is_triple(&self) -> bool {
        self.kind() == TermKind::Triple
    }

    /// Returns `true` if this term is a sequence (rdf-array).
    fn is_sequence(&self) -> bool {
        self.kind() == TermKind::Sequence
    }

    /// Returns the lexical representation of this term as a string.
    fn lexical_form(&self) -> String;

    /// Returns the elements of this term if it is a sequence (rdf-array).
    fn sequence_elements(&self) -> Option<Vec<Self>>;

    /// Builds a sequence term (rdf-array) holding `elements` in order.
    fn sequence(elements: Vec<Self>) -> Self;
}

/// Represents the kind of RDF term.
#[derive(PartialEq)]
pub enum TermKind {
    Iri,
    BlankNode,
    Literal,
    Triple,
    /// An rdf-array sequence: `(e1 e2 …)` is one term, `()` the empty sequence.
    Sequence,
}
