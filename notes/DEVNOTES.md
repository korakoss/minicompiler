
# CURRENT THING

## GOAL  
Have generic functions. 
- correctly monomorphized and typechecked


## STEPS

### Current problem (start here) 
Parsing is wrong and it's hard to integrate expecting type parameters for calls to generic functions. 
In *parse.rs: parse_expression_atom()*, we assume that a "[" character starts a generic *structlit*, but now it can start a call to a generic function too. 
So this needs to be sorted out first.
Add test for the parsing!

### Next steps
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
