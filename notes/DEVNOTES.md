
# CURRENT THING

## GOAL  
Have generic functions. 
- correctly monomorphized and typechecked


## STEPS

### Current problem (start here) 
Write _concretize_mir.rs_. 

### Plan notes (concretize_mir)
We want to do the following things in this pass:
- lower from MIR to CMIR – mostly concretizing types, I guess

Okay, but how do we do that? Here's where the call graph comes in, I think. 
We'll start from main and recursively monomorphize from there. 

We want to register newtype monomorphizations so we'll know what needs to be laid out for LIR.

### Plan details
Start from main. It has no type params. We go through the body of the function, scanning for function calls. Since main isn't generic, all calls in it are parametrized by concrete types. 
We iterate through the calls in the body, in a DFS manner. We also note down all newtype monomorphizations we encounter.

Probably, the right flow in a step of the algorithm looks something like this:
- (context):
    - we have a "monomorphization stack" – basically a callstack in the DFS iteration we're doing, consisting of monomorphized functions calling each other
    - we maintain some kind of "queue" to monomorphize next (function + typevars)
- at a given iteration, we take the next item of the queue
- we just create the monomorphization of the function all the way through 
    - we note down all the function calls made in it. that will be the new queue/stack to process
        - figure out the right data structure here. it has to be some kind of stack-ish things, we want to process "deeper" entries first, in line with DFS
    - we also note down monomorphized types in the process
        - careful not to duplicate
- we add the function to the "processed" stack
    - we probably calculate the Pareto rank vector thing for the halting criterion
- then we continue with the "processable" stack/queue

### Breakdown

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
