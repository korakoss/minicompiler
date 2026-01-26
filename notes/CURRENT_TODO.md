
# GOAL  

Have generic functions. 

Desiderata:
- type checked
    - NOTE: we don't have function return checks (does it always return on any execution path, with the right type?). So should we add that before doing anything further with generics?
- competent monomorphization
    - only making the necessary monomorphizations
    - finding when that would erroneously diverge


# STEPS

- implement generiticity rank calculation
- add some check for Type operations (checking the number of type parameters?)

Plan after: MIR-> MMIR (monomorphic MIR) -> LIR. 
In the meantime, make refactors and cleanups on the current codebase.


# IMPLEMENTATION NOTES

# Function return typechecking
We might do this in AST->HIR or some intermediate. 
It might be set up by as a check on statement blocks. Each block has a type it evaluates to.


# Monomorphization

In two separate stages:
- *semimorphization*: collecting top functions (those called by main), and then symbolically "monomorphizing" with their typevars. This is where divergence would be found too
    - should happen between HIR (needs types everywhere) and MIR (shouldn't be CFG probably)
        - probably just insert a new stage in between
        - it's basically like HIR, but type params are mostly concrete except main callees
    - requires building a "call graph"
- *monomorphization*: then just stamping in concrete types from main
    - we need to be able to do that easily with a top-down recursion. how?


# UNCLEAR STUFF
- do we need topo ordering? when? laying out?
- add a NewType struct?



# DEFERRED LATER

Implement func return typechecks.
