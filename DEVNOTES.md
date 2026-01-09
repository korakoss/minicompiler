
# Where are we
Structs and MIR done-ish, seems to work, but not fully clear how stably. Refactored the typing so that newtypes are kept symbolic. Did some other cleanups and polish.
Next, the polishing should be finished, and some more tests should be added (potentially also making testing nicer -- eg. being able to test for compilation _failure_).
After everything stabilized, probably start working on enums.


## Finalizations
- remaining naming uniformizations (especially between MIR and LIR)
- function call ABI
    - use pointers for args
    - maybe stack spilling
- implement struct moves
- some (not all) items from INSECTS.md
    - void funccall parsing issue
    - struct literal issues

## Tests to add
- break and continue
- some functions calling each other back and forth

## Things to find out 
- do struct returns and struct arguments work?
    - struct returns do, args don't
- do struct moves work?
    - no


## Possible next steps
- add pointers
- add enums, _match_, and some or a lot of pattern matching
- add an ownership and move checker

## Later
- generics
- possibly struct methods and __scoped__
- heap handling
- then later, provenance types, __above__, the works

## Maybe
- some basic optimizations like dead code etc.
    - but not focus

# Other

## QoL
- Make compilation and test scripts nicer
    - internal stages could be optional
    - test script could call the compile script
- Add informative errors

