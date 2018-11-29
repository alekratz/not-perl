# Ideas

This is more of a list of a "wouldn't it be neat if..." list. Points will have personal notes and
this should probably be converted to a wiki format.

This has a sister document, TODO.md, which lists more concrete ideas that need to be implemented.

# Language

## General

* Library and module system
* Compiler directives
* Lazy functions
* Function memoization
    * How in the heck do we do this
* Macros
* Pattern matching syntax

## Syntax

* Allow for string prefixes/suffixes
* Regex literals(?)
    * If they start with a / it may be tough to do with known prefixes.
    * However, the parser is LL(1) with a single lookahead, and we can make the parser LL(k) if
      necessary. This may be possible with LL(2).
* Suffix operators
* Circumfix operators?
    * May be more complicated

## Error handling

* Implicit vs. explicit error handling
    * This is most likely going to be exceptions vs. error codes
    * I lean towards implicit
    * Thrown errors do not necessarily need to be the "exception" type, it just needs to be a value
    * keywords: try/catch/finally/throw
    * Allow try { } without a catch block
* Distinguish between catchable errors (i/o read error, invalid user input, file not found) and
  uncatchable errors (type errors, programmer errors, null pointer exceptions)
    * Assertions are uncatchable, exceptions are catchable

## Runtime

* JIT compilation
    * Check out QBE for this https://c9x.me/compile/

# Compiler facilities

* Testing framework
* Linting
    * "while" loops can be checked if they have constant conditions and suggest using a "loop" instead
    * ? and ! symbols at the end of barewords as a convention and linting for booleans and things that change program state
    * Built-in linter with configurable lints
        * Do this in TOML?
        * Something similar to pylint
