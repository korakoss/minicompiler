Things to do in the next few weeks


# Meta-plans

So basically, the plan is: make some sort of acceptable v1, then proceed to v2.
Scheduling can generally depend based on what I want (should make >0 commits per day).

The way I vaguely think about constraints is the following.
The skeleton is just functions -> v2, with the rest sprinkled in wherever.
However, before doing functions, I want to clean up the current codebase, because this is probably the hardest to lex, parse and assemble and each component is.. suboptimal.
So it's more like refactor -> functions -> v2, at least in terms of more necessary readability/simplicity refactors.
The rest can be sprinkled in whenever I don't want to deal with the harder parts, but want to do something.
Or maybe some of them will be desirable anyway (eg. debugging functionalities like printing and improved panics; potentially even comments).



# ROUGH EDGES

- variable overwriting within blocks
- allocating 8 bits per var depspite it's 4 on arm32
- validating called function even exists
- emit should do the format! itself (and probably not take _borrows_, also mega annoying
- keep track of valid variable names in scope, valid funcnames, etc
- check function call correctness (argcount now, typing later)

# Action plan (XII.13.)

We are working on *functions*. We have to finish parsing, then move on to compiling. 
After finishing it, without further ado I plan to create the typing for primitive types (maybe only booleans before we have printing, for strings).
Then, clean up code, and then, do the type system.
Afterwards, we should probably collect our forces, get the whole v1 version together


# v1 plans 

## Easy wins
- Negation
    - introduce Expression::UnaryOp
- Remaining comparison operators (>, !=) 
    - maybe >= and =< but not necessary
- add commenting -- eg //
- add support for signed integers 
    - mostly a lexing issue i think -- need to parse the pattern -[intliteral]


## Less trivial 
- add _print_
    - esp. because echo $? chops off at 256, but generally nice for debugging
- fix the local variable allocation
    - currently a fixed 256 bytes are allocated to variables 
    - this should be fixed by variable counting or whatever


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
