# jisyo tasks

Do one card per run. `src/model.rs`, `Cargo.toml`, `Cargo.lock`, `Makefile`, `.gitignore`, and `data/euc-jis-2004-std.txt` are never on a card's file list. The dependencies are already in `Cargo.toml`: `lexopt` and, for tests only, `tempfile`.

For each module it adds, a card appends exactly one line `pub mod <name>;` to `src/lib.rs`, after the existing `pub mod` lines, and changes nothing else in that file. J5 adds two modules and therefore appends two lines, `pub mod dict;` followed by `pub mod ime;`.

Each named test below is a separate test function. Do not turn a table of independent cases into one loop test. A test checks one behavior through a public function, a synthetic-file round trip, or the generator CLI. Expected values are literals or independent standard-library encodings, never another jisyo function. A round-trip test compares the final bytes with the original synthetic bytes; it does not derive expected bytes from the implementation.

The gate is `make check`. It must not write under `.git`. Tests that need git state use a temporary index and object directory as card-flow requires. A `Must fail without` check counts only when the named test runs the code under test and fails because the guard is absent, not because compilation, import, or fixture setup fails. Commits and pull requests belong to the lead and skk-manager, not the worker.

SPEC.md "Layers" says which module may use which. A core module (`entry`, `dict`, `record`, `merge`, `diff`) never uses `encoding`, `text`, `ime`, `sync`, `store`, the file system, or the clock.

Tests use the public API (`jisyo::...`) unless a card says that a test is a unit test in the module's `#[cfg(test)] mod tests`. Byte strings in the tables below are hex, for example `A4 A2`. Strings use Rust escape syntax, for example `"\u{309A}"`.

The store cards (log, merge, diff, sync, CLI) come after `skk-design` reviews the store part of `SPEC.md`.

## J1. EUC-JP table

Needs: nothing.

Spec: SPEC.md "Encodings", subsection "`data/euc-jp.txt`".

Files: `tools/gen-euc-jp.py`, `data/euc-jp.txt`, `tests/gen_euc_jp.rs`, `testdata/iconv-double.py`.

Write `tools/gen-euc-jp.py` exactly as the SPEC subsection says. Use only the Python standard library and the `iconv` program through `subprocess`. Generate the ASCII identity rows directly. Convert the non-ASCII candidates in batches of at most 256, using the framing rules below, never one process per code. Then run the script once to produce `data/euc-jp.txt`.

Rules:

- With no arguments, the script writes `data/euc-jp.txt`, relative to the repository root, which is the parent of the script's own directory (`Path(__file__).resolve().parent.parent`). It prints one line, `wrote <n> mappings`, where `<n>` counts the mapping lines (no header lines).
- With `--check`, it writes nothing. It exits 0 with empty stdout and stderr when the file equals the text it would write. Otherwise it prints `data/euc-jp.txt is out of date\n` to stderr, leaves stdout empty, and exits 1.
- Any other argument: print `usage: gen-euc-jp.py [--check]` to stderr and exit 2.
- For decoding, send each candidate followed by one literal LF byte to `iconv -c -f EUC-JP -t UTF-8`. Decode each returned piece as UTF-8, then apply the single-character eligibility filter exactly once, when selecting records for the reverse batch. It excludes empty and multi-character pieces. Send each eligible string followed by one LF through `iconv -c -f UTF-8 -t EUC-JP`, keeping its original candidate associated with it. This selection filter is the only explicit character-count check in the generator. Do not repeat it at keep time, in reverse conversion, in formatting, or elsewhere.
- In both directions, accept an iconv return code of 0 or 1 only when stderr is empty and the output framing is intact. Code 1 can report skipped invalid input even with `-c`. Any other return code, nonempty stderr, or broken framing means stop and report, without writing the table.
- Check framing before pairing any output with an input: stdout must end with LF, and splitting only on the literal LF byte and removing the final empty piece must give exactly as many pieces as inputs. Keep empty pieces. Do not use `splitlines()` or discard empty lines. A framing mismatch is a failure, not permission to shift the remaining outputs.
- For a record admitted by the eligibility filter, the keep decision checks only whether the reverse-converted bytes equal the original candidate bytes. Do not check the character count again here. Together, the earlier filter and this byte comparison implement the two SPEC conditions. Invalid UTF-8 is rejected while decoding the returned piece; empty or multi-character pieces are excluded only by the earlier filter.
- Code points are written with at least 4 uppercase hex digits (`U+3042`, `U+20089`).
- If the mapping count, file line count, or any expected value below differs on this glibc version, report the actual value and stop. Do not change the expected values or the SPEC to fit the host.

Real-iconv checks, which the worker runs and pastes into the report. These host checks enforce the expected real-table counts; do not hardcode those counts in the generator CLI, whose synthetic-double tests intentionally produce a smaller table:

| Command | Expected |
|---|---|
| `python3 tools/gen-euc-jp.py` | prints `wrote 13137 mappings` |
| `python3 tools/gen-euc-jp.py --check` | no output, exit 0 |
| `wc -l < data/euc-jp.txt` | `13139` |
| `grep -c $'^0x8E' data/euc-jp.txt` | `63` |
| `grep -c $'^0x41\t' data/euc-jp.txt` | `1`; the row is `0x41\tU+0041` |
| `grep -c $'^0x8EB1\t' data/euc-jp.txt` | `1`; the row is `0x8EB1\tU+FF71` |
| `grep -c $'^0xA1B1\t' data/euc-jp.txt` | `1`; the row is `0xA1B1\tU+FFE3` |
| `grep -c $'^0xA1BD\t' data/euc-jp.txt` | `1`; the row is `0xA1BD\tU+2015` |
| `grep -c $'^0xA4A2\t' data/euc-jp.txt` | `1`; the row is `0xA4A2\tU+3042` |
| `grep -c $'^0x8FA2B7\t' data/euc-jp.txt` | `1`, exit 0; the row is `0x8FA2B7\tU+FF5E` |
| `grep -c $'^0x8FB0A1\t' data/euc-jp.txt` | `1`; the row is `0x8FB0A1\tU+4E02` |

The seven sample rows above use `\t` to mean one literal tab. Check their full contents and their positions in ascending unsigned-integer code order, not lexicographic hex-string order. The sample order is `0x41 < 0x8EB1 < 0xA1B1 < 0xA1BD < 0xA4A2 < 0x8FA2B7 < 0x8FB0A1`, which is also the current table order. For example, the three-byte integer 0x8FA2B7 is greater than the two-byte integer 0xA4A2 even though its hex text begins with 8. The expected total remains 13,137 mappings and 13,139 file lines. The U+FF5E row is included in the expected 6,067 three-byte mappings. Do not increment any total just because the erroneous negative check was corrected. The full generator run must confirm those totals.

### Generator CLI tests

Put these Rust integration tests in `tests/gen_euc_jp.rs`. They invoke `python3 -I` on a copy of the generator in a `tempfile` directory with `tools/` and `data/` children. Set the child process's working directory to a different temporary directory. Never regenerate the checkout's real table during the gate. These developer-tool tests are Unix-only with `#[cfg(unix)]`, because this generator requires the glibc iconv development environment; this does not restrict the Rust library.

