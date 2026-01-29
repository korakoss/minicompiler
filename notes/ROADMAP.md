
# Current status & recent past 

Added structs and pointers, currently working on generics. 
Generic types are implemented, but generic functions aren't yet.
Testing has been improved too, now enabling "negative" tests and overall more convenience. 
There are other branches open for working on the pointer-based ABI rewrite as well as adding x86 backends. There is a collection of known current bugs. 


# Next steps

In roughly chronological order, the short/medium-term next steps:
- generic functions
- function return typecheck
- lookahead parser
- enums and matching
    - simplest summing, no namespacing complications
    - simple matching, just handling subtypes
- infras
- methods
- heap
    - goal: Vec<T> working. roll an allocator


# Short-medium term generic function notes

## Function return typechecking
We might do this in AST->HIR or some intermediate. 
It might be set up by as a check on statement blocks. Each block has a type it evaluates to.


## Monomorphization
In two separate stages:
- *semimorphization*: collecting top functions (those called by main), and then symbolically "monomorphizing" with their typevars. This is where divergence would be found too
    - should happen between HIR (needs types everywhere) and MIR (shouldn't be CFG probably)
        - probably just insert a new stage in between
        - it's basically like HIR, but type params are mostly concrete except main callees
    - requires building a "call graph"
- *monomorphization*: then just stamping in concrete types from main
    - we need to be able to do that easily with a top-down recursion. how?


## Open questions 
- do we need topo ordering? when? laying out?
- add a NewType struct?

## Deferred for a bit later 
Implement func return typechecks.



# Orthogonal-ish improvements to do sometime:
- merge, sort structure out
- bug fixes (from INSECTS.md and so on)
- ABI on argument-passing
    - tentative plan:
        - functions (monomorphized concrete variants, in LIR) determine an "argument layout"
            - basically a uniform mapping of arguments to offsets in some chunk
            - at a given slot, it may contain values or pointers (uniformly for a slot)
        - the caller constructs such a chunk in its frame and passes a pointer to it
        - the callee then reads it, chases pointers, etc. to get the argument values
- x86 backend
- handling compiler errors nicely and informatively, improving compile/testing scripts, etc


# Later projects

These are larger, often fairly mutually orthogonal project to do after the language core is finished:
- type inference
    - probably limited to intra-function-bodies, explicit signatures required
- sophisticated pattern-matching
    - eg Rust-style deconstruction, doing it recursively, etc.
    - at each step, maintain exhaustiveness and overlap checking
- rolling my own memory allocator in a sophisticated way
- classic optimizations (DCE, register allocation, constant propagation)
- affine typing
    - ownership/move checking first
    - mutability
    - borrow checking eventually
- self-referentiality, _above_, etc.
    - dependent on borrow checking


# Notes on quitting 

If I don't want to continue the project at some point, I think it'd be nice to still add an x86 backend for the existing parts, and do the "niceness" parts (errors, better pipeline). 
And it'd be nice to get it to at least the "generics + heap + enums" stage. Also, it might be a good idea to do this polish anyway after those features.


