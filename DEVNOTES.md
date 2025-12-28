
# Goal
implement structs


## Main problem
- we need a central newstruct collection in HIR too, to determine layouts uniformly
- but currently, that's resolved away in HIR-gen to just have expressions with type info individually encapsulated

- POTENTIAL SOLUTION: 
    - in Type, don't do the current _Derived_ thing, but instead just make it 
        - Type::NewType(TypeIdentifier) 
        - (common::)NewType
            - basically the same as the current DerivedType thing, but "bare"
            - it's IRs that make the mapping between TypeId-s and NewType-s
    - this seems good, implement this

- then just decide on some random uniform layout and do the codegen. boom.
- probably parsing needs to be rolled back to directly parsing out primitive types            

## Details
- hashmap.insert() overwrites -- pay more agttention to this everywhere