Use `testdata/iconv-double.py` as the child process's `iconv`, copied to a temporary executable named `iconv` at the front of that child's PATH. The double uses the Python standard library and accepts `-c -f EUC-JP -t UTF-8` and the reverse options. It processes only literal-LF-separated records in memory. Its default synthetic forward mappings are `A4 A2` to U+3042, `FE FE` to U+20089, `8F A2 B7` to U+FF5E, `A4 A3` to the two characters `xy`, and `8E B1` to U+FF71. Other candidate codes give an empty record. Its reverse mappings are U+3042 to `A4 A2`, U+20089 to `FE FE`, U+FF5E to `8F A2 B7`, the complete two-character record `xy` to `A4 A3`, and U+FF71 to `8E B2`, deliberately different from the forward code. Mapping `xy` back to A4A3 is intentional. Reverse-byte equality must not mask removal of the eligibility filter. The filter's sole location and the keep decision are defined in Rules above; neither the double nor later generator stages may silently discard `xy` for its length after that filter is removed. The retained three-byte code is intentional too. Numeric order puts 0x8FA2B7 after 0xFEFE, while hex-text order puts it before 0xA4A2. It preserves every LF, returns 1 with empty stderr when it skipped anything, otherwise 0. It never opens dictionary files. Only the double reads `JISYO_TEST_ICONV_FAULT`, with values `exit2`, `stderr`, `missing-lf`, or `extra-line`; the production generator must not read this variable. Each value changes only that one property in the forward result: exit 2, stderr `synthetic failure\n`, removal of the final LF, or one extra LF respectively.

| Test | Assert through the generator CLI |
|---|---|
| `gen_writes_at_script_root` | With the default double, generation exits 0, prints `wrote 131 mappings\n`, and creates `data/euc-jp.txt` beside the copied tools directory, not under the process working directory |
| `gen_header` | The generated file starts with exactly the two header lines in the SPEC |
| `gen_ascii_identity` | All 128 ASCII rows are exactly `0x00\tU+0000` through `0x7F\tU+007F` |
| `gen_row_order` | Read only the code column in the file's existing row order and parse each code as an integer. Compare that unsorted vector with the literal ordered sequence 0x00 through 0x7F, then 0xA4A2, 0xFEFE, 0x8FA2B7. Never sort the actual vector in the test. The three-byte code must follow both two-byte codes, so sorting output rows as hex text must make this test fail. This test does not inspect code-point spelling |
| `gen_codepoint_hex_width` | Looking up rows by their code, independently of row order, finds code-point fields `U+0000` for 0x00, `U+3042` for 0xA4A2, and `U+20089` for 0xFEFE |
| `gen_keeps_empty_record_positions` | The empty outputs preceding A4A2 do not shift its association; the output contains exactly `0xA4A2\tU+3042` for that code |
| `gen_rejects_multi_character` | Generation exits 0 and no row for A4A3 appears, even though the double maps its complete decoded record `xy` back to A4A3; with the single-character filter removed, this test must fail rather than rely on reverse-byte equality to reject it |
| `gen_rejects_reverse_mismatch` | No row for 8EB1 appears |
| `gen_check_matches` | Seed the output with independently assembled expected bytes: the two literal SPEC headers, 128 ASCII identity rows, `0xA4A2\tU+3042\n`, `0xFEFE\tU+20089\n`, then `0x8FA2B7\tU+FF5E\n`. Do not run the generator to make this fixture. `--check` exits 0 with empty stdout and stderr and leaves its bytes and modification time unchanged |
| `gen_check_missing` | With no output file, `--check` exits 1, stdout is empty, stderr is `data/euc-jp.txt is out of date\n`, and no file is created |
| `gen_check_differs` | With an output containing only `sentinel\n`, `--check` exits 1, stdout is empty, stderr is `data/euc-jp.txt is out of date\n`, and the sentinel is preserved |
| `gen_bad_argument` | `--unknown` exits 2 and stderr is `usage: gen-euc-jp.py [--check]\n`; no output file is created |
| `gen_rejects_iconv_exit2` | With `exit2` and a pre-existing sentinel output, generation exits 1 and leaves the sentinel unchanged |
| `gen_rejects_iconv_stderr` | With `stderr` and a sentinel output, generation exits 1 and leaves the sentinel unchanged |
| `gen_rejects_missing_lf` | With `missing-lf` and a sentinel output, generation exits 1 and leaves the sentinel unchanged |
| `gen_rejects_extra_line` | With `extra-line` and a sentinel output, generation exits 1 and leaves the sentinel unchanged |

A conversion failure prints `error: iconv\n` to stderr and exits 1, without writing a table or printing the double's stderr. Each failure test asserts that stderr, empty stdout, and the preserved sentinel in addition to the exit code. Every successful-generation test invokes the CLI in the test body and first asserts exit 0, before opening the result file. Fixture setup only copies the script and double and prepares directories or independently specified files; it must not run the generator to make expected data. No reverse candidates means skip that empty batch, not a framing failure.

Must fail without: the sole single-character eligibility filter that selects reverse-batch inputs → `gen_rejects_multi_character`.
Must fail without: reverse-byte equality → `gen_rejects_reverse_mismatch`.
Must fail without: iconv status validation → `gen_rejects_iconv_exit2`.
Must fail without: iconv stderr validation → `gen_rejects_iconv_stderr`.
Must fail without: the output-framing guard, both the final-LF and record-count checks → `gen_rejects_missing_lf`, `gen_rejects_extra_line`.

Done: the real-iconv checks above hold, every named CLI test runs under `make check` on Unix, and `make check` exits 0.

## J2. Codec

Needs: J1.

Spec: SPEC.md "Encodings", "Errors".

Files: `src/encoding.rs`, `src/lib.rs`.

Implement `Codec` exactly as the SPEC says, with the tables from `include_str!("../data/euc-jis-2004-std.txt")` and `include_str!("../data/euc-jp.txt")`. Use plain `HashMap`s built in `Codec::new`, with no `static` maps. Errors are `Error::new(ErrorCode::Decode).offset(..)` and `Error::new(ErrorCode::Encode).offset(..)`.

Tests are unit tests in `src/encoding.rs` under `#[cfg(test)] mod tests` and call only the public Codec methods. "All codes" means every byte string of these forms: the single bytes 0x00..=0x7F; `[0x8E, x]` for x in 0xA1..=0xFE; `[a, b]` for a, b in 0xA1..=0xFE; `[0x8F, a, b]` for a, b in 0xA1..=0xFE. A code "decodes" when `decode` of exactly that byte string is `Ok`. The raw x0213 count is 11,456, but J2's accepted EucJis2004 count is 11,424 after skipping the 32 high single-byte controls. EucJp accepts `8F A2 B7` as U+FF5E; EucJis2004 does not.

Each comma-separated name below is its own function for the corresponding encoding or code family, never one combined test. An exhaustive loop within a test is allowed only to check that test's one property over its code family.

