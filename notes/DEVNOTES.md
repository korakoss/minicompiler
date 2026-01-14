
# Where are we

Added structs and the MIR. Seems mostly functional, except passing struct args.
Then I added pointers, which also seem to work, even convoluted programs now.
But neiter structs, nor pointers were meticulously tested.

I improved the running and testing scripts a lot. Now the testing script can also test for compilation failure, which should be made use of.

I improved the running and testing scripts a lot. Now the testing script can also test for compilation failure, which should be made use of.

I originally started to add pointers to enable passing function arguments as caller frame pointers. This is still not implemented however -- this should be next.
After that, finalization and cleanup.
Also, more tests should be added (some are collected below). It'd also be nice to improve the test script. In particular, enabling "negative testing": tests for compilation _failures_.

*NOTE:* as I just discovered, struct returns actually don't work, the previous seemingly functional example probably just lucked out by querying the 0-offset field. A more through code fail. See strucret.yum.

Maybe it's redundant in LIR that both places and values has a _size_. Or in upper IRs, the same with types. Dunno.

## Plans for the caller frame pointer thing
- each function determines a "callee layout" -- basically offsets in a struct-like memory chunk that they expect argument info in
- this "struct" will contain the actual pointers to argument values (in the caller frame or wherever)
- the caller then passesa single pointer to this "struct"
- the callee chases down the pointers to get the real argument values
- use r4 for the arg struct pointer and r5 for the return pointer 
    - reminder: 
        r0 -> main operational register 
        r1 -> helper register (eg for binops)
        r2 -> used in modulo, derefs
        r3 -> used in derefs
- okay, so at which point does this happen? *MIR->LIR* or *LIR->asm*?
    - what do we need to modify relative to the source Yum?
        - we need to add the creation of the **argtable** for the call
            - okay wait, does this work for any call? if the function calls another function in a loop or something?
                - i think so, yeah, at least in our current basic model. the code cannot hit a call again, in a given stack frame, without having finished the original call
                - and what about the output value? well, that does get overwritten, but that maps onto how the source uses that value (or rather, place) –– if someone needed it, they yanked it already
        - so probably the easiest way to hook this up is: we leave the functions as they are now and add synthetic scaffolding
            - in the current model, the functions basically assume that they have their arg values in local cells
            - we want to transform each function, by LIR, to have signature (&[argtable_type], &[ret_typ]) -> void. 
                - first, we create the struct type representing the argtable, which has fields of type &argtype_n and some synthetic labels or whatever
                - next, in MIR->LIR, we make the following changes to functions:
                    - callee-role modifications
                        - as input, only a single pointer argument is expected (pointing to a struct of type(argtable)) obviously
                        - the derefs of the fields of that struct are copied into the argument cells of the LIRFunction (the ones it currently uses to handle arguments)
                            - wait not, you can maybe overwrite them in the cell mapping? that would be neat!
                        - we switch return statements to a statement that loads the return value into the deref of the return pointer
                    - caller-role modifications
                        - each funccall expression is modified
                            - we add an argtable cell and fill it with the right refs
                            - we make a pointer to that argtable cell
                            - we make a pointer to the cell where we want the return value
                            - we pass the argtable and the return pointer to the (transformed) callee


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
- Add informative errors
