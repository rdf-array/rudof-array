#[cfg(test)]
mod tests {
    suite_tests! {
        crate::common::runner::validate_runner,
        "tests/data-shapes/data-shapes-test-suite/tests/core/complex/",
        personexample => "personexample",
        shacl_shacl_data_shapes => "shacl-shacl-data-shapes",
        #[ignore = "rdf-array: data-shapes-array's shacl-shacl checks lists with sh:memberShape and sh:minListLength (mf:requires ra:ShaclListConstraints), which rudof does not have yet, so its sh:xone over path kinds matches twice"]
        shacl_shacl => "shacl-shacl",
    }
}