| Test names | Assert, in the same order |
|---|---|
| `jis2004_ascii_count`, `jis2004_kana_count`, `jis2004_two_byte_count`, `jis2004_three_byte_count` | The accepted code counts for the four code families are 128, 63, 8797, 2436 respectively |
| `eucjp_ascii_count`, `eucjp_kana_count`, `eucjp_two_byte_count`, `eucjp_three_byte_count` | The counts are 128, 63, 6879, 6067 respectively |
| `jis2004_pair_count` | Exactly 25 accepted codes give two chars |
| `jis2004_decoded_lengths` | Every accepted code gives one or two chars |
| `eucjp_decoded_lengths` | Every accepted code gives exactly one char |
| `jis2004_every_code_round_trips`, `eucjp_every_code_round_trips` | For that encoding and each accepted code `c`, `encode(decode(c))` is the original input bytes `c` |
| `jis2004_decodes_are_distinct`, `eucjp_decodes_are_distinct` | For that encoding, accepted single-code decoded strings are pairwise distinct |
| `jis2004_ascii_identity`, `eucjp_ascii_identity` | Decoding the bytes 0x00 through 0x7F as one slice gives U+0000 through U+007F |
| `jis2004_pair_greedy_ipa` | Encoding the literal `"\u{E6}\u{300}"` gives `AB C4`, not two single codes |
| `jis2004_single_before_end` | Encoding `"か"` gives `A4 AB` |
| `jis2004_decode_separate_ipa` | Decoding `A9 DC AB DC` gives the literal `"\u{E6}\u{300}"` |
| `eucjp_encode_error`, `jis2004_encode_error` | Encoding `"a😀"` in EucJp gives `(Encode, offset 1)`; encoding `"あ😀"` in EucJis2004 gives `(Encode, offset 3)` |
| `eucjp_can_encode_false`, `jis2004_can_encode_false` | `can_encode("😀")` is false for that encoding |
| `eucjp_can_encode_true`, `jis2004_can_encode_true` | `can_encode("漢字")` is true for that encoding |
| `utf16le_encode`, `utf16le_decode` | Encoding `"a😀"` gives `61 00 3D D8 00 DE`; decoding those literal bytes gives `"a😀"` |
| `utf8_encode`, `utf8_decode` | Encoding `"あ"` gives `E3 81 82`; decoding those literal bytes gives `"あ"` |
| `encoding_getter_eucjp`, `encoding_getter_jis2004`, `encoding_getter_utf8`, `encoding_getter_utf16le` | `Codec::new(e).encoding()` is respectively EucJp, EucJis2004, Utf8, Utf16Le |
| `alias_decode_jis2004`, `alias_decode_eucjp` | Decoding literal `A1 BD` gives respectively `"\u{2014}"` and `"\u{2015}"` |

For each alias, write one encode-only test with the exact name `alias_<encoding>_<codepoint>`, where encoding is `jis2004` or `eucjp` and codepoint is lowercase hex without `U+`. The exact suffix lists are `jis2004_ffe3`, `jis2004_2015`, `jis2004_ffe5`, `jis2004_ffe0`, `jis2004_ffe1`, `jis2004_ffe2`, `eucjp_203e`, `eucjp_2014`, `eucjp_00a5`, `eucjp_2225`, `eucjp_ff0d`, `eucjp_ffe0`, `eucjp_ffe1`, `eucjp_ffe2`. Each test encodes the literal character and compares with that alias's literal bytes in SPEC.md. Do not derive its expected bytes by decoding or calling another jisyo function.

For every sample row below, write two functions named `decode_<case>` and `encode_<case>`. The first compares decoding the literal bytes with the literal string. The second compares encoding the literal string with the literal bytes.

| Case | Encoding | Bytes | String |
|---|---|---|---|
| `jis2004_hiragana` | `EucJis2004` | `A4 A2` | `"あ"` |
| `jis2004_wave_dash` | `EucJis2004` | `A1 C1` | `"\u{301C}"` |
| `jis2004_kana_pair` | `EucJis2004` | `A4 F7` | `"か\u{309A}"` |
| `jis2004_plane1_supplementary` | `EucJis2004` | `AE A2` | `"\u{2000B}"` |
| `jis2004_plane2_supplementary` | `EucJis2004` | `8F A1 A1` | `"\u{20089}"` |
| `jis2004_halfwidth` | `EucJis2004` | `8E B1` | `"\u{FF71}"` |
| `jis2004_kanji` | `EucJis2004` | `B4 C1 BB FA` | `"漢字"` |
| `eucjp_hiragana` | `EucJp` | `A4 A2` | `"あ"` |
| `eucjp_macron` | `EucJp` | `A1 B1` | `"\u{FFE3}"` |
| `eucjp_jis0212` | `EucJp` | `8F B0 A1` | `"\u{4E02}"` |
| `eucjp_fullwidth_tilde` | `EucJp` | `8F A2 B7` | `"\u{FF5E}"` |
| `eucjp_halfwidth` | `EucJp` | `8E B1` | `"\u{FF71}"` |
| `eucjp_kanji` | `EucJp` | `B4 C1 BB FA` | `"漢字"` |

Decode-error tests each compare the error's `(code, offset)` with `(Decode, Some(offset))`:

| Test | Encoding | Bytes | Offset |
|---|---|---|---|
| `decode_error_eucjp_invalid_lead` | `EucJp` | `41 FF` | 1 |
| `decode_error_jis2004_unmapped_three_byte` | `EucJis2004` | `8F A2 B7` | 0 |
| `decode_error_jis2004_truncated` | `EucJis2004` | `A4` | 0 |
| `decode_error_jis2004_high_control` | `EucJis2004` | `41 80 41` | 1 |
| `decode_error_jis2004_reserved` | `EucJis2004` | `41 A4 FC` | 1 |
| `decode_error_utf8_truncated` | `Utf8` | `61 C3` | 1 |
| `decode_error_utf16le_odd_length` | `Utf16Le` | `41 00 42` | 2 |
| `decode_error_utf16le_high_surrogate` | `Utf16Le` | `00 D8 41 00` | 0 |
| `decode_error_utf16le_low_surrogate` | `Utf16Le` | `41 00 00 DC` | 2 |

Done: `make check` exits 0.

## J3. Entry line

Needs: nothing.

Spec: SPEC.md "One entry line".

Files: `src/entry.rs`, `src/lib.rs`, `tests/layers.rs`.

Implement the four functions exactly as the SPEC says, including the final check that `format_entry` of the result equals the input.

Create `tests/layers.rs` as SPEC.md "Layout" says, with one test `core_modules_stay_pure` over the list `[("src/model.rs", include_str!("../src/model.rs")), ("src/entry.rs", include_str!("../src/entry.rs"))]`. The contract is Core too and is read, not edited. A failing assert names the file and the forbidden text.

Unit tests in `src/entry.rs` under `#[cfg(test)] mod tests` call only the four public functions. In the tables, `c(w)` is `Candidate { word: w, annotation: None }`, `c(w; a)` has `annotation: Some(a)`, and `b(k, [...])` is an `OkuriBlock`.

