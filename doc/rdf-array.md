# rudof-array: rudof with sequences as terms

rdf-array is RDF whose data model has one more kind of term, the **sequence**: `(1 2 3)` in Turtle
is one term holding its elements in order, not a ladder of `rdf:first`/`rdf:rest` blank nodes, and
`()` is the empty sequence, not `rdf:nil`. Triples that spell out `rdf:first`/`rdf:rest` stay
ordinary triples and `rdf:nil` an ordinary IRI; they are just no longer what `( … )` means. See
[rdf-array.md](https://rdf-array.github.io/rdf-array/docs/rdf-array.html) for the model and the proposed
SPARQL and ShEx syntax.

This fork of [rudof](https://github.com/rudof-project/rudof) is the ShEx-array implementation in
Rust. rudof reads, stores and queries RDF through the Oxigraph crates, so it takes the term, the
syntaxes and SPARQL-array from [oxigraph-array](https://github.com/rdf-array/oxigraph-array), and
adds the ShEx-array **LIST shape expression**, `LIST ( regex )` and `LIST card x`
([array-syntax.md](https://rdf-array.github.io/rdf-array/docs/array-syntax.html) §2 and §7), in ShExC, ShExJ
and ShExR, with its validation.

| crate | from | what it does here | version |
|---|---|---|---|
| `oxrdf`, `oxttl`, `oxrdfxml`, `oxjsonld`, `oxrdfio`, `oxsdatatypes`, `spargebra`, `sparopt`, `spareval`, `sparesults`, `oxigraph` | [oxigraph-array](https://github.com/rdf-array/oxigraph-array), branch `rdf-sequences` (`[patch.crates-io]` in `Cargo.toml`) | the sequence term (`oxrdf::Sequence`, `Term::Sequence`, and `oxrdf::Subject` = IRI, blank node or sequence as a triple's subject); every syntax reads and writes `( … )`; SPARQL-array; the store | oxigraph 0.5.13, oxrdf 0.3.5 (rudof 0.3.35 pins oxigraph 0.5.9, oxrdf 0.3.3) |
| `rudof_iri`, `rudof_rdf` | this fork | rudof's RDF abstraction: `Object::Sequence`, `TermKind::Sequence`, `Term::sequence_elements`/`Term::sequence`; lists read and written as sequences | rudof 0.3.35 |
| `shex_ast` | this fork | the LIST shape expression in the ShExJ AST, the ShExC grammar and printer, ShExR (reader and writer), and the IR | rudof 0.3.35 |
| `shex_validation` | this fork | validates a node against a LIST, with an ordered matcher of its own (`rbe` works on bags) | rudof 0.3.35 |
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
branch `main`), `shacl/tests/data-shapes` ([data-shapes-array](https://github.com/rdf-array/data-shapes-array),
the W3C SHACL suites read as SHACL-array, branch `rdf-sequences`) and `shacl/tests/data-shapes-contrib`
([data-shapes-contrib](https://github.com/rdf-array/data-shapes-contrib), branch `main`, run by
`shacl/tests/contrib/mod.rs`). `git submodule update --remote` moves them to their branches' heads.

## What a sequence does here

```shex
PREFIX : <http://a.example/>
PREFIX xsd: <http://www.w3.org/2001/XMLSchema#>

:Colour   { :rgb LIST {3} xsd:integer MININCLUSIVE 0 MAXINCLUSIVE 255 }
:Polygon  { :points LIST {3,} LIST {2} xsd:decimal }
:Playlist { :tracks LIST + @:Track }
:Track    { :title xsd:string }
:Anything { :seq LIST * . }
:L        LIST + IRI
:Entry    { :kv LIST ( xsd:string ; xsd:integer ) }
:Pairs    { :kv LIST ( ( xsd:string ; xsd:integer ) * ) }
:Tree     LIST ( xsd:string ; LIST ( @:Tree * ) )
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
- **ShExC.** `LIST` is a shape atom wherever one may stand (a declaration, a triple constraint's value,
  under `NOT`, `AND`, `OR`), but not after `EXTENDS`. `LIST ( regex )` joins members with `;` (ordered
  concatenation) and `|`, with groups and the cardinalities `*`, `+`, `?`, `{m}`, `{m,}`, `{m,*}`, `{m,n}`.
  Inside `LIST ( )` an opening `(` always starts a group, so a member may not begin with a parenthesis:
  `LIST ( ( @:A ) AND @:B )` is a syntax error, `LIST ( @:B AND ( @:A ) )` a member. `LIST card x` is
  `LIST ( x card )`, with x any `inlineShapeAtom`, parentheses allowed. A bare `LIST x` and `LIST ( )` are
  syntax errors, and so is the old `((x))[m, n]`; `((x))` with no bounds is now `x` in two pairs of
  parentheses. A cardinality after the list is the triple constraint's: `:p LIST + @:C {1,2}`. A minimum
  above the maximum is rejected. The compact printer writes `LIST card x` for one member with a
  cardinality, else `LIST ( regex )`, keeping the nesting, and `.` for the empty shape. A member that would
  begin with a parenthesis (it comes only from ShExJ or ShExR) has its operands rotated, and when all of
  them would, is written `. AND ( … )`.
- **ShExJ.** `{"type": "ListShape", "expression": listExpr}`, where listExpr is
  `{"type": "Member", "shapeExpr": …, "min": m, "max": n}`, or `ListEachOf` or `ListOneOf` with
  `"expressions": [ … ]`, `min` and `max`. Absent `min` and `max` are 1, and `max` -1 is unbounded, as on
  triple expressions. `( x {2} ) {3}` is a `ListEachOf` of one, `{3}`, around the `Member`, `{2}`.
- **ShExR.** `[ a sx:ListShape ; sx:expression [ a sx:Member ; sx:shapeExpr … ; sx:min 1 ; sx:max -1 ] ]`,
  and `sx:ListEachOf`, `sx:ListOneOf` with `sx:expressions ( … )`, read and written. The
  schema's lists (`sx:shapes`, `sx:shapeExprs`, `sx:expressions`, `sx:values`, …) are sequences:
  the reader takes `sx:shapes ( … )` as the one sequence term it now is, and the writer
  (`ShExRBuilder`) writes one sequence instead of a ladder.
- **Validation.** A node conforms to a `LIST` when it is a sequence whose members, in order, match the
  regular expression; each member is a focus node of its own, like a triple constraint's value.
  `shex_validation/src/list_matcher.rs` runs the expression over the sequence as a set of positions, so a
  member that may belong to more than one part keeps both readings (`LIST ( . * ; xsd:integer )` takes
  `( "a" 1 2 )`), a member is tested against a member shape at most once, and a nullable repetition
  such as `( x ? ) {3}` is accepted, whatever the size of its cardinality. A failure is a
  `ListShapeFailure` holding `SequenceExpected` (the node is not a sequence), `ListMemberFailure` (the 1-based
  index of the first member that passed none of the shapes tried on it, and their errors) or
  `ListLengthViolation` (no member failed; with the length and the expression's length bounds). A pass is a
  `Reason::ListShapeResults` holding the reasons of one way of matching, member by member, in order. A
  sequence satisfies no `nodeKind` (`IRI`, `BNODE`, `LITERAL`, `NONLITERAL`), no datatype and no string
  facet, but does match `.`, as in shex.js-array.
- **Recursion.** `:Tree LIST ( xsd:string ; LIST ( @:Tree * ) )` is not a circular reference, because a
  member is another node: `direct_references` does not follow a list, so neither `extends_cycles` nor `dep`
  takes a member for a reference of the node itself. A node reaching itself through a list and a triple
  constraint is assumed to hold while it is proved, as any recursive reference is. Negation through a
  list does count for stratification: `:S { :p LIST ( NOT @:S * ) }` has a negative cycle.
- **Lists elsewhere.** rudof's generic list reader (`parse_sequence`, used by `ListParser` and
  `.list()`) and list writer (`BuildRDF::add_rdf_list`) read and write sequences, so SHACL shapes
  graphs written in Turtle-array work as before: `sh:path ( :p :q )` is a sequence path, `sh:in`,
  `sh:or`, `sh:languageIn`, `sh:ignoredProperties` are sequences, and SHACL 1.2's list node
  expression is a sequence (`()` the empty list). The SHACL test manifests' `mf:entries ( … )`
  are read as sequences too. `NilParser` accepts `()`.

## What changed

- `Cargo.toml`, `Cargo.lock`: `[patch.crates-io]` onto oxigraph-array. Cargo resolved `sha1` to
  0.10 for both its users after the patch, and spareval needs 0.11; the lock gives spareval
  `sha1 0.11.0` as upstream's did, by hand ([rdf-array#14](https://github.com/rdf-array/rdf-array/issues/14)).
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
  labelled with its spelling), `vocabs/shexr.rs` (`sx:ListShape`, `sx:Member`, `sx:ListEachOf`, `sx:ListOneOf`).
- `shex_ast`: `ast/shape_expr.rs` (`ShapeExpr::ListShape`, `ListExpr`), `compact/shex_grammar.rs`
  (`list_shape`, `list_one_of`, `list_each_of`, `list_unary`, `list_member*`), `compact/shex_parser_error.rs`,
  `compact/shex_compact_printer.rs`, `compact/shex_parser.rs`, `ir/shape_expr.rs`
  (`ShapeExpr::ListShape`, `ListExpr`), `ir/ast2ir.rs`, `ir/schema_ir.rs`,
  `ir/schema_ir_error.rs`, `ir/shape_label.rs`, `node.rs`, `ast/cond_kind.rs`,
  `shexr/shexr_parser.rs`, `shexr/shexr_builder/mod.rs`, `shexr/shexr_error.rs`, `materialize/materializer.rs`.
- `shex_validation`: `list_matcher.rs` (`check_list_shape`), `engine.rs` (a LIST is checked like a connective),
  `reason.rs`, `validator_error.rs`.
- `shacl`: `rdf/parsers/node_expr/{expr,mod}.rs` (the ladder-detecting `list.rs` is gone),
  `validator/constraints/mod.rs` (a sequence in SPARQL is `( e1 e2 )`),
  `validator/store/cached.rs`, `tests/common/manifest/*.rs`.
- `sparql_service`, `rudof_lib` (`pg_db`), `rudof_generate`, `shapes_converter`,
  `shapes_comparator`: the new subject type and term variant; the converters (to HTML, UML,
  SPARQL, CoShaMo) report a LIST as not implemented.
- `shex_testsuite`: `ManifestShExSyntaxMode::ShExR` (`-s shexr` reads the `.ttl` beside each
  `.shex`), `tests/rdf_array_contrib.rs`, the submodules, `config.toml`, the baselines;
  `tests/validation_regression.rs` runs on a 32 MiB thread (see Suite drift below).

## Tests

- **Added.** `shex_ast`: `test_serde_list_shape`, `list_expr_wraps_a_second_cardinality` (ShExJ),
  `test_list_shape_expressions` (ShExC, including the rejected forms and the old syntax),
  `lists_are_printed_and_roundtrip`, `list_members_never_begin_with_a_parenthesis` (the printer),
  `contrib_schemas_round_trip_between_formats` (every shexTest-contrib schema through
  ShExC → ShExJ, ShExC → ShExR, ShExC → ShExC, ShExJ → ShExC and ShExR → ShExC).
  `shex_validation/tests/list_shape.rs`: order, backtracking, cardinalities, nullable repetitions, members
  of every kind, recursion through a list, stratification, the failures reported.
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
  `LIST`s pass. In ShExR mode the 34 further failures are entries with no ShExR form: the
  `.ttl` beside the `.shex` is missing (`1dot-relative`, `extends-abstract-multi-empty`,
  `extends-closed-diamond`, `extends-closed-3diamond-split`) or is the test's data, not a schema
  (`nPlus1`, `PstarT`, `PTstar`, `P2T2`, `false-lead-excluding-value-shape`, `skipped`, …)
  ([rdf-array#16](https://github.com/rdf-array/rdf-array/issues/16)).
  `shex_ast`'s format round trip reads all 484 schemas' `.ttl` forms, with their lists as
  sequences, and passes.
- **Suite drift, not sequences** ([rdf-array#11](https://github.com/rdf-array/rdf-array/issues/11)). shexTest-array is 35 upstream commits
  newer than the shexTest rudof 0.3.35 pins, and vanilla 0.3.35 fails the same entries on upstream's newer suite:
  `8dot5a2b1c_fail-a5b1` and `8dot5a2b1c_fail-a6b2c1` overflow the stack and abort the whole run
  (excluded in `config.toml`); `8dot5a2b1c_pass-a5b2c1` recurses to a depth that varies from run to run, and
  overflowed a test thread's 2 MiB on the commit before LIST too, so `validation_regression` has a 32 MiB thread; `1dotShapeCode1fail_abort`, `open2OnecardStarclosecardStar_fail-aI1`,
  `open2OnedotcloseCode1fail_abort`, `open3EachdotcloseCode1fail_abort` fail
  (`tests/baseline_failing.txt`); `1card1` (added by shexTest 754575c) loses its `{1}` in the
  printer round trip (`tests/baseline_pretty_print_failing.txt`, and skipped in
  `format_roundtrip.rs`).
- **shexTest-contrib** (branch `main`): validation 56 of 56 in each of ShExC, ShExJ and ShExR;
  schemas 22 of 22; negative syntax 6 of 6.
- **SHACL suites** (2026-10-26, data-shapes-array): `core/path/path-strange-001` (re-recorded: the
  ladder is ordinary triples and the path is `^ex:p`) and `shacl12/node-expr/list` (re-recorded: `()`
  outputs nothing) now pass. Flagged (`#[ignore = "rdf-array: …"]`): `path-strange-002` (rejected in the
  fork; rudof-array also panics on its `sh:inversePath ( ex:p )`) and `core/complex/shacl-shacl` (the
  fork's shacl-shacl checks lists with `sh:memberShape` and `sh:minListLength`, which rudof does not have
  yet). The other SHACL tests, the sequence paths and `sh:in` among them, pass with their lists read as
  sequences.
- **data-shapes-contrib**: 6 of 21 pass; 15 ignored with the reason: the 11 that need SHACL 1.2's list
  constraints, 2 that expect `sht:Failure` (the runner reads no failure expectations; the contrib build
  checks that rudof-array refuses them), `empty-path-001` (rudof-array accepts `sh:path ()`) and
  `seq-in-or-001` (rudof-array reports the results of the shapes inside `sh:or` as the focus node's).
  The divergences are [rdf-array#17](https://github.com/rdf-array/rdf-array/issues/17).
- `cargo test -p shacl --test shacl_testsuite`: 141 passed, 0 failed, 40 ignored.
- **Numbers.** `cargo test --workspace`: 1475 passed, 0 failed, 43 ignored (the SHACL suites' 40, with
  the flagged and contrib ones above among them, and 3 others); `cargo clippy --workspace --all-targets` and
  `cargo fmt --check` clean.
  The CLI validates Turtle-array data (`rudof shex-validate -s colour.shex -m map.sm data.ttl`
  passes `:red :rgb (255 0 0)`, and reports `(255 192)` by its length and `(0 0 256)` by its third
  member).

## rdf-undef

Branch `rdf-undef`, from `rdf-sequences`: the empty slot of [rdf-undef.md](https://rdf-array.github.io/rdf-array/docs/rdf-undef.html),
`( 1 undef 3 )`. The term, the syntaxes and SPARQL come from oxigraph-array's `rdf-undef`; `[patch.crates-io]` follows that branch
and `Cargo.lock` holds its commit, edited by hand.

- **Term.** `Object::Sequence` holds `Vec<Option<Object>>` and `Term::sequence_elements` gives `Option`s, `None` at a slot; shown as
  `undef`, and a slot orders before every term. `Object` has no new variant, so a slot cannot be a subject or an object.
- **ShEx.** A slot satisfies no member constraint, `.` and `NOT` included: `( 1 undef 3 )` fails `LIST {3} xsd:integer` and `LIST * .`,
  with `ValidatorError::ListSlotFailure` naming the position. `undef` in place of a member matches a slot and nothing else:
  `LIST ( ( xsd:integer | undef ) * )`, `LIST * undef`. It is `ListExpr::Undef` in the AST and the IR, `{ "type": "Undef" }` in ShExJ and
  `sx:Undef` in ShExR, and it takes no `AND`, `OR` or `NOT`. A slot counts in the length, and a sequence holding one still satisfies
  `.` as a whole value.
- **SHACL.** Data with slots parses and validates. rudof has no `sh:memberShape` (SHACL 1.2's list components are still to do), so the
  rule that a slot conforms to no member shape has nothing to act on yet. A list of a shapes graph may not hold one: `sh:in ( 1 undef )`,
  `sh:or`, a sequence or alternative path, `sh:ignoredProperties` and a list node expression are `RDFError::SlotInListError`.
- **Tests.** The `shexTest-contrib` and `data-shapes-contrib` submodules are on their `rdf-undef` branches. `shacl/tests/contrib`: the `undef-*`
  tests that need list constraints or `sht:Failure` are flagged, as those without slots are.
  `cargo test --workspace`: 1482 passed, 0 failed, 53 ignored (1475, 0, 43 on `rdf-sequences`); clippy and fmt clean.
- **Not done, from `rdf-sequences`:** `sh:in` or `sh:hasValue` naming a sequence as a value, with or without a slot, reaches `todo!()` in
  `shacl/src/rdf/parsers/utils.rs`.

## Decisions

- **The subject type.** As in oxigraph-array, a triple's subject is `oxrdf::Subject` (IRI, blank
  node or sequence), and every graph type rudof implements over Oxigraph uses it.
  `IriOrBlankNode` keeps its meaning for triple-term subjects and shape labels.
- **Lists are sequences, and only sequences.** Following the fork formula, the code that read and
  wrote `rdf:first`/`rdf:rest` ladders now reads and writes sequences, and a ladder is no longer a
  list anywhere in rudof: in ShExR, in SHACL shapes graphs, in test manifests. This keeps every
  Turtle-array shapes graph working, and is the reading SHACL-array would most likely take; it is
  raised for SHACL-array as [rdf-array#9](https://github.com/rdf-array/rdf-array/issues/9).
- **A LIST is a connective.** The engine proves a list's members on demand, as it proves the components
  of `AND`/`OR`/`NOT`, rather than scheduling them as dependencies: the members are not neighbours of
  the node. As a triple constraint's value, a LIST gets an index of its own (as `NOT` does), so the value is
  a reference to it, and each member's shape expression has an index too.
- **An ordered matcher.** A set of positions, computed by repetition to a fixpoint (as PyShEx-array does)
  rather than an NFA (as shex.js-array does). A large cardinality costs nothing, since a repetition stops
  when the positions run out or stop changing, and the positions keep one way of reaching each, for the
  results.
- **Cardinalities.** A minimum above the maximum (`LIST {3,1} x`) is rejected, by the ShExC parser and when
  a ShExJ or ShExR schema is compiled, as jena-array does (shex.js-array accepts it;
  [rdf-array#6](https://github.com/rdf-array/rdf-array/issues/6)). A failure names the first failing member
  only, where shex.js-array names every failing member when the expression is one `Member`.
- **A member that would begin with a parenthesis.** Rotated, or written `. AND ( … )`, as jena-array does;
  shex.js-array rotates or refuses. Read back, such a member means the same but is not the same AST.
- **Node constraints on sequences.** A sequence satisfies no node kind, datatype or pattern, and
  matches `.`, as in shex.js-array.

## Not done

- A sequence as a shape map's focus node, or in a value set (`ObjectValue` refuses it):
  ShEx-array does not define either yet ([rdf-array#10](https://github.com/rdf-array/rdf-array/issues/10)).
- A sequence as the subject of a triple term in rudof's `Object` (it is an error), matching
  oxigraph-array, whose Turtle does not parse one either ([rdf-array#5](https://github.com/rdf-array/rdf-array/issues/5)).
- `shapes_converter` (ShEx to HTML, UML, SPARQL, rdf-config) and `shapes_comparator` report a
  LIST as not implemented ([rdf-array#12](https://github.com/rdf-array/rdf-array/issues/12)); the visualizer draws a sequence as one
  labelled node, and `shex-validate` spells a sequence's literals with full datatype IRIs
  ([rdf-array#15](https://github.com/rdf-array/rdf-array/issues/15)).
- SHACL-array itself: the SHACL suites run with lists read as sequences, which is not yet a
  specified reading.
- The Python (`pyrudof`) and WASM bindings build but were not tested with sequences
  ([rdf-array#13](https://github.com/rdf-array/rdf-array/issues/13)).

Issues for this fork live in [rdf-array/rdf-array](https://github.com/rdf-array/rdf-array/issues),
as for every rdf-array fork.
