# rudof-array: rudof with sequences as terms

rdf-array is RDF whose data model has one more kind of term, the **sequence**: `(1 2 3)` in Turtle
is one term holding its elements in order, not a ladder of `rdf:first`/`rdf:rest` blank nodes, and
`()` is the empty sequence, not `rdf:nil`. Triples that spell out `rdf:first`/`rdf:rest` stay
ordinary triples and `rdf:nil` an ordinary IRI; they are just no longer what `( … )` means. See
[rdf-array.md](https://rdf-array.github.io/rdf-array/rdf-array.html) for the model and the proposed
SPARQL and ShEx syntax.

This fork of [rudof](https://github.com/rudof-project/rudof) is the ShEx-array implementation in
Rust. rudof reads, stores and queries RDF through the Oxigraph crates, so it takes the term, the
syntaxes and SPARQL-array from [oxigraph-array](https://github.com/rdf-array/oxigraph-array), and
adds the ShEx-array **Collection shape expression** `((shapeExpr))[min, max]` in ShExC, ShExJ and
ShExR, with its validation.

| crate | from | what it does here | version |
|---|---|---|---|
| `oxrdf`, `oxttl`, `oxrdfxml`, `oxjsonld`, `oxrdfio`, `oxsdatatypes`, `spargebra`, `sparopt`, `spareval`, `sparesults`, `oxigraph` | [oxigraph-array](https://github.com/rdf-array/oxigraph-array), branch `rdf-sequences` (`[patch.crates-io]` in `Cargo.toml`) | the sequence term (`oxrdf::Sequence`, `Term::Sequence`, and `oxrdf::Subject` = IRI, blank node or sequence as a triple's subject); every syntax reads and writes `( … )`; SPARQL-array; the store | oxigraph 0.5.13, oxrdf 0.3.5 (rudof 0.3.35 pins oxigraph 0.5.9, oxrdf 0.3.3) |
| `rudof_iri`, `rudof_rdf` | this fork | rudof's RDF abstraction: `Object::Sequence`, `TermKind::Sequence`, `Term::sequence_elements`/`Term::sequence`; lists read and written as sequences | rudof 0.3.35 |
| `shex_ast` | this fork | the Collection shape expression in the ShExJ AST, the ShExC grammar and printer, ShExR (reader and writer), and the IR | rudof 0.3.35 |
| `shex_validation` | this fork | validates a node against a Collection | rudof 0.3.35 |
| `shex_testsuite` | this fork | runs [shexTest-array](https://github.com/rdf-array/shexTest-array) and [shexTest-contrib](https://github.com/rdf-array/shexTest-contrib); a ShExR syntax mode | rudof 0.3.35 |
| `shacl`, `sparql_service`, `rudof_lib`, `rudof_generate`, `shapes_converter`, `shapes_comparator` | this fork | follow the new subject and term types; SHACL lists and paths are sequences | rudof 0.3.35 |

The branch is `rdf-sequences`, from the tag `0.3.35`. The other ShEx-array implementations are
[shex.js-array](https://github.com/rdf-array/shex.js-array) (JavaScript),
[PyShEx-array](https://github.com/rdf-array/PyShEx-array) (Python) and
[jena-array](https://github.com/rdf-array/jena-array) (Java).

## Setting up

```bash
git clone --recurse-submodules -b rdf-sequences git@github.com:rdf-array/rudof-array.git
cd rudof-array
cargo test --workspace
cargo run -p shex_testsuite -- -m shex_testsuite/shexTest-contrib/validation-contrib/manifest.jsonld -s shexr
```

The submodules are `shex_testsuite/shexTest` ([shexTest-array](https://github.com/rdf-array/shexTest-array),
branch `rdf-sequences`), `shex_testsuite/shexTest-contrib` ([shexTest-contrib](https://github.com/rdf-array/shexTest-contrib),
branch `main`) and `shacl/tests/data-shapes` (W3C's, unchanged). `git submodule update --remote` moves
the first two to their branches' heads.

## What a sequence does here

```shex
PREFIX : <http://a.example/>
PREFIX xsd: <http://www.w3.org/2001/XMLSchema#>

:Colour   { :rgb ((xsd:integer MININCLUSIVE 0 MAXINCLUSIVE 255))[3] }
:Polygon  { :points ((((xsd:decimal))[2]))[3, *] }
:Playlist { :tracks ((@:Track))[1, *] }
:Track    { :title xsd:string }
:Anything { :seq ((.)) }
:L        ((IRI))[1, *]
```

```turtle
:red :rgb (255 0 0) .                        # conforms to :Colour
:pink :rgb (255 192) .                       # does not: two elements
:tri :points ((0.0 0.0) (1.0 0.0) (0.0 1.0)) .
```

- **Data.** `Object::Sequence(Vec<Object>)` is rudof's own term for a sequence (shape maps,
  results, reasons and errors carry it), spelled `(e1 e2)` by `Display` and `( e1 e2 )` with
  prefixes by `show_qualified`. It orders after IRIs, blank nodes, literals and triple terms,
  element by element, a prefix first. The Oxigraph-backed graphs (`OxigraphInMemory`,
  `OxigraphEndpoint`, `RdfData`, the QLever container, SHACL's cached store) have
  `oxrdf::Subject` as their subject type, so a sequence can be a subject; `IriOrBlankNode`, used
  for the subject of a triple term and for shape labels, stays an IRI or a blank node, and a
  sequence there is an `ExpectedIriOrBlankNodeFoundSequence` error. Blank node relabelling
  (`BlankNodeMode`) reaches blank nodes inside sequences.
- **ShExC.** `((shapeExpr))` is a shape atom wherever one may stand (a declaration, a triple
  constraint's value, under `NOT`, `AND`, `OR`), optionally followed by bounds: `[m]` is exactly
  m elements, `[m, n]` from m to n, `[m, *]` (or `[m,]`) at least m; no bounds means any length.
  `((x))` is always a Collection, never `x` inside two pairs of parentheses; `((x) OR y)` is still
  a parenthesised `OR`. Once `[` is read the bounds must follow, so `((.))[]`, `[1 2]`, `[,1]`,
  `[*]`, `[1, 2, 3]` and an unclosed `[1` are syntax errors, as is a minimum above the maximum.
  The compact printer writes Collections back in the same form.
- **ShExJ.** `{"type": "Collection", "shapeExpr": …, "min": 1, "max": -1}`; `min` and `max` are
  optional (absent: 0 and unbounded; `max` -1 for `*`).
- **ShExR.** `[ a sx:Collection ; sx:shapeExpr … ; sx:min 1 ; sx:max -1 ]`, read and written. The
  schema's lists (`sx:shapes`, `sx:shapeExprs`, `sx:expressions`, `sx:values`, …) are sequences:
  the reader takes `sx:shapes ( … )` as the one sequence term it now is, and the writer
  (`ShExRBuilder`) writes one sequence instead of a ladder.
- **Validation.** A node conforms to `((e))[m, n]` when it is a sequence of m to n elements each of
  which conforms to `e`; each element is a focus node of its own, like a triple constraint's value.
  The failures are `SequenceExpected` (the node is not a sequence), `CollectionLengthViolation` and
  `CollectionElementFailure` (with the 1-based index of the first failing element and its
  errors); a pass is a `Reason::Collection` holding the elements' reasons in order. A sequence
  satisfies no `nodeKind` (`IRI`, `BNODE`, `LITERAL`, `NONLITERAL`), no datatype and no string
  facet, but does match `.`, as in shex.js-array.
- **Lists elsewhere.** rudof's generic list reader (`parse_sequence`, used by `ListParser` and
  `.list()`) and list writer (`BuildRDF::add_rdf_list`) read and write sequences, so SHACL shapes
  graphs written in Turtle-array work as before: `sh:path ( :p :q )` is a sequence path, `sh:in`,
  `sh:or`, `sh:languageIn`, `sh:ignoredProperties` are sequences, and SHACL 1.2's list node
  expression is a sequence (`()` the empty list). The SHACL test manifests' `mf:entries ( … )`
  are read as sequences too. `NilParser` accepts `()`.

## What changed

- `Cargo.toml`, `Cargo.lock`: `[patch.crates-io]` onto oxigraph-array. Cargo resolved `sha1` to
  0.10 for both its users after the patch, and spareval needs 0.11; the lock gives spareval
  `sha1 0.11.0` as upstream's did.
- `rudof_iri/src/iri/iris.rs`: `From<IriS> for oxrdf::Subject`.
- `rudof_rdf`: `term/term.rs` (`TermKind::Sequence`, `is_sequence`, `sequence_elements`,
  `sequence`), `term/triple.rs` (`Object::Sequence`, its conversions, ordering, display;
  `show_ox_sequence`), `term/iri_or_blanknode.rs`, `errors.rs`
  (`ExpectedIriOrBlankNodeFoundSequence`, `ExpectedSequenceError` replacing
  `RecursiveRDFListError`), `rdf_impl/oxigraph/{oxrdf_impl,in_memory,endpoint}.rs`,
  `rdf_impl/{backend,qlever/graph_container}.rs`, `rdf_core/blank_node_mode.rs`,
  `rdf_core/build_rdf.rs`, `parser/rdf_node_parser/{utils,combinators}.rs`,
  `parser/rdf_node_parser/constructors/{list,validation,shacl}.rs`,
  `query/query_solutions.rs`, `visualizer/visual_rdf_node.rs` (a sequence is drawn as one node
  labelled with its spelling), `vocabs/shexr.rs` (`sx:Collection`).
- `shex_ast`: `ast/shape_expr.rs` (`ShapeExpr::Collection`), `compact/shex_grammar.rs`
  (`collection_shape_expr`, `collection_bounds`), `compact/shex_parser_error.rs`,
  `compact/shex_compact_printer.rs`, `compact/shex_parser.rs`, `ir/shape_expr.rs`
  (`ShapeExpr::Collection { expr, min, max }`), `ir/ast2ir.rs`, `ir/schema_ir.rs`,
  `ir/schema_ir_error.rs`, `ir/shape_label.rs`, `node.rs`, `ast/cond_kind.rs`,
  `shexr/shexr_parser.rs`, `shexr/shexr_builder/mod.rs`, `materialize/materializer.rs`.
- `shex_validation`: `engine.rs` (`check_collection`; a Collection is checked like a connective),
  `reason.rs`, `validator_error.rs`.
- `shacl`: `rdf/parsers/node_expr/{expr,mod}.rs` (the ladder-detecting `list.rs` is gone),
  `validator/constraints/mod.rs` (a sequence in SPARQL is `( e1 e2 )`),
  `validator/store/cached.rs`, `tests/common/manifest/*.rs`.
- `sparql_service`, `rudof_lib` (`pg_db`), `rudof_generate`, `shapes_converter`,
  `shapes_comparator`: the new subject type and term variant; the converters (to HTML, UML,
  SPARQL, CoShaMo) report a Collection as not implemented.
- `shex_testsuite`: `ManifestShExSyntaxMode::ShExR` (`-s shexr` reads the `.ttl` beside each
  `.shex`), `tests/rdf_array_contrib.rs`, the submodules, `config.toml`, the baselines.

## Tests

- **Added.** `shex_ast`: `test_serde_collection` (ShExJ), `test_collection_shape_expressions`
  (ShExC, including the bad bounds), `collections_are_printed_and_roundtrip` (the printer),
  `contrib_schemas_round_trip_between_formats` (every shexTest-contrib schema through
  ShExC → ShExJ, ShExC → ShExR, ShExC → ShExC, ShExJ → ShExC and ShExR → ShExC).
  `shex_testsuite/tests/rdf_array_contrib.rs`: shexTest-contrib's validation manifest in ShExC,
  ShExJ and ShExR, its schemas manifest (ShExJ against ShExC, and the printer round trip) and its
  negative syntax manifest, with no baseline. `rudof_rdf`: `add_rdf_list_empty_is_the_empty_sequence`
  replaces `add_rdf_list_empty_is_nil`; `shex_ast`'s `builds_an_empty_shape_declaration` counts
  five triples instead of seven (`sx:shapes` holds one sequence, not a two-triple ladder).
- **shexTest-array** (branch `rdf-sequences`), with `cargo run -p shex_testsuite`, one entry at a
  time so that a crash costs one entry:

  | manifest | rudof-array on shexTest-array | rudof 0.3.35 on upstream shexTest 642f0f7 |
  |---|---|---|
  | validation, ShExC | 1300 passed, 4 failed, 2 crashed (of 1306) | 1303 passed, 4 failed, 2 crashed (of 1309) |
  | validation, ShExJ | 1289 passed, 15 failed, 2 crashed | 1292 passed, 15 failed, 2 crashed |
  | validation, ShExR | 1266 passed, 38 failed, 2 crashed | — (no ShExR mode) |
  | schemas | 481 passed, 3 failed | 481 passed, 3 failed |
  | negativeSyntax | 105 passed | 105 passed |
  | negativeStructure | 17 passed, 2 failed | 17 passed, 2 failed |

  The failures are the same entries on both sides: none comes from sequences. The three entries
  fewer are the `1list0PlusDot-manualList_*` tests, which walk an `rdf:first`/`rdf:rest` ladder by
  hand and which shexTest-array comments out of its manifest; the `1list*` tests it rewrote as
  Collections pass. In ShExR mode the 34 further failures are entries with no ShExR form: the
  `.ttl` beside the `.shex` is missing (`1dot-relative`, `extends-abstract-multi-empty`,
  `extends-closed-diamond`, `extends-closed-3diamond-split`) or is the test's data, not a schema
  (`nPlus1`, `PstarT`, `PTstar`, `P2T2`, `false-lead-excluding-value-shape`, `skipped`, …).
  `shex_ast`'s format round trip reads all 484 schemas' `.ttl` forms, with their lists as
  sequences, and passes.
- **Suite drift, not sequences.** shexTest-array is 35 upstream commits newer than the shexTest
  rudof 0.3.35 pins, and vanilla 0.3.35 fails the same entries on upstream's newer suite:
  `8dot5a2b1c_fail-a5b1` and `8dot5a2b1c_fail-a6b2c1` overflow the stack and abort the whole run
  (excluded in `config.toml`); `1dotShapeCode1fail_abort`, `open2OnecardStarclosecardStar_fail-aI1`,
  `open2OnedotcloseCode1fail_abort`, `open3EachdotcloseCode1fail_abort` fail
  (`tests/baseline_failing.txt`); `1card1` (added by shexTest 754575c) loses its `{1}` in the
  printer round trip (`tests/baseline_pretty_print_failing.txt`, and skipped in
  `format_roundtrip.rs`).
- **shexTest-contrib** (branch `main`): validation 37 of 37 in each of ShExC, ShExJ and ShExR;
  schemas 13 of 13; negative syntax 6 of 6.
- **Flagged** (`#[ignore = "rdf-array: …"]`, in the W3C SHACL suites, which have no -array fork):
  `core/path/path-strange-001` and `-002` (their `sh:path` is spelled as an explicit
  `rdf:first`/`rdf:rest` ladder, which is no longer a list), `core/complex/shacl-shacl`
  (shacl-shacl's list shapes walk `rdf:rest*/rdf:first`, but the shapes graph's lists are now
  sequences), `shacl12/node-expr/list` (`()` is the empty sequence, a list expression with no
  members, not the constant `rdf:nil`). The other SHACL tests, the sequence paths and `sh:in`
  among them, pass with their lists read as sequences.
- **Numbers.** `cargo test --workspace`: 1456 passed, 0 failed, 30 ignored (26 ignored upstream,
  the 4 flagged above); `cargo clippy --workspace --all-targets` and `cargo fmt --check` clean.
  The CLI validates Turtle-array data (`rudof shex-validate -s colour.shex -m map.sm data.ttl`
  passes `:red :rgb (255 0 0)`, and reports `(255 192)` by its length and `(0 0 256)` by its third
  element).

## Decisions

- **The subject type.** As in oxigraph-array, a triple's subject is `oxrdf::Subject` (IRI, blank
  node or sequence), and every graph type rudof implements over Oxigraph uses it.
  `IriOrBlankNode` keeps its meaning for triple-term subjects and shape labels.
- **Lists are sequences, and only sequences.** Following the fork formula, the code that read and
  wrote `rdf:first`/`rdf:rest` ladders now reads and writes sequences, and a ladder is no longer a
  list anywhere in rudof: in ShExR, in SHACL shapes graphs, in test manifests. This keeps every
  Turtle-array shapes graph working, and is the reading SHACL-array would most likely take; it is
  noted for the SHACL-array discussion in rdf-array.md.
- **A Collection is a connective.** The engine proves a Collection's elements on demand, as it
  proves the components of `AND`/`OR`/`NOT`, rather than scheduling them as dependencies: the
  elements are not neighbours of the node. As a triple constraint's value, a Collection gets an
  index of its own (as `NOT` does), so the value is a reference to it.
- **Bounds.** `[m]` is exactly m; `[m,]` and `[m, *]` are unbounded, as in shex.js-array; a minimum
  above the maximum (`((x))[3, 1]`) is rejected, by the ShExC parser and when a ShExJ or ShExR
  schema is compiled, as jena-array does (shex.js-array accepts it; rdf-array.md lists it as
  open).
- **Node constraints on sequences.** A sequence satisfies no node kind, datatype or pattern, and
  matches `.`, as in shex.js-array.

## Not done

- A sequence as a shape map's focus node, or in a value set (`ObjectValue` refuses it).
- A sequence as the subject of a triple term in rudof's `Object` (it is an error), matching
  oxigraph-array, whose Turtle does not parse one either.
- `shapes_converter` (ShEx to HTML, UML, SPARQL, rdf-config) and `shapes_comparator` report a
  Collection as not implemented; the visualizer draws a sequence as one labelled node.
- SHACL-array itself: the SHACL suites run with lists read as sequences, which is not yet a
  specified reading.
- The Python (`pyrudof`) and WASM bindings build but were not tested with sequences.