For each accepted case, write `parse_accept_<case>` and `format_entry_<case>` as separate test functions. The parse test compares with the literal Entry in that row. The format test starts from that literal Entry and compares with the literal Content, without deriving either side with `parse_entry`.

| Case | Content | Section | Midashi | Candidates | Blocks |
|---|---|---|---|---|---|
| `plain` | `かんじ /漢字/感じ/` | nasi | `かんじ` | `c(漢字), c(感じ)` | none |
| `annotations` | `かんじ /漢字;注釈/幹事;/` | nasi | `かんじ` | `c(漢字; 注釈), c(幹事; "")` | none |
| `blocks` | `おおk /大/多/[く/多/]/[き/大/]/` | ari | `おおk` | `c(大), c(多)` | `b(く, [c(多)]), b(き, [c(大)])` |
| `empty_block` | `おおk /[く/]/` | ari | `おおk` | none | `b(く, [])` |
| `empty_list` | `かんじ /` | nasi | `かんじ` | none | none |
| `nasi_brackets` | `かっこ /[/]/` | nasi | `かっこ` | `c([), c(])` | none |
| `literal_concat` | `きごう /(concat "a\057b")/` | nasi | `きごう` | `c((concat "a\057b"))`, with a literal backslash | none |
| `semicolons` | `a;b /x;y;z/` | nasi | `a;b` | `c(x; y;z)` | none |
| `slash_in_midashi` | `http:// /h/` | nasi | `http://` | `c(h)` | none |

Each rejection test calls `parse_entry(content, section)` and asserts `None`:

| Test | Content | Section | Why |
|---|---|---|---|
| `reject_no_space` | `かんじ` | nasi | no space |
| `reject_empty_midashi` | ` /漢字/` | nasi | empty midashi |
| `reject_comment_midashi` | `;x /a/` | nasi | midashi starts with `;` |
| `reject_no_initial_slash` | `かんじ 漢字/` | nasi | rest does not start with `/` |
| `reject_no_final_slash` | `かんじ /漢字` | nasi | no final `/` |
| `reject_only_empty_token` | `かんじ //` | nasi | empty token |
| `reject_trailing_empty_token` | `かんじ /漢字//` | nasi | empty token |
| `reject_double_space` | `かんじ  /漢字/` | nasi | two spaces |
| `reject_trailing_text` | `かんじ /漢字/ ` | nasi | text after the last `/` |
| `reject_empty_candidate_word` | `かんじ /;注/` | nasi | empty word |
| `reject_unclosed_block` | `おおk /[く/多/` | ari | missing `]` |
| `reject_candidate_after_block` | `おおk /[く/多/]/大/` | ari | candidate after a block |
| `reject_stray_close` | `おおk /]/` | ari | `]` outside a block |
| `reject_empty_okuri` | `おおk /[/多/]/` | ari | empty okuri |
| `reject_nested_block` | `おおk /[く/[多/]/` | ari | block candidate starts with `[` |
| `reject_close_in_okuri` | `おおk /[く]/多/]/` | ari | okuri `く]` contains `]`; the block has a candidate and a separate closing token |
| `reject_open_in_okuri` | `おおk /[く[/多/]/` | ari | okuri `く[` contains `[`; the block has a candidate and a separate closing token |

For each candidate case below, write separate `parse_candidate_<case>` and `format_candidate_<case>` tests. Use the literal Candidate as the expected parse result and the literal string as the expected formatted result. Do not obtain the format fixture by parsing.

| Case | Input and expected formatted string | Literal Candidate |
|---|---|---|
| `plain` | `"a"` | `c(a)` |
| `empty_annotation` | `"a;"` | `c(a; "")` |
| `first_semicolon` | `"a;b;c"` | `c(a; b;c)` |
| `empty_word` | `""` | `c("")` |

Must fail without: rejecting `]` inside block okuri → `reject_close_in_okuri`.
Must fail without: rejecting `[` inside block okuri → `reject_open_in_okuri`.

Done: `make check` exits 0.

## J4. Lossless file

Needs: J2, J3.

Spec: SPEC.md "One file", "Errors".

Files: `src/text.rs`, `src/lib.rs`, `tools/gen-testdata.py`, `testdata/basic.euc-jp.jisyo`, `testdata/basic.euc-jis-2004.jisyo`, `testdata/nocookie.euc-jis-2004.jisyo`, `testdata/basic.utf-8.jisyo`, `testdata/bom.utf-8.jisyo`, `testdata/basic.utf-16le.jisyo`, `testdata/edge.utf-8.jisyo`, `tests/round_trip.rs`.

Implement `detect`, `read`, `write`, and `line_content` exactly as the SPEC says. Keep the round-trip guard (step 3 of `read`) as its own function, `fn check_round_trip(codec: &Codec, body: &[u8], text: &str, bom_len: usize) -> Result<(), Error>`.

### Test data

`tools/gen-testdata.py` uses only the Python standard library. It writes the seven files with Python's own codecs (`euc_jp`, `euc_jis_2004`, `utf-8`, `utf-16-le`), so the expected bytes do not come from our Rust code. Its output directory is `Path(__file__).resolve().parent.parent / "testdata"`, independent of the working directory. With no arguments, `python3 tools/gen-testdata.py` creates the directory if needed, writes exactly the seven named files, and exits 0. Repeated runs write identical bytes. With `--check`, it writes nothing, compares all seven files with the expected bytes, and exits 0 if every file exists and matches, else 1. Any other argument prints `usage: gen-testdata.py [--check]` to stderr and exits 2. Run it once and leave the generated files in the worktree without committing. The lead commits them. Each file's text is given below, line by line. Every line ends with `\n` unless the file says otherwise.

`B` is these lines:

```
;; synthetic test dictionary
;; okuri-ari entries.
わたs /渡/
もt /持/[って/持/]/[た/持/]/
おおk /大/多/[く/多/]/[き/大/]/
;; okuri-nasi entries.
かんじ /漢字/幹事;組織の世話役/感じ/
から /〜/空/
あ /ｱ/亜/
きごう /(concat "a\057b")/[/]/
```

`〜` is U+301C, `ｱ` is U+FF71, and `\057` is a literal backslash followed by `057`.

| File | Text | Encoding |
|---|---|---|
| `basic.euc-jp.jisyo` | `;;; -*- coding: euc-jp -*-`, then `B`, then `こう /丂/` (U+4E02) | `euc_jp` |
| `basic.euc-jis-2004.jisyo` | `;;; -*- coding: euc-jis-2004 -*-`, then `B`, then `ぱ /か゚/𠀋/𠂉/` (U+304B U+309A, U+2000B, U+20089) | `euc_jis_2004` |
| `nocookie.euc-jis-2004.jisyo` | `B`, then `ぱ /か゚/𠀋/𠂉/` | `euc_jis_2004` |
| `basic.utf-8.jisyo` | `;;; -*- coding: utf-8 -*-`, then `B`, then `えもじ /😀/か゚/𠂉/丂/` | `utf-8` |
| `bom.utf-8.jisyo` | `B`, then `えもじ /😀/か゚/𠂉/丂/` | bytes `EF BB BF`, then `utf-8` |
| `basic.utf-16le.jisyo` | `B`, then `えもじ /😀/か゚/𠂉/丂/`, with every `\n` written as `\r\n` | bytes `FF FE`, then `utf-16-le` |
| `edge.utf-8.jisyo` | the lines below | `utf-8` |

