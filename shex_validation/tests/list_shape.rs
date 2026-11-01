//! rdf-array: the LIST shape expression, beyond what shexTest-contrib covers:
//! order, backtracking, nullable repetitions, large cardinalities, recursion
//! through a list, stratification, and the results reported.

use rudof_iri::IriS;
use rudof_rdf::rdf_core::RDFFormat;
use rudof_rdf::rdf_impl::{OxigraphInMemory, ReaderMode};
use shex_ast::ir::shape_label::ShapeLabel;
use shex_ast::ir::{map_state::MapState, schema_ir::SchemaIR, semantic_actions_registry::SemanticActionsRegistry};
use shex_ast::shapemap::{NodeSelector, QueryShapeMap, ShapeSelector};
use shex_ast::{Node, ResolveMethod, ShExParser, ShapeExprLabel, ir::ast2ir::AST2IR};
use shex_validation::{Validator, ValidatorConfig};

const PREFIXES: &str = "prefix : <http://example.org/>\nprefix xsd: <http://www.w3.org/2001/XMLSchema#>\n";

fn compile(schema_src: &str, config: &ValidatorConfig) -> Result<SchemaIR, String> {
    let base = IriS::new_unchecked("http://example.org/");
    let src = format!("{PREFIXES}{schema_src}");
    let ast = ShExParser::parse(&src, Some(base.clone()), &base).map_err(|e| e.to_string())?;
    let mut map_state = MapState::default();
    let registry = SemanticActionsRegistry::default();
    registry.set_map_state(&mut map_state);
    let mut compiler = AST2IR::new(&ResolveMethod::default(), map_state);
    let mut compiled = SchemaIR::new(registry);
    compiler
        .compile(
            &ast,
            &base,
            &Some(base.clone()),
            &mut compiled,
            config.external_resolvers(),
        )
        .map_err(|e| e.to_string())?;
    Ok(compiled)
}

/// Validates `:x` against `:S` in `schema` over `data`: whether it conforms,
/// and the text of the result.
fn validate(schema: &str, data: &str) -> (bool, String) {
    let config = ValidatorConfig::default();
    let compiled = compile(schema, &config).expect("compile");
    let validator = Validator::new(&compiled, &config).expect("validator");
    let data = format!("{PREFIXES}{data}");
    let graph = OxigraphInMemory::from_str(&data, &RDFFormat::Turtle, None, &ReaderMode::Strict).expect("parse graph");
    let label = ShapeLabel::iri(IriS::new_unchecked("http://example.org/S"));
    let node = Node::parse("http://example.org/x", None).unwrap();
    let mut shapemap = QueryShapeMap::new();
    shapemap
        .add_association(
            NodeSelector::Node((&node).try_into().unwrap()),
            &None,
            ShapeSelector::label(ShapeExprLabel::from(&label)),
            &None,
        )
        .unwrap();
    let result = validator
        .validate_shapemap(&shapemap, &graph, &compiled, &Some(graph.prefixmap().clone()))
        .expect("validate");
    let info = result.get_info(&node, &label).unwrap();
    (info.is_conformant(), format!("{info:?}"))
}

/// Whether the value `seq` of `:x :p` conforms to `{ :p value_expr }`.
fn conforms(value_expr: &str, seq: &str) -> bool {
    validate(&format!(":S {{ :p {value_expr} }}"), &format!(":x :p {seq} .")).0
}

