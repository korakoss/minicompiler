
> Currently trying to add generics. 
If that finishes, clean up a bit, especially the infrastructure around running. Add more test programs, and also add Rust unit tests where possible. Improve documentation as well


# Generics
Typing, parsing and AST maybe done. HIR and HIR lowering are next. We need to decide where to monomorphize. And I need a better sense of what needs to be done.

## Scope
In this first run, we are basically just making types generic, not functions. That is, we want to define a generic struct and then, make a concrete instance, and operate on it. 

## Next step
Update HIR to use the new typing system (still having generic types). Extend the type checking, and add resolution of type vars in scopes. 


## Design ideas

I think we want to monomorphize in MIR->LIR. For futureproofing reasons, when MIR would be heavily analyzed, and we want to make that cheaper by staying generic. 
So then, the roles of various stages with respect to generics are:
- AST: just represent the program, collecting type variables, newtype identifiers, and so on
- HIR: scoped resolution of type symbols. 
    - understand whether a symbol is a newtype identifier or a type variable, detect potential clashes. 
    - scope-aware binding of type variable symbols. Probably rename the current String ID to TypeVarSymbol (in AST), then assign genuinely unique IDs to a given symbol per scope (in HIR).
    - extend the topo ordering / circularity detection machinery to handle generics
- MIR: probably just passing through, for now
- LIR: monomorphization


# Others

## Cleanup
- fix bugs on the list
- make running/debug scripts nicer

## Yum tests to add
- break and continue
- various struct and pointer tests
- some functions calling each other back and forth
- "negative" test (that shouldn't compile)

## Rust tests to add
- topo ordering
- monomorphization machinery
