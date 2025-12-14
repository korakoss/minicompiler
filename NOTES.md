Things to do in the next few weeks


# Meta-plans
Work on the static analysis.
Possibly sort out things in the "Rough edges" section, or other sections of "v1 plans".
Generally just get the code cleaner and ready for starting v2 work on it.
Start adding the types when it feels ready.


## Static analysis
Work in the "analyzing.rs" file.
1. Start with validating variable names (scope-sensitively). 
	- we might add type info already to the machinery, just not use it (eg. type everything as int, which is kinda fair)
2. This should enable allocating the right amount of memory to variables by counting them, instead of the constant 256 bytes, so fix that
3. Also validate function names 
4. And also function argument count (in a separate pass probably)


# v1 plans 

## Rough edges 
- allocating 8 bits per var depspite it's 4 on arm32
- emit should do the format! itself (and probably not take _borrows_, also mega annoying


## Easy wins
- Negation
    - introduce Expression::UnaryOp
- Remaining comparison operators (>, !=) 
    - maybe >= and =< but not necessary
- add commenting -- eg //
- add support for signed integers 
    - mostly a lexing issue i think -- need to parse the pattern -[intliteral]

## Quality of life 
- print tokenization and AST human-readably
	- we can make a simple recursive function to display it with indents?
- make scripts that show tokens, then AST, then assembly, then run
- add syntax highlighting
    - vim script / treesitter

# v2 plans

## Phase 1: Typing 
Scope:
    - start off with primitive types _int_ and _bool_
    - and perhaps _None_? that could be nice for eg having _Option_-s early
    - add _struct_-s 
    - add _enum_-s
At each step:
    - introduce the type, add the keywords for it
    - rewrite the binary/whatever operation parsing/analyzing to check for the correct type
        - (skip function checking for now)
        

## Phase 2: Functions - harder stuff
- implement type checking through functions
- support arbitrary number of arguments via pushing to stack

## Phase 3: Arrays
(I don't want to do this earlier because I don't want to build untyped arrays then restrict to typed ones)

## Next phase candidates
- *mut*ability stuff
- subtyping, | operator on types to go in function signatures


