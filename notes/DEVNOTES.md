
# CURRENT THING

## GOAL  
Have generic functions. 
- correctly monomorphized and typechecked


## STEPS

### Current problem (start here) 
Write _concretize_mir.rs_. 


### Implementation plan
Iterate through functions recursively, in a DFS manner, like planned. Start from main.

Data we're tracking:
- monomorphizations of functions
    - both the monomorphized bodies and the parameter (or just rank) vectors for a FuncID
- the current "monomorphization" stack
- things to be monomorphized next 
    - sort of as a stack of queues, maybe?
- required monomorphizations of generic types

Algorithm outline:
- start at entry
- pop the next monomorphization "request" off the to-process stack
    - check if that monomorphization already exists, proceed if not
- compute and note down the "rank vector"
- monomorphize the function body
    - go through the blocks in the body
    - keep track of the "typevar binding" we're operating with
    - monomorphize generic types in the blocks we encounter
    - also collect function calls 
        - noting down the typevar bindings 
        - add these to the queue stack thing
        - run the Pareto rank check as planned
- add the monomorphized function body to monomorphizations
- proceed with the queue stack


### Things we need to implement along the way, questions etc.
- Clear up how to represent type variables in scope and bind concrete types to them ergonomically
- Visitor patterns?
- problems with Hashmap nondeterministic order?


### Later steps
- add (Rust-)tests:
    - eg. for the parsing!
- add type params to funccalls in AST too
- weave the type params stuff in all the IRs it concerns (so maybe up to current MIR)
- probably create a ConcreteTypetable
    - topological order determination is sufficient for _that_ one (we need it for laying out newtypes)
    - (!!!) actually, _rank_ does this for us, no? if we simply process concrete types in rank order, we're good
- add CMIR
    - vision:
        - MIR: functions stay generic, probably reuses the same HIR->MIR machinery
- add some check for Type operations (checking the number of type parameters?)


# OTHER

## Cleanup
- continuously try refactoring to make it nicer
- fix bugs on the list
- make running/debug scripts nicer

## Yum tests to add
- break and continue
- various struct and pointer tests
    - circular struct defs
- some functions calling each other back and forth
- "negative" test (that shouldn't compile)

## Rust tests to add
- topo ordering
- monomorphization machinery