`edge.utf-8.jisyo`, exactly, where `⏎` is `\n`, `␍⏎` is `\r\n`, and `␍` alone is a CR in the middle of the line:

```
;; edge cases⏎
junk before marker⏎
⏎
;; okuri-ari entries.␍⏎
おおk /[く/]/⏎
おおk /大/[く/多/]/大/⏎
;; okuri-nasi entries.⏎
かんじ /漢字/ ⏎
かんじ /漢␍字/⏎
;; comment inside section⏎
かっこ /[/]/
```

The last line has no newline. Line 8 ends with a space before its `\n`.

### Tests

Integration tests in `tests/round_trip.rs` read the files through `env!("CARGO_MANIFEST_DIR")`.

| Test | Assert |
|---|---|
| `round_trip_basic_euc_jp`, `round_trip_basic_euc_jis_2004`, `round_trip_nocookie_euc_jis_2004`, `round_trip_basic_utf_8`, `round_trip_bom_utf_8`, `round_trip_basic_utf_16le`, `round_trip_edge_utf_8` | for that file, `write(&read(&bytes, None)?)?` equals `bytes` |
| `detect_basic_euc_jp` | `detect` of that file gives `(EucJp, false)` |
| `detect_basic_euc_jis_2004` | Gives `(EucJis2004, false)` |
| `detect_nocookie_euc_jis_2004` | Gives `(EucJis2004, false)` |
| `detect_basic_utf_8` | Gives `(Utf8, false)` |
| `detect_bom_utf_8` | Gives `(Utf8, true)` |
| `detect_basic_utf_16le` | Gives `(Utf16Le, true)` |
| `detect_edge_utf_8` | Gives `(Utf8, false)` |
| `basic_eucjp_line_kinds` | Kinds are Comment, Comment, Marker(OkuriAri), Entry, Entry, Entry, Marker(OkuriNasi), Entry ×5 |
| `basic_eucjp_eols` | All line terminators are Lf |
| `basic_utf16le_line_kinds` | Kinds are Comment, Marker(OkuriAri), Entry ×3, Marker(OkuriNasi), Entry ×5 |
| `basic_utf16le_eols` | All line terminators are CrLf |
| `basic_okuri_blocks` | In basic.euc-jp, entry `もt` has main candidates `[持]` and blocks `[(って, [持]), (た, [持])]` |
| `basic_annotation` | In basic.euc-jp, the second candidate of `かんじ` is `幹事` with annotation `Some("組織の世話役")` |
| `basic_jis0212_candidate` | In basic.euc-jp, entry `こう` has candidates `["\u{4E02}"]` |
| `basic_jis2004_candidates` | In basic.euc-jis-2004, entry `ぱ` has candidates `["か\u{309A}", "\u{2000B}", "\u{20089}"]` |
| `basic_utf16le_supplementary` | In basic.utf-16le, the first candidate of `えもじ` is `"😀"` |
| `edge_comment` | Line 1 is Comment(";; edge cases") with Lf |
| `edge_before_marker` | Line 2 is Other("junk before marker") with Lf |
| `edge_blank` | Line 3 is Blank with Lf |
| `edge_crlf_marker` | Line 4 is Marker(OkuriAri) with CrLf |
| `edge_empty_block` | Line 5 is Entry `おおk`, no main candidates, blocks `[(く, [])]`, with Lf |
| `edge_malformed_entry` | Line 6 is Other("おおk /大/[く/多/]/大/") with Lf |
| `edge_nasi_marker` | Line 7 is Marker(OkuriNasi) with Lf |
| `edge_trailing_space` | Line 8 is Other("かんじ /漢字/ ") with Lf |
| `edge_embedded_cr` | Line 9 is Entry `かんじ` with candidates `["漢\r字"]` and Lf |
| `edge_section_comment` | Line 10 is Comment(";; comment inside section") with Lf |
| `edge_no_final_newline` | Line 11 is Entry `かっこ`, candidates `["[", "]"]`, with None |

The edge fixture has exactly 11 lines. In the expected entries above, candidates have no annotations and blocks are empty unless the row explicitly gives them.

The following generator-CLI tests also go in `tests/round_trip.rs`. Run `python3 -I` on a copy of gen-testdata.py under a temporary `tools/` directory. Every test gets its own temporary directory and runs from a different working directory. It never writes the checkout's testdata.

Build the expected bytes independently in the test code, before invoking the generator. Embed a literal copy of B, including its final LF, and the literal cookie lines, extra lines, and complete edge text from the Test data section. Do not read TASKS.md at runtime. For each filename, assemble exactly the text in that section's file table. Obtain UTF-8 bytes with Rust's standard library. Obtain UTF-16LE bytes with its standard-library encode_utf16, prefix literal FF FE, and use CRLF for every newline. For EUC-JP and EUC-JIS-2004 only, pass the independently assembled literal UTF-8 text on stdin to a separate `python3 -I` process that uses only the standard-library `euc_jp` or `euc_jis_2004` codec, capturing its bytes. This reference process must not import, execute, or parse gen-testdata.py, any other project module, or jisyo. Alternatively embed literal reference bytes verified against those standard-library codecs. Preserve U+301C, U+FF71, the literal backslash in `\057`, and the exact supplementary code points named above. The expected edge text has an embedded CR in 漢字, a space before line 8's LF, and no terminator on its last line. Do not get any expected bytes from generated files or a previous generator run. A test-only helper may share these literal fixtures and standard-library encodings between tests, but no helper from the generator.

| Test | Assert |
|---|---|
| `gen_testdata_root` | No arguments exits 0 and creates exactly the seven filenames listed above under the copied script's sibling testdata directory, not under the process working directory |
| `gen_testdata_bytes_basic_euc_jp` | Generation exits 0; basic.euc-jp.jisyo equals the independently encoded literal cookie, B, and `こう /丂/\n` |
| `gen_testdata_bytes_basic_euc_jis_2004` | Generation exits 0; basic.euc-jis-2004.jisyo equals the independently encoded literal cookie, B, and `ぱ /か゚/𠀋/𠂉/\n` |
| `gen_testdata_bytes_nocookie_euc_jis_2004` | Generation exits 0; nocookie.euc-jis-2004.jisyo equals the independently encoded literal B and `ぱ /か゚/𠀋/𠂉/\n`, without a cookie |
| `gen_testdata_bytes_basic_utf_8` | Generation exits 0; basic.utf-8.jisyo equals the independent UTF-8 bytes of its literal cookie, B, and `えもじ /😀/か゚/𠂉/丂/\n` |
| `gen_testdata_bytes_bom_utf_8` | Generation exits 0; bom.utf-8.jisyo equals literal EF BB BF followed by the independent UTF-8 bytes of B and the emoji entry, with no cookie |
| `gen_testdata_bytes_basic_utf_16le` | Generation exits 0; basic.utf-16le.jisyo equals literal FF FE followed by the independent UTF-16LE bytes of B and the emoji entry, with every LF replaced by CRLF |
| `gen_testdata_bytes_edge_utf_8` | Generation exits 0; edge.utf-8.jisyo equals the UTF-8 bytes of the complete literal 11-line edge text, with its specified CR, LF, CRLF, trailing space, and unterminated last line |
| `gen_testdata_repeat` | Invoke generation twice. After each invocation, compare every file to its independently specified expected bytes, never to the first invocation's output |
| `gen_testdata_check_matches` | Seed all seven files with independently specified expected bytes, without first running the generator. `--check` exits 0 and leaves each file's bytes and modification time unchanged |
| `gen_testdata_check_missing` | Seed the independently specified files except basic.utf-8.jisyo. `--check` exits 1 and does not create the missing file |
| `gen_testdata_check_differs` | Seed independently specified files, but use `sentinel\n` for basic.utf-8.jisyo. `--check` exits 1 and preserves the sentinel |
| `gen_testdata_bad_argument` | `--unknown` exits 2 with stderr `usage: gen-testdata.py [--check]\n` and creates no testdata directory |

