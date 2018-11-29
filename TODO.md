# TODO

This is a list of things that need to be done before (insert deadline here).

This is also by no means a comprehensive list, unless it is. This document itself has a sister
document, IDEAS.md, with less concrete things that I think I want in the language but have not
fleshed out as well.

## Backend rewrite

* VM
    * VM pieces need ranges and positions like IR has
    * Implementations needed:
        * Bytecode
        * VM storage

# General

* A better name than "not perl"
* Vim syntax highlighting
* Logging for debug, warning, verbose, etc
    * `slog` crate may be too complicated, `env_logger` with terminal may be all we need for this

# Language

* Language spec would be a wise move at this point
* Arrays
* Internal memory model
    * How are strings allocated?
        * How is UTF-8 handled?
    * How are user objects allocated?
    * How are built-in objects exposed?
        * How are these allowed to be mutated?
    * How do we allow all of this to be overridden by the language?
    * Arrays

## Syntax

* Allow calling functions without parens
* Varargs + kwargs
    * Function varargs + kwargs declaration
    * Varargs and kwargs spread operators
        * Python's `*args` and `**kwargs` are attractive here

## Builtin functions

These are all located in `vm::function` (in src/vm/function.rs).

* `writef`
* `readf`
* `print`
* `println`
* `readln`
* `plus_binop`
* `minus_binop`
* `splat_binop`
* `fslash_binop`
* `tilde_binop`

# Tests

* Add more examples

# Compiler facilities

* REPL
