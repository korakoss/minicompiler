
# Goal
implement structs

2 phases:
- only primitive-typed fields
- allowing other struct types in fields


## Main problem

- in parsing, we need to pass thru the thing, but struct defs and func defs might be interleaved
    - do we like collect symbols then resolve?

- we could just collect "type identifiers" for _let_ and resolve in HIRgen


## Details
- hashmap.insert() overwrites -- pay more agttention to this everywhere
