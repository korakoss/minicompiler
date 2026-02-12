
# Status
Finished generic functions. Or at least some text programs using them can compile and run.


# Next topics

There should be a lot of polishing before starting something substantial (likely return type checking next?) again. 
I think most of the following should be cleared:

## A minimal CLI
Use the _clap_ crate for example. The goal is to replace with code the current messy collection of scripts – run scripts, test script, various things.
A simple command with a number of subcommands and potentially arguments, like:
    - _yum build [filepath]_ for making an exe    
    - _yum run [filepath]_ for build and run  
    - _yum test_ for running the test suite
We should have flags for things like whether the IRs should be saved and so forth.

## Much more tests
We should add Rust unit tests wherever possible. It'd be particularly important for parsing and the monomorphization machinery, I think, but generally anywhere too.
We should also add a bunch of Yum tests as wel, especially:
    - break/continue    
    - edge cases involving structs, pointers, or generics
        - for example, circular struct definitions
    - functions with too many arguments for the current ABI
    - functions calling each other back and forth   

- clean up in the code  
    - example: _HIR->MIR_ pass still seems messy
- write more tests
- fix bugs

## Polishing


# Old
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
    - well, it's probably too much indirection for generic types which we want to use semantically a bunch of times, but could work for concrete types

- fix bugs in INSECTS.md
- the literal //TODO-s across the code

# OTHER

## Vague design problems that pop up in a flew places:
1. We want to lower some IR. It has interreferences that we are tracking by IDs. (For example, we have blocks in MIR and their terminators refer to other blocks). Then, in the lowering pass, we like to issue new IDs (I think sometimes we need to, as new objects of the given type can be created in the pass -- this definitely happens with cells, I believe). But this leads to this unwieldy mapping problem, as we need to track what old ID was mapped to what new ID and lower correctly.

2. In general, I think the lowering passes are architectured in an awkward pattern. There's typically some kind of builder, we cram a bunch of info into it, it has this weird entry point function that is typically simultaneously a constructor and the lowering function itself. The builders also seem too "global" possibly. For example, in _at least_ later IRs, functions are fairly logically independent of each other, and the lowering should be correspondingly more local, probably.
    - maybe look into Visitor patterns, see if they can help

## Good practices discovered but not implemented everywhere
- drain()



# TODOs
- Get rid of GenTypeVariable, just store types?

