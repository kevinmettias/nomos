# Provenance: `hex-0.4.3/`

`P45-RULES-CALIBRATED-AGAINST-CODE-THEY-WERE-NOT-TUNED-ON`'s fixture: idiomatic
third-party Rust source, written by someone who never heard of this workspace's rules,
committed here so the composed rule set can be measured against it in
`tests/integration/tests/calibration.rs`.

## Source

- Crate: [`hex`](https://crates.io/crates/hex)
- Version: `0.4.3`
- Repository: <https://github.com/KokaKiwi/rust-hex>
- Author: KokaKiwi <kokakiwi@kokakiwi.net>
- Drawn from this machine's own Cargo registry cache:
  `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hex-0.4.3/`

## Files and what was kept

- `hex-0.4.3/lib.rs` — copied verbatim from the crate's own `src/lib.rs`, **except**:
  the `#[cfg(test)] mod test { ... }` block at the end of the original file was removed
  (it uses `pretty_assertions::assert_eq`, a dev-dependency test framework, which would
  have exercised this workspace's rules against a test-harness choice rather than
  against `hex`'s own library code), and the two `#[cfg(feature = "serde")]`-gated `mod
  serde_untagged;`/`pub use` lines were removed (the `serde` feature's own module is not
  part of this excerpt, and leaving the `mod` declaration in without the file behind it
  would be an artifact of trimming, not something `hex`'s own author wrote this way).
  Everything else — doc comments, the crate-level attributes, the `BytesToHexChars`
  iterator, both public traits, every public function, the `from_hex_array_impl!` macro
  and its invocations — is unmodified.
- `hex-0.4.3/error.rs` — copied verbatim from the crate's own `src/error.rs`, **except**:
  the `#[cfg(test)] mod tests { ... }` block (again gated on `pretty_assertions`) was
  removed. `FromHexError`, its `Display` impl and its `std::error::Error` impl are
  unmodified.
- `hex-0.4.3/standards.json` — **not** part of the upstream crate. This is the fixture's
  own declared convention (`OD-RULES-011`'s `nomos.cap.naming.policy` read), stated so
  the composed naming rules resolve against `hex`'s own real, ordinary Rust convention
  (`lower_snake` functions, modules and fields) rather than silently falling back to
  this workspace's own `Pascal_Snake_Case`/`upper-snake` default. Checked directly
  against the copied source: every function, method and module name here is
  `lower_snake`; there are no struct fields declared at all in the excerpt actually
  submitted.

## Licence

`hex`'s own `Cargo.toml` declares `license = "MIT OR Apache-2.0"`. This fixture keeps
the excerpt under the same terms and reproduces the registry cache's own `LICENSE-MIT`
text below in full (the `LICENSE-APACHE` file sits alongside it in the same registry
cache directory for anyone who wants the Apache-2.0 text instead). `hex-0.4.3/lib.rs`
itself opens with the crate's own copyright header, unmodified, naming both licences by
reference.

```
Copyright (c) 2013-2014 The Rust Project Developers.
Copyright (c) 2015-2020 The rust-hex Developers

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```