#[test]
fn semicolon_is_ordered() {
    let pair = "LIST ( xsd:string ; xsd:integer )";
    assert!(conforms(pair, r#"( "a" 1 )"#));
    assert!(!conforms(pair, r#"( 1 "a" )"#));
    assert!(!conforms(pair, r#"( "a" )"#));
    assert!(!conforms(pair, r#"( "a" 1 2 )"#));
}

#[test]
fn matching_backtracks() {
    // The last member belongs to the second part, not the greedy first
    let dot_then_int = "LIST ( . * ; xsd:integer )";
    assert!(conforms(dot_then_int, r#"( "a" 1 2 )"#));
    assert!(!conforms(dot_then_int, r#"( 1 2 "a" )"#));
    // Either reading of the choice's prefix may be the one that finishes
    let choice = "LIST ( ( xsd:integer | xsd:integer ; xsd:integer ) ; xsd:string )";
    assert!(conforms(choice, r#"( 1 "a" )"#));
    assert!(conforms(choice, r#"( 1 2 "a" )"#));
    assert!(!conforms(choice, r#"( 1 2 3 "a" )"#));
    // A group that repeats, and a member that belongs to two parts in turn
    let groups = "LIST ( ( xsd:integer ; xsd:integer ) * ; xsd:integer ? ; xsd:string )";
    assert!(conforms(groups, r#"( "a" )"#));
    assert!(conforms(groups, r#"( 1 "a" )"#));
    assert!(conforms(groups, r#"( 1 2 "a" )"#));
    assert!(conforms(groups, r#"( 1 2 3 "a" )"#));
    assert!(!conforms(groups, r#"( 1 2 3 )"#));
}

#[test]
fn cardinalities() {
    assert!(conforms("LIST {2,3} xsd:integer", "( 1 2 )"));
    assert!(conforms("LIST {2,3} xsd:integer", "( 1 2 3 )"));
    assert!(!conforms("LIST {2,3} xsd:integer", "( 1 )"));
    assert!(!conforms("LIST {2,3} xsd:integer", "( 1 2 3 4 )"));
    assert!(conforms("LIST {0} xsd:integer", "( )"));
    assert!(!conforms("LIST {0} xsd:integer", "( 1 )"));
    assert!(conforms("LIST ? xsd:integer", "( )"));
    assert!(conforms("LIST ( ( xsd:integer {2} ) {3} )", "( 1 2 3 4 5 6 )"));
    assert!(!conforms("LIST ( ( xsd:integer {2} ) {3} )", "( 1 2 3 4 5 )"));
}

#[test]
fn nullable_repetitions_are_accepted() {
    for value_expr in [
        "LIST ( ( xsd:integer ? ) {3} )",
        "LIST ( ( xsd:integer ? ) * )",
        "LIST ( ( xsd:integer * ) * )",
        "LIST ( ( xsd:integer * ) + )",
        "LIST ( ( xsd:integer ? ; xsd:string ? ) {2,} )",
        "LIST ( ( xsd:integer ? ) {1000000000} )",
        "LIST ( ( ( xsd:integer ? ) * ) {1000000000,} )",
    ] {
        assert!(conforms(value_expr, "( )"), "{value_expr} on ()");
        assert!(conforms(value_expr, "( 1 2 )"), "{value_expr} on ( 1 2 )");
    }
    assert!(!conforms("LIST ( ( xsd:integer ? ) {3} )", "( 1 2 3 4 )"));
    assert!(!conforms("LIST ( ( xsd:integer ? ) * )", r#"( 1 "a" )"#));
    // A large cardinality that takes members ends with the sequence
    assert!(!conforms("LIST {1000000000} xsd:integer", "( 1 2 )"));
}

#[test]
fn a_sequence_satisfies_no_node_kind_or_datatype_but_matches_dot() {
    for value_expr in ["IRI", "BNODE", "LITERAL", "NONLITERAL", "xsd:string"] {
        assert!(!conforms(value_expr, "( 1 )"), "{value_expr}");
        assert!(
            !conforms(&format!("LIST ( {value_expr} )"), "( ( 1 ) )"),
            "{value_expr}"
        );
    }
    assert!(conforms(".", "( 1 )"));
    assert!(conforms("LIST ( . )", "( ( 1 ) )"));
    assert!(conforms("LIST + .", r#"( ( ) "a" <http://example.org/i> )"#));
    // A non-sequence is no list, even for `*`
    assert!(!conforms("LIST * .", "1"));
    assert!(!conforms(
        "LIST * .",
        "<http://www.w3.org/1999/02/22-rdf-syntax-ns#nil>"
    ));
    assert!(conforms("LIST * .", "( )"));
}

#[test]
fn members_may_be_any_shape_expression() {
    let s = ":S { :p @:L } :L LIST ( @:T ; NOT @:T ; @:T AND { :r . } ; . ) :T { :q . }";
    let data = r#":x :p ( :a :b :a :b ) . :a :q 1 ; :r 2 . :b :s 3 ."#;
    assert!(validate(s, data).0);
    let data = r#":x :p ( :a :a :a :b ) . :a :q 1 ; :r 2 . :b :s 3 ."#;
    assert!(!validate(s, data).0);
}

#[test]
fn recursion_through_a_list_is_not_a_circular_reference() {
    let tree = ":S { :p @:Tree } :Tree LIST ( xsd:string ; LIST ( @:Tree * ) )";
    let good = r#":x :p ( "root" ( ( "left" ( ) ) ( "right" ( ( "leaf" ( ) ) ) ) ) ) ."#;
    assert!(validate(tree, good).0);
    let bad = r#":x :p ( "root" ( ( "left" ( ) ) ( "right" ( ( 1 ( ) ) ) ) ) ) ."#;
    assert!(!validate(tree, bad).0);

    // Directly in the declaration, without a member's own LIST
    let direct = ":S { :p @:L } :L LIST ( @:L * )";
    assert!(validate(direct, ":x :p ( ( ) ( ( ) ) ) .").0);
    assert!(!validate(direct, ":x :p ( ( ) 1 ) .").0);

    // A node can reach itself through a list and a triple constraint
    let cycle = ":S { :p LIST ( @:S * ) }";
    assert!(validate(cycle, ":x :p ( :x :y ) . :y :p ( ) .").0);
    assert!(!validate(cycle, ":x :p ( :x :y ) . :y :q ( ) .").0);

    // `EXTENDS` through a list is not an extension cycle either
    let config = ValidatorConfig::default();
    assert!(compile(":S EXTENDS @:B { :q . } :B { :p @:L } :L LIST ( @:S * )", &config).is_ok());
}

#[test]
fn negation_through_a_list_counts_for_stratification() {
    let config = ValidatorConfig::default();
    for schema in [
        ":S { :p LIST ( NOT @:S * ) }",
        ":S { :p LIST * ( NOT @:S ) }",
        ":S LIST ( xsd:string ; LIST ( NOT @:S ) )",
        ":S NOT LIST ( @:S * )",
        ":S { :p @:L } :L LIST ( xsd:string | NOT @:S )",
    ] {
        let compiled = compile(schema, &config).expect("compile");
        assert!(compiled.has_neg_cycle(), "{schema} negates :S in :S");
    }
    for schema in [
        ":S { :p LIST ( @:S * ) }",
        ":S LIST ( xsd:string ; LIST ( @:S * ) )",
        ":S { :p LIST ( NOT @:T ) } :T { :q . }",
    ] {
        let compiled = compile(schema, &config).expect("compile");
        assert!(!compiled.has_neg_cycle(), "{schema} is stratified");
    }
}

#[test]
fn failures_name_the_member_or_the_length() {
    let colour = ":S { :p LIST {3} xsd:integer }";
    let (ok, text) = validate(colour, ":x :p ( 1 2 ) .");
    assert!(!ok);
    assert!(text.contains("has 2 members and the expression takes 3 to 3"), "{text}");
    let (ok, text) = validate(colour, r#":x :p ( 1 2 "a" ) ."#);
    assert!(!ok);
    assert!(text.contains("member 3 ("), "{text}");
    let (ok, text) = validate(colour, ":x :p 1 .");
    assert!(!ok);
    assert!(text.contains("is not a sequence"), "{text}");
    assert!(validate(colour, ":x :p ( 1 2 3 ) .").0);
}
