
# Where are we
Structs and MIR done-ish, seems to work, but unclear how stable. Test it, clean up everything.

## Current cleanup
- (see INSECTS.md; not all are urgent, but some)
- finalize function call ABI
    - pointers for args!
- uniformize the design and namings (esp. between MIR and LIR)

## Possible next steps
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

