
# Goal
implement structs


## Main problem

- let's target getting initting struct literals and gets on fields right first -- basically the "struct as rvalue" stuff
- field setting later

- could make more efficient by noticing when an expr is already a vreg and not allocating a new one and store !

## Misc notes 
- hashmap.insert() overwrites -- pay more agttention to this everywhere
