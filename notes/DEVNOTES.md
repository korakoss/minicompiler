
> Currently trying to add generics. 
If that finishes, clean up a bit, especially the infrastructure around running. Add more test programs, and also add Rust unit tests where possible. Improve documentation as well

> (I.21) I think it might be finishing up. Weaved ConcreteType though most of the layers. The codebase compiles but fails on tests. The reason is an innocent change in parsing, where I got rid of branching on whether an identifier was a newtype or not (so we can have more flexible ordering of typedefs in code). But this leads to a parsing problem – for example it seems to me it parses some 
```[identifier] {```
pattern (which is meant to start a code block) as a struct literal. Solving this requires a lookahead parser. I think I should instead revert the relevant parsing for now, so we can see what's up with generics, debug, finish. Also, I did *concrete* types in the lower layers. Which is fine for the moment, not having generic funcs, but we should add those next.


> *!!!* Okay, generic typing seems to work. Now, onto generic functions. The plan is, first weave things through the IRs (mostly MIR left, hopefully). I want to first just achieve that, don't think about function monomorphization yet, just wire up a quick "monomorphization" in MIR->LIR (or someplace) that assumes functions are not generic yet. Test it that way. Then, wire up the monomorphization.


# Generics
Typing, parsing and AST maybe done. HIR and HIR lowering are next. We need to decide where to monomorphize. And I need a better sense of what needs to be done.

## Scope
In this first run, we are basically just making types generic, not functions. That is, we want to define a generic struct and then, make a concrete instance, and operate on it. 

## Next step
Update HIR to use the new typing system (still having generic types). Extend the type checking, and add resolution of type vars in scopes. 

## Current problem
I'm a bit lost in the dichotomy of "generic vs concrete" types. So in the current version, we're not planning to have generic functions yet. What that means is: type variables exist in the the scope of struct definitions, and within function bodies, it has to work out to a concrete type. 

And I guess that's exactly what GenericType and ConcreteType are meant for. Within a struct body, you can refer to the typevars, but outside, in functions, you mut put concrete types. So I suppose – at least in the current sprint – we must write two separate _parsers_, one accepting type variables and one not. Though, on second thought, this should probably be kept around for the later version, or something similar this, since we do want to get concrete types out of nongeneric functions (like, notably, the entry point _main_). 

So the current task is maybe easier than I thought. We have immediate, on the spot monomorphization when we declare variables. 


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
