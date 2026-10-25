#[cfg(test)]
mod tests {
    suite_tests! {
        crate::common::runner::validate_runner,
        "tests/data-shapes/data-shapes-test-suite/tests/core/complex/",
        personexample => "personexample",
        shacl_shacl_data_shapes => "shacl-shacl-data-shapes",
        #[ignore = "rdf-array: shacl-shacl's list shapes walk rdf:rest*/rdf:first ladders, but the shapes graph's lists are now sequences"]
        shacl_shacl => "shacl-shacl",
    }
}
