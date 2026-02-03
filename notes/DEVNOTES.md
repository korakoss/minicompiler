
# NEXT STEP REMINDER
Put the _CallGraph_ definition in some more adequate place (probably it's own file or some utils file, not in IR codes), and write code for constructing it in _make_hir_.


# CURRENT FOCUS NOTES

## GOAL  
We want to have generic functions, with a sound monomorphization algorithm that can detect "divergent" call cycles that would result in infinitely many monomorphizations.

## IMPLEMENTATION OUTLINE
We add a new CMIR stage, and replace the MIR->LIR pass by a MIR->CMIR->LIR sequence. Most of the current MIR->LIR logic would go to the CMIR->LIR pass. In the MIR->CMIR pass, we collect the required concrete monomorphizations (or detect if the monomorphization process would diverge), create those monomorphic functions, and construct the CMIR from these, also collecting the required monomorphizations of generic _types_ in the process.

## NOTES
- Instead of doing the full topological sort on types, it seems sufficient to simply iterate by rank.

## THINKING OUT LOUD ABOUT THE ALGORITHM
We create a call graph. A _CallGraph_ type is currently sketched out in the MIR file, but I think this should be constructed in the AST->HIR pass instead. 
The call graph should enable us to determine that monomorphizing a given function with some concrete type parameters leads to what downstream monomorphizations.

Opposed to some previous plans, I think we should actually first collect all required monomorphizations constructed from this call graph abstractly, then monomorphize the bodies of the collected batch in a separate step. 

I'm not sure about this yet, but this approach maybe also enables us to ditch the DFS approach and just recursively monomorphize the current "leaves" of the "call tree" until we either reach "saturation" or we detect divergent cycling. Hmm no, on second thought, that's not how it works. A given generic function can be called by two different functions in such a way that "one call Pareto-dominates the other" – this doesn't lead to divergence in itself. Okay, so we still need to do this in a DFS way. 

The other difficult part is that we need to track mono requests in two kinds of ways. Firstly, we'll want as output a non-redundant, flat list of required monomorphizations. But – I think – for the tree walk algorithm, we don't want to deduplicate naively (eg. just naively stop when we run into an already-requested mono, like I previously planned). So what do we really need to do?

Okay, here's a naive approach. We grow the tree in a naive and wasteful way, just adding new, potentially redundant nodes. 
This makes it easy to check whether adding a leaf leads to a divergent cycle or not. 
So, in each step, we choose a leaf (in some kind of DFS manner, I suppose), and do one iteration of generating its children. If all children of it are redundant, we mark the node as "finished" and backtrack to other parts of the tree left to process. If there are "new" children, we add them to the stack-queue-thingy and iterate into them. Eventually we either find a cycle downstream, or complete the descendants of the node, in which case we also mark the node as "finished" and pivot elsewhere. And so on and so on.


## ALGROTHM WRITEUP (naive, possibly can be optimized later)

The MIR->CMIR pass has two stages:

1. Collect all monomorphizations (just in the form of function ID + type parameters) we'll need to make (or detect a divergent cycle that makes this impossible).
2. Actually produce the monomorphized functions for the "requests" we collected in Step 1.

Step 2 is fairly trivial, so let's focus on how to do Step 1. Firstly, there is a call graph, that stores what functions call what other functions and what type variables they plug into them.
This call graph will be probably constructed in HIR. 
Using this graph, we can determine, given a generic function and some concrete type parameters, the monomorphized callees corresponding to that caller. 
We'll walk this graph with a DFS algorithm, starting from _main_ (which is never generic). 
At each step, we'll have a "call stack" that we're currently exploring and further "dangling" monomorphization requests associated with each other. 
In an iteration, we select the first unprocessed child of the current end of the stack for processing. Using _CallGraph_, we see what monomorphizations it requests.
We check two things about these monomorphizations:
- First, we check the Pareto criterion to see if we detect a cycle. We panic (or whatever) if so.
- Secondly, we check whether all the new monomorphizations were already requested. If so, we mark the node as completed and backtrack in the stack. Otherwise we iterate into its children.


## Design questions along the way
- How to represent the type variables in scope and bind concrete types to them ergonomically?

## Things about the current code that I'm unsure about
- Is parsing stable now? Can it parse function type params? In defs and funccalls?
- Are type parameters inside every IR now that they need to be in?
- Is it problematic that Hashmap has nondeterministic iteration order? (This can happen if we do some function argument-related thing with hashmaps)
- Does MIRValueKind::Reference mean the _reference_ or the _dereference_ of its content? Clear this up!

## Miscellaneous TODOs
- some checks for various type stuff (eg. checking for the number of type parameters in funccalls or newtype literals)
- continuously refactor. use clippy. do everything you can.
- switch out NewTypeID for an int one (with some mapping to original strings), so we can implement copy for the ID (and consequently for Generic/ConcreteType and so forth
- in make_HIR, we don't issue globally unique IDs to variables! (technically probably fine, but unaesthetic)


## Miscellaneous notes
- maybe look into visitor patterns and whether they can help with better design for the lowering passes?


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

