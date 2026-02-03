
# NEXT STEP REMINDER
Write the algorithm – in callgraph.rs.


# CURRENT FOCUS: **Generic functions** 

We want to have generic functions, with a sound monomorphization algorithm that can detect "divergent" call cycles that would result in infinitely many monomorphizations.


## Implementation plan

We add a new stage, _CMIR_ (concrete MIR), between the previous MIR and LIR. This stage will be like MIR, but with concrete types instead of generic ones.
Most of the current MIR->LIR logic goes into CMIR->LIR. In the MIR->CMIR pass goes our algorithm for collecting the required monomorphizations of generic functions and types – or detecting divergence.

The MIR-> CMIR pass itself has two stages:

1. Collect all "monomorphization requests" in the form (func ID, type params) or detect a divergent cycle   
2. Actually produce the monomorphized function bodies for the "requests"

To help with Step 1, we'll construct a _call graph_. This is done in the AST->HIR pass.
The graph stores information about what (generic) functions call what other (generic) functions with what type parameters – including potentially their own abstract typevars.

Our algorithm walks this graph, in a DFS manner, starting from main() – which is never generic. 
For the algorithm, we maintain a "call stack" – a path in the call graph we're currently working on – and further "dangling" monomorphization requests for each element of the stack.
An iteration of the algorithm goes like this:
- select the first unprocessed child node of the end of the stack, and push it onto the stack 
- using CallGraph, compute what other monomorphizations it "requests"
- check if one of these requests induce a divergent cycle according to the Pareto criterion, panic (or whatever) if so 
- check if all requested monomorphizations were requested already (by previous stack nodes)
        - if so, mark the current node as "completed" and backtrack on the stack for unprocessed nodes
        - if not, iterate into its children

Repeat until we find a cycle or all nodes are completed.
During this whole process, we should also note down what concrete parametrizations of generic *types* were instantiated. 
(Note: it seems to me that laying them out in LIR won't require the toposorting we used to do – we can simply iterate by rank)


## Next step / Current blocker
The currently muddy area is how _type variables_ should work. Firstly, we need to be able to bind concrete (or maybe even generic) types to them. This is maybe more tractable with some work.
The more difficult question is how we should represent the connections in the CallGraph. 

For example, if we have functions and calls like
```
fun f<A,B,C>(a: A, b:B, c:C) {
    //...
    g<C, B>(c, b);
}
```
we need to be able to somehow sorta say "the third param of f is plugged into the first of g and the second of f into the second of g". But we can't just map them by index – since f is not limited to plugging in its type params, but also any concrete type as well. Which is basically GenericType, I guess? Ehh, it seems a bit confusing, but probably this is the way.
    

## Design questions along the way
- How to represent the type variables in scope and bind concrete types to them ergonomically?

## Things about the current code that I'm unsure about
- Is parsing stable now? Can it parse function type params? In defs and funccalls?
- Are type parameters inside every IR now that they need to be in?
- Is it problematic that Hashmap has nondeterministic iteration order? (This can happen if we do some function argument-related thing with hashmaps)
- Does MIRValueKind::Reference mean the _reference_ or the _dereference_ of its content? Clear this up!

## Miscellaneous TODOs
- the bind/monomorphize namings for substitutions kinda suck
- some checks for various type stuff (eg. checking for the number of type parameters in funccalls or newtype literals)
- continuously refactor. use clippy. do everything you can.
- switch out NewTypeID for an int one (with some mapping to original strings), so we can implement copy for the ID (and consequently for Generic/ConcreteType and so forth
- in make_HIR, we don't issue globally unique IDs to variables! (technically probably fine, but unaesthetic)


# TODOS AFTERWARDS
- add a bunch of Rust tests, esp. for:
    - parsing
    - monomorphization machinery
    - but basically for everything we can
- also add a bunch of Yum tests, esp.:
    - break and continue
    - various struct and pointer ones
        - circular struct defs  
    - some functions calling each other back and forth  
    - functions with many arguments 
    - negative tests (that shouldn't compile)
    - many generic function tests once we have them
        - divergent monos
- fix bugs in INSECTS.md
- make run/test scripts nicer

# OTHER

## Vague design problems that pop up in a flew places:
1. We want to lower some IR. It has interreferences that we are tracking by IDs. (For example, we have blocks in MIR and their terminators refer to other blocks). Then, in the lowering pass, we like to issue new IDs (I think sometimes we need to, as new objects of the given type can be created in the pass -- this definitely happens with cells, I believe). But this leads to this unwieldy mapping problem, as we need to track what old ID was mapped to what new ID and lower correctly.

2. In general, I think the lowering passes are architectured in an awkward pattern. There's typically some kind of builder, we cram a bunch of info into it, it has this weird entry point function that is typically simultaneously a constructor and the lowering function itself. The builders also seem too "global" possibly. For example, in _at least_ later IRs, functions are fairly logically independent of each other, and the lowering should be correspondingly more local, probably.
    - maybe look into Visitor patterns, see if they can help
