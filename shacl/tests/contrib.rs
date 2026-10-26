//! rdf-array: data-shapes-contrib (https://github.com/rdf-array/data-shapes-contrib), the SHACL-array
//! tests no W3C suite has, from the `data-shapes-contrib` submodule.

#[cfg(test)]
mod path {
    suite_tests! {
        crate::common::runner::validate_runner,
        "tests/data-shapes-contrib/tests/core/path/",
        seq_path_001 => "seq-path-001",
        seq_path_alternative_nested_001 => "seq-path-alternative-nested-001",
        nil_predicate_path_001 => "nil-predicate-path-001",
        #[ignore = "rdf-array: the runner does not read sht:Failure expectations (an ill-formed shapes graph); bin/build.py in data-shapes-contrib checks that rudof-array refuses it"]
        ladder_path_001 => "ladder-path-001",
        ladder_path_002 => "ladder-path-002",
        #[ignore = "rdf-array: expects sht:Failure, and rudof-array accepts sh:path (), an empty sequence path"]
        empty_path_001 => "empty-path-001",
    }
}

#[cfg(test)]
mod node {
    suite_tests! {
        crate::common::runner::validate_runner,
        "tests/data-shapes-contrib/tests/core/node/",
        #[ignore = "rdf-array: rudof-array also reports the results of the shapes inside sh:or as results of the focus node (not specific to sequences)"]
        seq_in_or_001 => "seq-in-or-001",
        seq_language_in_001 => "seq-languageIn-001",
        seq_closed_ignored_001 => "seq-closed-ignored-001",
        #[ignore = "rdf-array: the runner does not read sht:Failure expectations (an ill-formed shapes graph); bin/build.py in data-shapes-contrib checks that rudof-array refuses it"]
        ladder_in_001 => "ladder-in-001",
        #[ignore = "rdf-array: rudof has no SHACL 1.2 list constraints (sh:memberShape, sh:minListLength, sh:maxListLength, sh:uniqueMembers) yet"]
        in_empty_001 => "in-empty-001",
        #[ignore = "rdf-array: rudof has no SHACL 1.2 list constraints (sh:memberShape, sh:minListLength, sh:maxListLength, sh:uniqueMembers) yet"]
        xone_empty_001 => "xone-empty-001",
    }
}

#[cfg(test)]
mod list {
    suite_tests! {
        crate::common::runner::validate_runner,
        "tests/data-shapes-contrib/tests/core/list/",
        #[ignore = "rdf-array: rudof has no SHACL 1.2 list constraints (sh:memberShape, sh:minListLength, sh:maxListLength, sh:uniqueMembers) yet"]
        membershape_property_001 => "memberShape-property-001",
        #[ignore = "rdf-array: rudof has no SHACL 1.2 list constraints (sh:memberShape, sh:minListLength, sh:maxListLength, sh:uniqueMembers) yet"]
        membershape_node_001 => "memberShape-node-001",
        #[ignore = "rdf-array: rudof has no SHACL 1.2 list constraints (sh:memberShape, sh:minListLength, sh:maxListLength, sh:uniqueMembers) yet"]
        listlength_property_001 => "listLength-property-001",
        #[ignore = "rdf-array: rudof has no SHACL 1.2 list constraints (sh:memberShape, sh:minListLength, sh:maxListLength, sh:uniqueMembers) yet"]
        listlength_node_001 => "listLength-node-001",
        #[ignore = "rdf-array: rudof has no SHACL 1.2 list constraints (sh:memberShape, sh:minListLength, sh:maxListLength, sh:uniqueMembers) yet"]
        uniquemembers_property_001 => "uniqueMembers-property-001",
        #[ignore = "rdf-array: rudof has no SHACL 1.2 list constraints (sh:memberShape, sh:minListLength, sh:maxListLength, sh:uniqueMembers) yet"]
        uniquemembers_node_001 => "uniqueMembers-node-001",
        #[ignore = "rdf-array: rudof has no SHACL 1.2 list constraints (sh:memberShape, sh:minListLength, sh:maxListLength, sh:uniqueMembers) yet"]
        rgb_colour_001 => "rgb-colour-001",
        #[ignore = "rdf-array: rudof has no SHACL 1.2 list constraints (sh:memberShape, sh:minListLength, sh:maxListLength, sh:uniqueMembers) yet"]
        polygon_001 => "polygon-001",
        #[ignore = "rdf-array: rudof has no SHACL 1.2 list constraints (sh:memberShape, sh:minListLength, sh:maxListLength, sh:uniqueMembers) yet"]
        playlist_001 => "playlist-001",
    }
}
