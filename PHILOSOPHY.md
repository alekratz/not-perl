# Philosophy of the language

Builtin functions should encompass the most *important* and *common* functions.
The list of builtins may become unwieldy. *This is okay*. Purity is *not* a
priority.

Performance intensive stuff implemented in the language itself is likely to be
very, very slow. Things like:
 * regex engine
 * grep
 * strings
 * string find, string replace
 * yaml/json parsing, maybe?
 * language primitives
 * the language itself
 * the language's package manager, maybe?
 
should **not** be implemented in this language, and instead offloaded onto other
languages.

This language *uses* these features. It is not designed to be big and powerful
enough to *provide* them.
