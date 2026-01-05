
# Goal
implement structs


## Main problem

- let's target getting initting struct literals and gets on fields right first -- basically the "struct as rvalue" stuff
- field setting later
- could make more efficient by noticing when an expr is already a vreg and not allocating a new one and store !
- does the current parsing require struct defs to appear "before" making literals? that would suck. maybe not though 

## Misc notes 
- hashmap.insert() overwrites -- pay more agttention to this everywhere


## Bugs

- (i think) the parser currently allows arbitrary expressions as assignment lvalues, which is wrong, it should only be vars and chained field accesses
- none rettype functions are probably handled in a retarded way I think
    - yeah! the issue is that the parser assumes that an stmt starting with an identifier is an LValue expr
- struct literal syntax is kinda wacky, needs trailing commas
    - change the parser


## notes

- we may unify the LIRPlace into a single struct (where Vreg subtype is just a spec case with 0 offset) 
