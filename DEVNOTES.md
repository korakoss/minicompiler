
# Roadmap 

## Current-ish things
- mir for ownership checking
- enums and match and pattern matching
- clean everything so far (especially structs, lir etc)

## Right now doing
Do HIR->MIR, possibly MIR->LIR (latter maybe more thinky).
Or actually also sketch planned MIR internals (the ownership checker, potentially the provenance type machinery).

## Afterwards
- MIR and pattern matching are fairly orthogonal and both interesting -- figure out the order
- probably a large clean of everything after all these (especially structs, lir etc)

## After that 

### Collecting
- generics?
- actually, the innovations can start to trickle in
    - scope-type dispatch could like, right now
        - probably get a good MIR though first
    - __above__ needs borrow checking and will probably be _harder_
- eventually heap
    - but it'd be funny to do it really late, possibly aftere borrow and above
        - tbf natural above usages involve vecs or sth though
        - get it working with a stupid bumper
            - own allocator only after the innovations stabilize

### Ordering
- generics needed for kosher vecs, etc. so those first
- then mayyyyyyybe the __scoped__ kw for proof of concept
    - dunno whether wiser in tandem with above tbh
- dump heap stuff, vecs, string probably


# Other

## Design questions

- original subtyp layout monomorphization ideas probably don't work
    - maybe salvageable as explicit extending with diamonds banned, and no promised enum monomorph 
        - though then what's even the point? funcs?
            - so I think there's something around scopedispatch here, we should probably only deploy to explicit subtypes of scope annot
                - unless wacky row mono features, but let's forget that

## Misc notes 
- hashmap.insert() overwrites -- pay more agttention to this everywhere
- hashmaps nondeterministic -- not good for multiple reasons

## Bugs

- none rettype functions are probably handled in a retarded way I think
    - yeah! the issue is that the parser assumes that an stmt starting with an identifier is an LValue expr
    - also, it's not checked whether functions return properly
        - though this is a MIR concern
- struct literal syntax is kinda wacky, needs trailing commas
    - change the parser
- the main func deciding logic is probably wacky too
    - but what to do?
- we cannot move >8 size values !!! -> cf. move checker
    - probably actually fine for now in most cases given the ABI and how structlits are done currently
    - later the ownersh checker rules block this mostly
    - but write the large mover routines too, probably not even that hard
- no typecheck for print
    - and booleans improperly printed

## notes

- we may unify the LIRPlace into a single struct (where Vreg subtype is just a spec case with 0 offset) ?

- ABI: when we implement __scoped__, we could have a scope pointer register

## Other TODOs

- rethink function calling 
    - returns are r12 pointers, okay
    - but we might also want to use pointers for args
- eventually get rid of all the cloning
- eventually we want more informative errors


## Enum syntax notes


```
enum T {
    S,
    R,
}

let t:T = T::new(); //eg

match t {
    S => {...}  // t implicitly typecast to t:S
    R => {...}
}
```

also namespaced

## MIR
sneak in the calculation kind in temps?