Unit tests in `src/text.rs`:

| Test | Assert |
|---|---|
| `cookie_euc_jp` | `detect(b";; -*- coding: euc-jp -*-\n")` gives `(EucJp, false)` |
| `cookie_case_insensitive` | Name `EUC-JP` in the same cookie gives `(EucJp, false)` |
| `cookie_euc_japan_unix` | Name `euc-japan-unix` gives `(EucJp, false)` |
| `cookie_iso_alias` | Name `japanese-iso-8bit` gives `(EucJp, false)` |
| `cookie_eucjp_alias` | Name `eucjp` gives `(EucJp, false)` |
| `cookie_jisx0213_alias` | Name `euc-jisx0213` gives `(EucJis2004, false)` |
| `cookie_jis2004_dos` | Name `euc-jis-2004-dos` gives `(EucJis2004, false)` |
| `cookie_utf8` | Name `utf-8` gives `(Utf8, false)` |
| `cookie_utf8_unix` | Name `utf8-unix` gives `(Utf8, false)` |
| `cookie_unknown` | Name `shift_jis` gives `(UnknownCoding, line Some(1))` |
| `cookie_empty` | `detect(b";; coding:\n")` gives `(UnknownCoding, line Some(1))` |
| `cookie_ignored_outside_comment` | `detect(b"x coding: euc-jp\n")` gives `(Utf8, false)` |
| `cookie_ignored_after_first_line` | `detect(b"\n;; coding: euc-jp\n")` gives `(Utf8, false)` |
| `forced_matching_bom` | `read(EF BB BF 61, Some(Utf8))` gives a Utf8 Jisyo with bom true and `[Other("a") None]` |
| `forced_eucjp` | `read(b"abc", Some(EucJp))` gives an EucJp Jisyo with bom false and `[Other("abc") None]` |
| `forced_mismatched_bom` | `read(FF FE 61 00, Some(Utf8))` gives `(Decode, offset Some(0))` |
| `not_round_trip_forced` | `read(A9 DC AB DC 0A, Some(EucJis2004))` gives `(NotRoundTrip, offset Some(0))` |
| `not_round_trip_detected` | `read(A9 DC AB DC 0A, None)` gives `(NotRoundTrip, offset Some(0))` |
| `decode_offset_counts_bom` | `read(EF BB BF 61 FF, None)` gives `(Decode, offset Some(4))` |
| `read_empty` | `read(b"", None)` has no lines |
| `read_unterminated` | `read(b"a", None)` has `[Other("a") None]` |
| `read_blank_lf` | `read(b"\n", None)` has `[Blank Lf]` |
| `read_unterminated_cr` | `read(b"a\r", None)` has `[Other("a\r") None]` |
| `round_trip_empty` | Reading then writing literal `b""` gives those original bytes |
| `round_trip_unterminated` | Reading then writing literal `b"a"` gives those original bytes |
| `round_trip_blank_lf` | Reading then writing literal `b"\n"` gives those original bytes |
| `round_trip_unterminated_cr` | Reading then writing literal `b"a\r"` gives those original bytes |
| `write_encode_error_has_line` | A Jisyo with EucJp, bom false, lines `[Comment(";; ok") Lf, Comment(";; 😀") Lf]` gives `(Encode, line Some(2), offset Some(3))` |
| `write_utf16le_bom` | A Utf16Le Jisyo with bom true and `[Blank Lf]` writes literal `FF FE 0A 00` |
| `write_ignores_eucjp_bom` | An EucJp Jisyo with bom true and `[Blank Lf]` writes literal `0A` |
| `line_content_blank` | `line_content(Blank)` gives `""` |
| `line_content_comment` | Comment(";; synthetic") gives `";; synthetic"` |
| `line_content_marker_ari` | Marker(OkuriAri) gives `";; okuri-ari entries."` |
| `line_content_marker_nasi` | Marker(OkuriNasi) gives `";; okuri-nasi entries."` |
| `line_content_entry` | A literal Entry with midashi `かんじ`, candidate word `漢字`, annotation Some("注"), and no blocks gives literal `"かんじ /漢字;注/"`; do not compute the expected string with format_entry |
| `line_content_other` | Other("junk") gives `"junk"` |

Each row calls the named public function. Cookie rows that specify only a name use `b";; -*- coding: NAME -*-\n"`. Error tests assert the indicated code and position together as one public error result. Do not directly test `check_round_trip` or any other private helper.

Must fail without: the round-trip guard (`check_round_trip` and its call in `read`) → `not_round_trip_forced`, `not_round_trip_detected`.

Done: `make check` exits 0.

## J5. Dict and IME profiles

Needs: J4.

Spec: SPEC.md "Layers", "Meaning of a file".

Files: `src/dict.rs`, `src/ime.rs`, `src/lib.rs`, `tests/dict.rs`, `tests/layers.rs`.

Implement `DictReport`, `from_jisyo`, and `without_blocks` in `src/dict.rs` (core: it uses only `model`). Implement `profile`, `target_name`, `parse_target`, `Rendered`, and `render` in `src/ime.rs`. Append both `pub mod dict;` and `pub mod ime;` to `src/lib.rs`, in that order. Add `("src/dict.rs", include_str!("../src/dict.rs"))` to the list in `tests/layers.rs`.

Put all tests below in `tests/dict.rs` as separate integration tests through the public API. Resolve every `testdata/` path from `env!("CARGO_MANIFEST_DIR")`. Do not put file-reading tests in the Core source files. For expected EUC-JP bytes, use Python's standard-library `euc_jp` codec on the literal expected string, invoked with `python3 -I` through stdin and captured stdout, or copy its independently verified literal bytes into the test. Never use our Codec, render, or write to compute an expected value. The UTF-8 and UTF-16LE byte expectations use Rust's standard-library encoders on the literal strings.

The test Dict `D` used below is:

