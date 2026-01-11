Bugs and similar things.


# Bugs/issues 

## Void funccall parsing
The parser currently assumes that an stmt starting with an identifier token is an assignment's lValue
    - However, it can actually be other things, eg. a void function call

## Return checks
It is not checked whether a function does return on any execution path. (Probably do this in MIR).Also, void functions work awkwardly, relying on this wacky Return(None) insert in HIR->MIR lowering, which is also bad. Void functions should probably get a tail Return(None) in AST->MIR instead.

## Struct literal parsing
Struct literals currently needs a trailing comma after the last field, change this. By the way, the other comma-related subparsers instead _don't_ allow trail commas, that could be made more liberal instead (not priority).

Also, I think struct definitions have to _precede_ their uses in literals, which is also inconvenient.

## Print sloppinesses
The printf call assumes that it's printing an int. This fails on structs and prints booleans as ints. Not too urgent to fix, or make print smarter, but maybe we could typecheck.

## Binop typecheck sloppiness
It currently typechecks any a==b expression as valid if the two types are the same, despite this not being implemented for structs.

## Struct moves
Moves on large types currently don't work.

# Could be done nicer

## Hashmap things
HashMap has nondeterministic order, which is somewhat annoying.
Also, whether we keep it or not, it and probably the alternatives too, have an .insert() that _overwrites_. Which is probably not a big issue but it'd be nice to be careful.

## Code quality
Eventually get rid of cloning and so on, turn it into proper Rust.


# Questions
- Do struct arguments and struct returns work currently?
- Can we move structs currently or does it fail?
    - I'm not sure. Maybe MIR -> HIR decomposes it correctly?
    - If not:
        - Not a big issue due to the funccall ABI
        - But eventually write a large value move routine, probably not even hard to do it

# Design questions

## Main func choice
Currently, the logic allows multiple functions named _main_ if the signatures differ, and picks the last or whatever. This should be changed. To what? Only one named main? And what signature to allow?



