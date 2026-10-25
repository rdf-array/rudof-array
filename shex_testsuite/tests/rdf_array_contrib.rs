//! rdf-array: the shared ShEx-array tests of
//! [shexTest-contrib](https://github.com/rdf-array/shexTest-contrib) (the
//! `shexTest-contrib` submodule): the Collection shape expression in ShExC,
//! ShExJ and ShExR, validation over sequence-valued data, and the negative
//! syntax of its bounds. Every entry must pass; there is no baseline.

#[cfg(not(target_family = "wasm"))]
use shex_testsuite::{
    manifest::Manifest, manifest_mode::ManifestShExSyntaxMode, manifest_negative_syntax::ManifestNegativeSyntax,
    manifest_schemas::ManifestSchemas, manifest_validation::ManifestValidation,
};
#[cfg(not(target_family = "wasm"))]
use std::{fs, path::Path};

#[cfg(not(target_family = "wasm"))]
fn manifest_path(dir: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("shexTest-contrib")
        .join(dir)
        .join("manifest.jsonld")
}

#[cfg(not(target_family = "wasm"))]
fn run_all<M: Manifest>(manifest: &M, base: &Path, mode: ManifestShExSyntaxMode) {
    assert!(!manifest.is_empty(), "the manifest has no entries");
    let failing: Vec<String> = manifest
        .entry_names()
        .into_iter()
        .filter_map(|name| match manifest.run_entry(&name, base, mode) {
            Ok(()) => None,
            Err(e) => Some(format!("{name}: {e}")),
        })
        .collect();
    assert!(
        failing.is_empty(),
        "{} of {} entries failed in {mode:?}:\n  {}",
        failing.len(),
        manifest.len(),
        failing.join("\n  ")
    );
}

#[cfg(not(target_family = "wasm"))]
fn validation(mode: ManifestShExSyntaxMode) {
    let path = manifest_path("validation-contrib");
    let manifest: ManifestValidation =
        serde_json::from_str(&fs::read_to_string(&path).expect("read the manifest")).expect("parse the manifest");
    run_all(&manifest, path.parent().unwrap(), mode);
}

#[cfg(not(target_family = "wasm"))]
#[test]
fn contrib_validation_shexc() {
    validation(ManifestShExSyntaxMode::ShExC);
}

#[cfg(not(target_family = "wasm"))]
#[test]
fn contrib_validation_shexj() {
    validation(ManifestShExSyntaxMode::ShExJ);
}

#[cfg(not(target_family = "wasm"))]
#[test]
fn contrib_validation_shexr() {
    validation(ManifestShExSyntaxMode::ShExR);
}

/// Each Collection schema's ShExJ round-trips and equals its ShExC, and its
/// ShExC pretty-prints and parses back to the same schema.
#[cfg(not(target_family = "wasm"))]
#[test]
fn contrib_schemas() {
    let path = manifest_path("schemas-contrib");
    let manifest: ManifestSchemas =
        serde_json::from_str(&fs::read_to_string(&path).expect("read the manifest")).expect("parse the manifest");
    let base = path.parent().unwrap();
    run_all(&manifest, base, ManifestShExSyntaxMode::ShExC);
    let failing: Vec<String> = manifest
        .entry_names()
        .into_iter()
        .filter_map(|name| match manifest.run_pretty_print_roundtrip_entry(&name, base) {
            Ok(()) => None,
            Err(e) => Some(format!("{name}: {e}")),
        })
        .collect();
    assert!(
        failing.is_empty(),
        "pretty-print round trips failed:\n  {}",
        failing.join("\n  ")
    );
}

#[cfg(not(target_family = "wasm"))]
#[test]
fn contrib_negative_syntax() {
    let path = manifest_path("negativeSyntax-contrib");
    let manifest: ManifestNegativeSyntax =
        serde_json::from_str(&fs::read_to_string(&path).expect("read the manifest")).expect("parse the manifest");
    run_all(&manifest, path.parent().unwrap(), ManifestShExSyntaxMode::ShExC);
}
