
# CURRENT FOCUS: **Generic functions** 

I think I largely finished the CMIR addition stuff. Yum test cases run again. However, there are issues
- running gen1.yum printed 7, which would mean that struct fields are scrambled
- I wrote a small generic function program, gen2.yum, which fails to compile

So the whole thing is not actually sound yet. Debug and fxix, then proceed with the rest of the todos.


## Things about the current code that I'm unsure about
- Is parsing stable now? Can it parse function type params? In defs and funccalls?
- Are type parameters inside every IR now that they need to be in?
- Is it problematic that Hashmap has nondeterministic iteration order? (This can happen if we do some function argument-related thing with hashmaps)

## Miscellaneous TODOs
- the bind/monomorphize namings for substitutions kinda suck
- some checks for various type stuff (eg. checking for the number of type parameters in funccalls or newtype literals)
- continuously refactor. use clippy. do everything you can.
- switch out NewTypeID for an int one (with some mapping to original strings), so we can implement copy for the ID (and consequently for Generic/ConcreteType and so forth
- in make_HIR, we don't issue globally unique IDs to variables! (technically probably fine, but unaesthetic)
- we could create various new ID _types_ wherever we reassign IDs
    - eg separate block/cell ID types for IR/CMIR
    - reason: the old->new mappings and so forth are kinda confusing currently, that the type is the same
- we should rename some things probably 
    - instead of MIR/CMIR, have GMIR/CMIR at least, or sth even better
    - change some pass names, eg. make_hir and concretize_mir
    - and some file-internal renamings across te boards too of course
    - does MIRValueKind::Reference mean the _reference_ or the _dereference_ of its content? 
        - *Clear this up!*
    - sometimes a field storing a [thing]ID type is called [thing], which is... not sure 


## Showerthoughts
- can the two MIRs be made generic?
- type IDs, so they can implement copy  
    - currently the two Type types can't, because the Newtype variants aren't sized
- store struct literals as Vec<fieldname, fieldtype>
    - corresponds to fixed layout, could be nice
- collect what _new types_ do monomorphizations of certain generic functions induce, collect and produce them in one pass (in MIR->CMIR) after collecting func monos
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
- the literal //TODO-s across the code

# OTHER

## Vague design problems that pop up in a flew places:
1. We want to lower some IR. It has interreferences that we are tracking by IDs. (For example, we have blocks in MIR and their terminators refer to other blocks). Then, in the lowering pass, we like to issue new IDs (I think sometimes we need to, as new objects of the given type can be created in the pass -- this definitely happens with cells, I believe). But this leads to this unwieldy mapping problem, as we need to track what old ID was mapped to what new ID and lower correctly.

2. In general, I think the lowering passes are architectured in an awkward pattern. There's typically some kind of builder, we cram a bunch of info into it, it has this weird entry point function that is typically simultaneously a constructor and the lowering function itself. The builders also seem too "global" possibly. For example, in _at least_ later IRs, functions are fairly logically independent of each other, and the lowering should be correspondingly more local, probably.
    - maybe look into Visitor patterns, see if they can help
