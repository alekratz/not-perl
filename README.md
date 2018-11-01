# not-perl

A dumb language that isn't Perl.

## Current status

This project is currently in the middle of a backend rewrite. Nothing is actually set up to run, and
only a few tests are available.

Basically, this is a whole lotta nothing for the time being. Hopefully I'll update this README when
this rewrite is complete.

# Build and use

This project requires Rust 1.30-beta.

1. Install [rust](rustup.rs)
2. In the project directory, run `rustup set override beta`
3. Run `cargo check` to make sure it builds correctly
4. Run `cargo run -- examples/blocks.npl` to watch the compiler break

More examples in `examples/`.

If you build and get lifetime errors, switch to nightly with
`rustup override set nightly` and enable non-lexical lifetimes in main.rs by
uncommenting `#![feature(nll)]` at the top, and try to build it again.

## Shame statistics
To generate shame statistics, you can run these commands:

`grep -R src/ -e 'unimplemented\|TODO' | wc -l`

This is roughly the square root of the number of hours of technical debt payoff
I have created for myself.

# Disclaimer

## If you're reading this, nothing works.

# License

(C) 2018 Alek Ratzloff. All rights reserved.

I'll make this more open when I'm happy with its stability. Until then, ORIGINAL
CONTENT, DO NOT STEAL!
