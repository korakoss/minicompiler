
# Where are we

Added structs and the MIR. Seems mostly functional, except passing struct args.
Then I added pointers, which also seem to work, even convoluted programs now.
But neiter structs, nor pointers were meticulously tested.

I originally started to add pointers to enable passing function arguments as caller frame pointers. This is still not implemented however -- this should be next.
After that, finalization and cleanup.
Also, more tests should be added (some are collected below). It'd also be nice to improve the test script. In particular, enabling "negative testing": tests for compilation _failures_.


## Finalizations
- some naming cleanup/uniformization (especially between MIR and LIR, I think)
- some items from INSECTS.md
    - actually, doing most from the Bugs/issues section would be nice

## Tests to add
- break and continue
- some functions calling each other back and forth



# Roadmap

After pointers are completed, I think we should start working towards heaped things. 
The reason for this is that I expect those to fuck up several IRs (by introducing indexing "projections"), and it'd be nice to have the IRs stabilize for the most part.

For heaped stuff however, we first need to add generics. So that'll be the next big step. 

After having generics, write some kind of dumb bump allocator (in C?) to use, then implement Vec<T>. 

After Vec<T> is implemented, start working on enums. Implement basic matching for them.

If all of these are completed, I think that's sort of the language core finished. Then, there are several, fairly orthogonal "rabbitholes" that we can start to go towards:
- more sophisticated pattern matching
- more sophisticated memory allocator
- adding a borrow checker or at least an ownership/move checker
    - and mutability?
- adding an x86 backend
- adding optimizations
    - dead code elimination
    - register allocation
    - etc., see materials
- cosmetic improvements on the compiler (better scripts, informative errors, etc.)

The _scoped_ keyword can also be experimented with at this point (and methods in general should be added around here, with whatever implementation).
The _above_ keyword needs the borrow checker.

If I don't want to continue the project at some point, I think it'd be nice to still add an x86 backend for the existing parts, and do the "niceness" parts (errors, better pipeline). 
And it'd be nice to get it to at least the generics+heap+enums stage. Also, it might be a good idea to do this polish anyway after those features.

If still continuing after those, I think the better pattern matching could be the next step. After that, maybe move checking/mutability. Then better malloc or borrow checking, whatever seems interesting.


# Other

## QoL
- Make compilation and test scripts nicer
    - internal stages could be optional
    - test script could call the compile script
    - some script that jumps over to Pi and compiles there in one go
    - put the yumpi/yute scripts here, not in dotfs
- Add informative errors


## Someday
- basic optimizations like dead code elimination
