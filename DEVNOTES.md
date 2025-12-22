
# Capn log XII.22

HIR->assembly codegen possibly works now. Test it, clean up, merge maybe.
Improved the running scripts: _yumc_ compiles, and (courtesy of dotfiles/) _ycp_ compiles and runs on pi, _yumpi_ jumps to project folder on pi. 

## OVERHAUL STUFF
- main.rs finalized
- common.rs done
- ast.rs done

- QUESTION: who's checking being in a loop nest/fun currently. Is it the HIR lowerer?
    - write a test case for this
    - other testcase: write a nonparametric function
        - unconfident about AST parsing there

# Current things

## Next steps
- finalize stuff
    - possibly merge even
    - grep for TODOs
    - _significant_ cleanups on the codebase
- write a small "test suite" of yum programs to check compiler updates against
    - create an automatic test script too
- rename everything to make more sense

## Typechecking gaps
- returns in func bodies having the correct type
- assign values having the type of the target



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
- currently allocating 8 bits per var depspite it's 4 on arm32

## Niceties
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
