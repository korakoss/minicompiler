
# Generics syntax

We use Generic[T: Bound] for type vars, to avoid the turbofish, parsing ambiguities, etc.


# Enum syntax notes

Enums should be free sum types, eg.:

```
struct S {...}

struct T {...}

enum T {
    S,
    R,
}
```

Not exactly sure how matching should work, maybe it should be the narrowed type in the given arm:
```
match t {
    S => {...}  // t implicitly typecast to t:S
    R => {...}
}
```

We should also allow namespaced enums, like

```
enum T {
    ::S {a: int, b: bool},
    R
}
```



# OPEN QUESTIONS
- how should the trait/interface/whatever system work?



# NOTES

- original subtyp layout monomorphization ideas probably don't work
    - maybe salvageable as explicit extending with diamonds banned, and no promised enum monomorph 
        - though then what's even the point? funcs?
            - so I think there's something around scopedispatch here, we should probably only deploy to explicit subtypes of scope annot
                - unless wacky row poly features, but let's forget that
- ABI: when we implement __scoped__, we could have a scope pointer register

# LANGUAGE WRITEUP
[TBD]
