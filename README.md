not-perl
=

A dumb language that isn't Perl. Trust me.

Build and use
=

1. Install [rust](rustup.rs)
2. Run `cargo check` to make sure it builds correctly
3. Run `cargo run -- examples/blocks.npl` to watch the compiler break

More examples in `examples/`.

If you build and get lifetime errors, switch to nightly with
`rustup override set nightly` and enable non-lexical lifetimes in main.rs by
uncommenting `#![feature(nll)]` at the top, and try to build it again.
