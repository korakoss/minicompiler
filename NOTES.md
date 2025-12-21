# Current thing
- HIR->Assembly codegen

# Typechecking gaps
- returns in func bodies having the correct type
- assign values having the type of the target
- LATER: prints
    - not all types might be naively printable
    - we want bool prints to print True/False not 1/0, etc

# Variable counting
- we now have enough info to allocate the correct amount of space actually needed by variables

# Rough edges & QoL improvement areas
- allocating 8 bits per var depspite it's 4 on arm32
- print tokenization and AST human-readably
	- we can make a simple recursive function to display it with indents?
- nicer scripts
    - flags for displaying/saving each pipeline stage
    - uniformize the result names: $.yum turns into $.everythingelse
- add syntax highlighting
    - vim script / treesitter

# Easy but unnecessary wins
- Negation
    - introduce Expression::UnaryOp
- Remaining comparison operators (>, !=) 
    - maybe >= and =< but not necessary
- add commenting -- eg //
- add support for signed integers 
    - mostly a lexing issue i think -- need to parse the pattern -[intliteral]


# Next progess steps

## More types

### Structs
- just make normal struct stuff (I guess with int and bool -- and struct -- fields)

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


