Things to do in the next few weeks


# Meta-plans
Clean up the current code, get it all working properly.
Then add the typing things.


# ROUGH EDGES
- variable overwriting within blocks
- allocating 8 bits per var depspite it's 4 on arm32
- validating called function even exists
- emit should do the format! itself (and probably not take _borrows_, also mega annoying
- keep track of valid variable names in scope, valid funcnames, etc
- check function call correctness (argcount now, typing later)
- variable allocation
    - currently a fixed 256 bytes are allocated to variables 
    - this should be fixed by variable counting or whatever


# v1 plans 

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


# v2 plans

## Phase 1: Typing 
- start off with primitive types _int_ and _bool_
- and perhaps _None_? that could be nice for eg having _Option_-s early
- add _struct_-s 
- add union types (especially if None was already implemented)
- add _enum_-s
- add _mut_


## Phase 2: Arrays
(I don't want to do this earlier because I don't want to build untyped arrays then restrict to typed ones)