- `okuri_ari`: `おおk` with candidates `[大, 多]` and blocks `[(く, [多])]`.
- `okuri_nasi`: `かんじ` with candidates `[漢字, 幹事;組織の世話役]` (the second has the annotation `組織の世話役`), then `えもじ` with candidates `[😀, 笑]`.

| Test | Assert |
|---|---|
| `ari_file_order` | `from_jisyo` of basic.euc-jp has ari midashi `[わたs, もt, おおk]` |
| `nasi_file_order` | The same input has nasi midashi `[かんじ, から, あ, きごう, こう]` |
| `basic_has_no_other_lines` | The same input has empty other_lines |
| `basic_has_no_duplicate_lines` | The same input has duplicate_lines 0 |
| `basic_has_no_duplicate_candidates` | The same input has duplicate_candidates 0 |
| `other_line_numbers` | from_jisyo of edge.utf-8 has other_lines `[2, 6, 8]` |
| `dict_keeps_empty_block` | The same edge input has one ari entry `おおk`, no main candidates, blocks `[(く, [])]` |
| `dict_keeps_embedded_cr` | The same edge input's nasi entries are `かんじ` with `["漢\r字"]`, then `かっこ` with `["[", "]"]` |
| `duplicate_lines_count` | The duplicate fixture below gives duplicate_lines 2 |
| `duplicate_candidates_count` | The duplicate fixture gives duplicate_candidates 4 |
| `duplicate_ari_merge` | Its ari is `おおk` with main candidates `[大, 多]` and blocks `[(く, [多, 大]), (き, [大])]` |
| `duplicate_nasi_merge` | Its nasi is `かんじ` with candidates `[漢字, 感じ, 幹事]`, all without annotation |
| `candidate_identity_ignores_annotation` | Reading `";; okuri-nasi entries.\nかんじ /漢字;first/漢字;second/\n"` then from_jisyo gives one candidate 漢字 with annotation Some("first") |
| `blocks_with_same_okuri_merge` | Reading `";; okuri-ari entries.\nおおk /[く/大/]/[く/多/]/\n"` then from_jisyo gives one block `(く, [大, 多])` |
| `profile_corvus` | profile(Corvus) equals the literal Profile with Utf16Le, bom true, CrLf, cookie None, blocks true |
| `profile_fcitx5` | profile(Fcitx5) equals the literal Profile with EucJp, bom false, Lf, cookie Some(";;; -*- coding: euc-jp -*-"), blocks false |
| `profile_skkeleton` | profile(Skkeleton) equals the literal Profile with Utf8, bom false, Lf, cookie None, blocks true |
| `render_corvus_bytes` | Writing render(D, Corvus).jisyo gives `FF FE` followed by the standard-library UTF-16LE encoding of `";; okuri-ari entries.\r\nおおk /大/多/[く/多/]/\r\n;; okuri-nasi entries.\r\nかんじ /漢字/幹事;組織の世話役/\r\nえもじ /😀/笑/\r\n"` |
| `render_corvus_skipped` | render(D, Corvus).skipped is 0 |
| `render_fcitx5_bytes` | Writing render(D, Fcitx5).jisyo gives the independent Python euc_jp encoding of `";;; -*- coding: euc-jp -*-\n;; okuri-ari entries.\nおおk /大/多/\n;; okuri-nasi entries.\nかんじ /漢字/幹事;組織の世話役/\nえもじ /笑/\n"` |
| `render_fcitx5_skipped` | render(D, Fcitx5).skipped is 1 |
| `render_skkeleton_bytes` | Writing render(D, Skkeleton).jisyo gives the standard-library UTF-8 encoding of `";; okuri-ari entries.\nおおk /大/多/[く/多/]/\n;; okuri-nasi entries.\nかんじ /漢字/幹事;組織の世話役/\nえもじ /😀/笑/\n"` |
| `render_skkeleton_skipped` | render(D, Skkeleton).skipped is 0 |
| `render_corvus_reads_back` | from_jisyo(read(write(render(D, Corvus).jisyo), None)).dict equals the literal D |
| `render_skkeleton_reads_back` | The same public file round trip for Skkeleton equals the literal D |
| `render_drops_unencodable_midashi` | With Fcitx5 and only nasi entry 😀 with candidates `[a, b]`, there are no output entries |
| `render_counts_unencodable_midashi` | The same input gives skipped 2 |
| `render_drops_unencodable_candidate` | With Fcitx5 and nasi entry `かんじ` with candidates `[😀;x, 漢字]`, the only remaining candidate is 漢字 |
| `render_counts_unencodable_candidate` | The same input gives skipped 1 |
| `render_drops_unencodable_okuri` | With an EucJp profile that keeps blocks, ari entry おおk with main `[大]` and block `(😀, [多, 大])` retains the main candidate and drops that block |
| `render_counts_unencodable_okuri` | The same input gives skipped 2 |
| `render_drops_empty_block` | With the same block-preserving profile, ari entry おおk with main `[大]` and block `(く, [😀])` retains the main candidate and drops the emptied block |
| `render_drops_empty_entry` | With Fcitx5 and only nasi entry かんじ with `[😀]`, output has no entries |
| `render_profile_drops_blocks` | With Fcitx5 and only ari entry おおk with no main candidates and block `(く, [多])`, output has no entries |
| `render_dropped_profile_blocks_not_skipped` | The same input gives skipped 0 |
| `without_blocks_drops_only_blocks` | without_blocks(D) equals a literal D whose おおk blocks vector is empty; do not compute the expected Dict by calling another jisyo function |
| `target_name_corvus`, `target_name_fcitx5`, `target_name_skkeleton` | Three separate tests give respectively `corvus`, `fcitx5`, `skkeleton` |
| `parse_target_corvus`, `parse_target_fcitx5`, `parse_target_skkeleton` | Three separate tests parse those literal names to the corresponding literal Target |
| `parse_target_rejects_case` | parse_target("Corvus") is None |
| `parse_target_rejects_empty` | parse_target("") is None |

The duplicate fixture is the literal UTF-8 text `";; okuri-ari entries.\nおおk /大/[く/多/]/\nおおk /多/大/[く/大/多/]/[き/大/]/\n;; okuri-nasi entries.\nかんじ /漢字/漢字;注/感じ/\nかんじ /幹事/感じ/\n"`. Read it through text::read before calling from_jisyo. A block-preserving EucJp profile means encoding EucJp, bom false, eol Lf, cookie None, blocks true. In all synthetic Dicts above, unspecified lists are empty and unspecified annotations are None.

Done: `make check` exits 0.

## J6. Pull-request and main-branch CI

Tier: R1s

Needs: nothing. The existing crate, Cargo.lock, and Makefile on main are sufficient. This card does not inherit or require the J1 implementation, J2-J5, or the store-contract branch.

Spec: SPEC.md "Dependencies and licenses"; Cargo.toml; Cargo.lock; Makefile; `/data/softwares/skk/CLAUDE.md` "GitHub" and merge criteria; issue https://github.com/kkiyama117/kskk/issues/4.

Files: `.github/workflows/check.yml` only. Its parent directories may be created. Do not edit Rust or Python code, dependency files, Makefile, toolchain files, TASKS.md, or test data.

### Workflow contract

