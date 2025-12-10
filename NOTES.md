Things to do in the next few weeks


# Next steps (XII.10 prolly)
- add _continue_


# Prob easy but not that necessary
- add commenting -- eg //
- add __print__
    - esp. because echo $? chops off at 256, but generally nice for debugging
- add support for signed integers 
    - mostly a lexing issue i think -- need to parse the pattern -[intliteral]


# Cleaning up before the storm
- make the code nicer
    - see comments in code
    - lots of things should be hashmaps or sth like that
    - some things might not need to be boxed
        - exprs in stmts maybe?
    - just generally do another pass, I've gotten the hang of Rust more since writing some of it
- fix the local variable allocation
    - currently a fixed 256 bytes are allocated to variables 
    - this should be fixed by variable counting or whatever


# Functions
- planned v1 syntax: 
    > fun fname(arg_1, ..., arg_n) {}
- of course everything is implicitly typed integer now
- _return_, probably
    - eventually a trailing expr return would be nice but that can wait
- milestone: we want "sandbox/dream_program.txt" code or something like that running


And then v2.



# v2 plans

## Phase 1: Typing 
- start off with primitive types _int_ and _bool_
- and perhaps _None_? that could be nice for eg having _Option_-s early
- add _struct_-s 
- add union types (especially if None was already implemented)
- add _enum_-s
- add _mut_

