
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
> hir
- ast-to-hir
    - **TODOs:**
        - returns in func bodies having the correct type
        - assign values having the type of the target

### PLAN
- hir-codegen
- delete the AST codegen

### TESTS
- write a small "test suite" of yum programs to check compiler updates against
- create an automatic test script too
- specific test cases:
    - being in a loop nest to break/cont and in a func for return (who's checking this btw? HIR-lowerer?)
    - a nonparametric function -- unconfident about AST parsing there
    - long ass binop expression
    - loop/branch conditions being bool
    
### FINISH
- grep for TODOs
- run tests 
- clean up notes
- merge


# Next progess steps

## Better error handling
- Result types, propagating up, etcetc?

## More types

### Structs
- just make normal struct stuff (I guess with int and bool -- and struct -- fields)
    - this makes variables have different sizes and thus forces us to start counting and using it in stackspace alloc

### The Option<T> push
- add _None_ type 
- _enums_-s
- add generics
- NOTE: we might start needing _match_ here

### Floats
- which also enable the Division binop etc

## Stack args 
- support arbitrary number of arguments via pushing to stack

## Phase 3: Arrays
(I don't want to do this earlier because I don't want to build untyped arrays then restrict to typed ones)

## Other features 
- *mut*ability stuff
- subtyping, | operator on types to go in function signatures



# "Meh, do it someday"

## Cringe things 
- currently allocating 8 bits per var depspite it's 4 on arm32 (or is it?)
- refactor thing to do less clone()-s and other stopgap practices 

## Niceties
- add syntax highlighting
    - vim script / treesitter
- improving print()
    - not all types might be naively printable
    - we want bool prints to print True/False not 1/0, etc
- _if else_ syntax sugar
- trailing expression returns
- long ints

## Easy but unnecessary wins
- Negation
    - introduce Expression::UnaryOp
- Remaining comparison operators (>, !=) 
    - maybe >= and =< but not necessary
- add commenting -- eg //
- add support for signed integers 
    - mostly a lexing issue i think -- need to parse the pattern -[intliteral]