Write a YAML workflow named `Jisyo check`. Its event mapping contains only unfiltered `pull_request` and `push` with `branches: [main]`. Do not add branch, path, or activity filters to pull_request. Do not use pull_request_target, schedule, workflow_run, or workflow_dispatch. Its only job has id `check`, name `make check`, `runs-on: ubuntu-24.04`, and `timeout-minutes: 10`. Do not add a matrix, container, services, job or step conditions, or continue-on-error at either level.

Set root-level permissions to exactly `contents: read`. No job-level permissions key is allowed. Set the job environment variable `RUSTUP_TOOLCHAIN` to the literal string `'1.94.0'` and do not override it at step level. Do not set RUSTC_BOOTSTRAP or select a moving stable/nightly channel. The workflow has exactly these seven steps, in order:

1. `Checkout`. Use `actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1` with end-of-line version comment `v7.0.1`. Set persist-credentials false and fetch-depth 1. Do not set ref, token, SSH key, submodules, or another repository. On pull_request, the default event checkout must test the head merged into the base main, not the head alone. On a main push it tests the pushed commit.
2. `Set up Python`. Use `actions/setup-python@6d4fd22b20465ccc0f7354476588e8df7c9ca0e0` with end-of-line version comment `v7.1.0`. Set python-version to `'3.15.2'`, architecture to `'x64'`, and allow-prereleases false. Do not add a cache, mirror, or token, or fall back to the system Python.
3. `Set up Rust`. Run exactly `rustup toolchain install 1.94.0 --profile minimal --component rustfmt --component clippy --no-self-update`. This uses the runner's rustup executable, not an additional action. Its failure must fail the job, including if rustup is absent. Do not add an unverified action SHA, installer download, alternate toolchain, or fallback. RUSTUP_TOOLCHAIN applies to the later version, fetch, and gate commands too.
4. `Tool versions`. Run exactly `python3 --version && rustc --version && cargo --version`. The native log must identify Python 3.15.2 and stable rustc/cargo 1.94.0, not nightly. Do not suppress command failures.
5. `Check iconv`. Run exactly `command -v iconv && iconv --version`. A missing or failing converter fails the job. Record its actual native version; do not require the local machine's glibc version or regenerate the real table in this step.
6. `Fetch locked dependencies`. Run exactly `CARGO_NET_OFFLINE=false cargo fetch --locked` from the repository root. This is CI preparation for a fresh runner before the offline gate. It downloads only the dependencies selected by Cargo.lock and must fail rather than update the lock file. This command is written into the workflow, not executed by the implementer locally; AGENTS.md's local no-fetch rule still applies.
7. `Run gate`. Run exactly `make check` from the repository root. Do not narrow it to selected tests, override CARGO_NET_OFFLINE to false, suppress its exit, or allow failure. The Makefile exports CARGO_NET_OFFLINE=true and runs fmt, clippy with warnings as errors, and cargo test.

The two action/version pins and Python version match the locally checked conformance workflow at commit 988d9239fca0fae568a1bbddafafbc7e13c1b8f5. Stable Rust 1.94.0, its clippy and rustfmt components, and the full gate including J1's 16 tests were checked locally. Current crate and J1 code do not require nightly. This pin is a tested stable version, not a claim about the minimum supported Rust version or future unimplemented cards. No Rust setup action is needed, so this card has no unverified Rust action SHA.

Python3 and iconv are explicit runner requirements. J1's CLI tests place the synthetic iconv double on a test child's PATH; they must not trigger real table regeneration during the gate. The actual GNU converter is still checked as a runner prerequisite. Do not add pip/apt installation, a package cache action, external engines, containers, real dictionaries, secrets, uploads, publication, or deployment. Network use by checkout, interpreter/toolchain setup, and locked dependency preparation is separate from the offline make check gate.

### Verification

Run the full `make check` locally with RUSTUP_TOOLCHAIN=1.94.0 and report its exit code and last 20 lines. Do not install or fetch a missing local toolchain, component, or crate; report that blocker. No new unittest, script, or fixture belongs to this card. The integration being added is native GitHub Actions running the existing public gate; comparing a YAML file with a second copy of itself would not test it.

The names below are separate artifact or native-CI acceptance checks, not new Rust/Python test function names. An artifact assertion checks the written YAML. Native assertions require an actual Actions run and cannot be claimed from a local gate or a worker report.

| Acceptance check | Observable assertion and evidence |
|---|---|
| `ci_events` | The only events are unfiltered pull_request and push with branches [main]. |
| `ci_runner` | Exactly one check job uses ubuntu-24.04, name make check, and timeout 10, with no matrix, container, condition, or continue-on-error. |
| `ci_action_pins` | Exactly two uses values have the complete action SHAs and version comments above. No Rust or cache action is added. |
| `ci_readonly_token` | Root permissions are exactly contents read and no job-level override exists. |
| `ci_no_persisted_credentials` | Checkout explicitly sets persist-credentials false and adds no credential input. |
| `ci_pr_merge` | Checkout has no ref override. For the native PR run record the head SHA receiving the check, the base main commit, and the tested merge SHA. It must be the merge result, not a head-only checkout. |
| `ci_python_version` | The native Tool versions log prints Python 3.15.2, matching the exact setup-python version and x64/non-prerelease settings. |
| `ci_rust_toolchain` | The artifact pins job RUSTUP_TOOLCHAIN to 1.94.0 and installs minimal with clippy and rustfmt. The native version log prints stable rustc and cargo 1.94.0 with no bootstrap setting or fallback. |
| `ci_iconv_available` | The native Check iconv step succeeds and logs the converter path/version before the gate. A YAML command alone does not prove availability. |
| `ci_offline_gate_preparation` | Locked dependency fetch precedes the exact make check command, without modifying dependency/build files or turning off the Makefile's offline gate. |
| `ci_pr_gate` | The card's pull_request run tests its merge result into main and executes the complete make check successfully. Record the run URL, PR head, base main commit, and tested merge SHA; skipped or selected-test-only execution does not pass. |
| `ci_main_push_gate` | A native push-to-main run containing this workflow executes the same complete gate successfully. Record its run URL and main commit; a feature-branch push is not this evidence. |

Must fail without: the explicit root read-only permissions declaration → `ci_readonly_token`.
Must fail without: disabling persisted checkout credentials → `ci_no_persisted_credentials`.

These guard-removal checks are assertions on the workflow artifact, not unittest runs. Removing each declaration must make its named artifact assertion fail. Restore the workflow byte-for-byte before any native run. Never send a deliberately weakened workflow to GitHub. Report artifact guard checks separately from local unit-test counts and native CI evidence.

Open: none. All action SHAs and language-version choices used above have local evidence; native runner provisioning and execution remain acceptance checks, not assumed successes.

Done for PR acceptance: the pinned local full gate exits 0, artifact checks hold, and the native pull_request gate succeeds on the merge result into main. The main-push check is post-merge completion evidence, not a prerequisite that can exist before that merge. Status: done additionally requires inclusion in origin/main and the successful native main-push run. Local success alone is not native CI evidence. No path outside Files is changed.
