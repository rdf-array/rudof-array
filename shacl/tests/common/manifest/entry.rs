use crate::common::TestSuiteError;
use crate::common::manifest::ManifestGraph;
use oxrdf::Subject;

/// A manifest entry
pub(crate) trait ManifestEntry: Sized {
    fn parse(manifest: &mut ManifestGraph, entry: Subject) -> Result<Self, TestSuiteError>;
}
