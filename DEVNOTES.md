
# Capn log XII.22

HIR->assembly codegen possibly works now. Test it, clean up, merge maybe.
Improved the running scripts: _yumc_ compiles, and (courtesy of dotfiles/) _ycp_ compiles and runs on pi, _yumpi_ jumps to project folder on pi. 
Currently doing a finializing overhaul of the present codebase (renames, code cleanups, etc).

## OVERHAUL STUFF


### DONE, FINALIZED
- main.rs 
- common.rs 
- ast.rs 
- (lex.rs)
- parse.rs 

### CURRENT STEP
- hir.rs (probaby in final form now)
- hir_builder.rs
    - **TODOs:**
        - returns in func bodies having the correct type
        - assign values having the type of the target
        - turn hashmaps to vecs if possible?
        - (some final touches)
- rething who owns what in HIR


### PLAN
- hir-codegen
- delete the AST codegen

### TESTS
- write a small "test suite" of yum programs to check compiler updates against
- create an automatic test script too
    - (maybe we should wait until the 'sync' reliability works again, ie home network(?))
- specific test cases:
    - being in a loop nest to break/cont and in a func for return (who's checking this btw? HIR-lowerer?)
    - a nonparametric function -- unconfident about AST parsing there
    - long ass binop expression
    - loop/branch conditions being bool
    - _let_ definition within a loop (or actually also an if)
    - various typechecking tests
        - typing binops, funcs, etc

### FINISH
- grep for TODOs
- run tests 
- final touches
    - clean up notes
    - more clearly declared imports?
- merge


# Next progess steps

## (Whenever) Better error handling
- Result types, propagating up, etcetc?
- (whenever it gets too annoying to debug without)

## Phase 1: More type stuff

### Structs and stuff
- just make normal struct stuff (I guess with int and bool -- and struct -- fields)
    - this makes variables have different sizes and thus forces us to start counting and using it in stackspace alloc
- once we are doing sizes anyway, we could also deal with a number of other things:
    - various int sizes 
    - signed ints (u/i)
        - mostly a lexing thing I think
        - also needs primitive type names or something
    - currently allocating 8 bits per var despite it's 4 on arm32 (or is it?)
        - not sure but the printprimes program seems to cap out at 128 so something is up

### The Option<T> push
- add _None_ type 
- _enums_-s
- add generics
- NOTE: we might start needing _match_ here

### Floats
- which also enable the Division binop etc

## Phase 2: Stack args 
- support arbitrary number of arguments via pushing to stack

## Phase 3: Arrays
(I don't want to do this earlier because I don't want to build untyped arrays then restrict to typed ones)

## Later
- *mut*ability stuff
- subtyping, | operator on types to go in function signatures


# "Meh, do it someday"

## Cringe things 
- refactor thing to do less clone()-s and other stopgap practices 
    - not sure how much of a problem this is anymore

## Niceties
- add syntax highlighting
    - vim script / treesitter
- improving print()
    - not all types might be naively printable
    - we want bool prints to print True/False not 1/0, etc
- _if else_ syntax sugar
- trailing expression returns
- add commenting -- eg //

## Easy but unnecessary wins
- Negation
    - introduce Expression::UnaryOp
- Remaining comparison operators (>, !=) 
    - maybe >= and =< but not necessary
