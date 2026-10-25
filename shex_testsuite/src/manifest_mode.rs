use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum ManifestMode {
    Schemas,
    Validation,
    NegativeSyntax,
    NegativeStructure,
}

/// The syntax mode to use when parsing the manifest. This is only relevant for the Validation tests
#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
#[value(rename_all = "lowercase")]
pub enum ManifestShExSyntaxMode {
    ShExJ,
    ShExC,
    /// The schema written in RDF (the `.ttl` beside the `.shex`), read by the
    /// ShExR parser; in rdf-array its lists are sequences.
    ShExR,
}
