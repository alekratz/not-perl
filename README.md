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

Disclaimer
=

If you're reading this, the compiler doesn't work.

License
=

(C) 2018 Alek Ratzloff. All rights reserved.

I'll make this more open when I'm happy with its stability. Until then, ORIGINAL
CONTENT, DO NOT STEAL!
