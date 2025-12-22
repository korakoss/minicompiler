
# Capn log XII.22

HIR->assembly codegen possibly works now. Test it, clean up, merge maybe.
Write nicer scripts (this can even enable having a small "test suite" of Yum programs nicely).


# Current things

## Next steps
- finalize stuff
    - possibly merge even
    - _significant_ cleanups on the codebase
- nicer scripts 
    - Yum codefiles and other pipeline stage results should go in a Yum folder's subfolders
    - we might want to also do a bit nicer compiler internal stage displays ("serialize" AST and HIR, save to file)
- rename everything to make more sense

## Typechecking gaps
- returns in func bodies having the correct type
- assign values having the type of the target



# Next progess steps

## More types

### Structs
- just make normal struct stuff (I guess with int and bool -- and struct -- fields)
    - this makes variables have different sizes and thus forces us to start counting and using it in stackspace alloc

### The Option<T> push
- add _None_ type 
- _enums_-s
- add generics
- NOTE: we might start needing _match_ here

## Stack args 
- support arbitrary number of arguments via pushing to stack

## Phase 3: Arrays
(I don't want to do this earlier because I don't want to build untyped arrays then restrict to typed ones)

## Other features 
- *mut*ability stuff
- subtyping, | operator on types to go in function signatures



# "Meh, do it someday"

## Rough edges & QoL improvement areas
- allocating 8 bits per var depspite it's 4 on arm32
- add syntax highlighting
    - vim script / treesitter
- improving print()
    - not all types might be naively printable
    - we want bool prints to print True/False not 1/0, etc


## Easy but unnecessary wins
- Negation
    - introduce Expression::UnaryOp
- Remaining comparison operators (>, !=) 
    - maybe >= and =< but not necessary
- add commenting -- eg //
- add support for signed integers 
    - mostly a lexing issue i think -- need to parse the pattern -[intliteral]
