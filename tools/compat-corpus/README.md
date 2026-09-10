# RSS compatibility corpus runner

This is an internal discovery tool. It does not add external fixtures to the
crate's test suite and it does not create GitHub issues automatically.

The first stage runs the RSS parser fixtures from `mmcdole/gofeed` at a pinned
revision. It checks fields represented by both libraries, then serializes and
re-parses every successfully parsed channel. Differences are candidates for
manual triage, not confirmed defects.

The initial field comparison covers core channel and item fields, categories,
image, cloud, text input, enclosure, GUID, and source. It deliberately does not
yet compare parsed dates, multiple-link normalization, Dublin Core, iTunes,
generic extensions, or gofeed's custom field map. A fixture marked `pass`
therefore means “no difference in the currently compared contract,” not full
semantic equivalence.

Run it from the repository root:

```console
./tools/compat-corpus/run-gofeed.sh
./tools/compat-corpus/run-feedparser.sh
./tools/compat-corpus/run-feedvalidator.sh
```

The script stores the external checkout below `target/compat-corpus/`, which is
ignored by Git. It writes the reproducible inventory and triage checkpoint to
`progress/gofeed.json`. The report contains every fixture, not only findings.
Candidate records have a manually maintained `triage` object whose status is
preserved across reruns at the same source revision. Use `pending`, `excluded`,
`not-a-bug`, or `confirmed`, and record the reason and fork issue when relevant.
Confirmed behavior-level findings are also indexed in `progress/confirmed.json`;
this is the stable bridge between several source fixtures, the minimized ignored
tests in `tests/compat_confirmed.rs`, and the fork-only issues. One issue may
cover several fixtures when they demonstrate the same underlying behavior.
The report also records the tested crate revision and all fixture paths, so a
new contributor can reproduce the inventory and resume at the first `pending`
candidate without reconstructing prior local state.

A successful experiment can still report candidate differences; the command
fails only when the runner or corpus setup itself fails. If the pinned corpus
revision changes, the runner refuses to reuse old manual triage state.

The feedparser and Feed Validator stages are structural screens. They retain
the source suite's embedded description and expectation, check whether this
crate can parse and round-trip the input, and record this crate's validation
result. They do not claim that differently named validator diagnostics are
equivalent. Feed Validator `SAXError` expectations are compared at the XML
acceptance boundary; other diagnostics remain evidence for later manual
standards review.

Before recording a candidate as a defect:

1. Confirm that the fixture uses a supported RSS version and feature.
2. Check the expected behavior against the RSS/XML specification.
3. Distinguish malformed-input tolerance from correctness.
4. Minimize the fixture and reproduce it against the current baseline.
5. Prove that the proposed Rust regression test fails for the target behavior.

Run the confirmed red tests explicitly with:

```console
cargo test --all-features --test compat_confirmed -- --ignored
```

They are ignored during the normal suite because each assertion describes the
desired behavior and intentionally remains red until its linked issue is fixed.
