
# Current status & recent past 

Added structs and pointers, currently working on generics. 
Testing has been improved too, now enabling "negative" tests and overall more convenience. 
There are other branches open for working on the pointer-based ABI rewrite as well as adding x86 backends. There is a collection of known current bugs. 


# Next steps

The items below are roughly in planned chronological order of implementation. Some reasoning for this order.
So, first, we do only generic types and then concretely instantiated variables. This is a much more tractable problem than monomorphizing functions, but lets us test a lot of the infrastructure for generics. Much of the parsing is the same, the type system is the same. 
Then, we add generic functions. This adds some complication to tracking the scoping of the type variables, and then also in lower stages, where we now have to monomorphize functions too.
Then, we start preparation for infras, by implementing enums and methods. It's clear why methods should be implemented before infras: the point of infras are the inheritance of methods to subtypes. Enums, on the other hand, are helpful for infras (I think), because both are basically concerned with subtyping in different forms, so if we work out some kind of subtyping infrastructure in the easier case of enums, that can set us up for infras to some degree.  
Finally, we add heap. This must clearly come after generics, generic functions and methods, if we want to do them in a correctly typed manner. They are, a priori, fairly interchangeable with infras (and potentially even enums), but I want to do those first because they seem more disruptive to the overall pipeline.


## Finish generics
The current, v1 implementation with no bounds on type variables, etc. Further, we are only making types generic, not functions yet. Propagate generics through the AST and (some) IRs, decide where and how to monomorphize. 

## Generic functions
Type variables in function definitions, monomorphization of functions, etc. 

## Enums
Implement the basics of enums, simple summing. No namespacing complications yet. Basic matching for them.

## Methods
Add methods callable on types. Decide whether we want _self_ on concrete types, or omit it like on infras. Implement generic functions here if not already.

## Infras
Implement infras and the basic conformance model. 

## Heap
Implement Vec<T>. Use a bump allocator, _malloc_, or roll own malloc. 



# Loose ends

## Bug fixes
See INSECTS.md. Most of the _Bugs/issues_ sections should be fixed.

## Argument-passing ABI
Support arbitrary number of arguments. Implement the planned ABI which passes a pointer into the caller frame. Or something.
Currently planned ABI:
    - each function determines a "callee layout" -- basically offsets in a struct-like memory chunk that they expect argument info in
    - this "struct" will contain the actual pointers to argument values (in the caller frame or wherever)
    - the caller then passesa single pointer to this "struct"
    - the callee chases down the pointers to get the real argument values


# Later steps (substantive)

## Type inference

## More sophisticated pattern-matching
Implement "matching out the fields", like "S{a,b} => {//..}". Implement nested versions for this. And so on. At each step, maintain exhaustiveness (and overlap?) checking.

## Sophisticated memory allocator

## Borrow checking or something
Add mutability/immutability. Make the type system affine (tracking moves/ownership). Then eventually, add borrow checking.

## The _above_ keyword, the self-referentiality stuff
Needs the borrow checker.


# Later steps (professionalness)

## More backends
Add x86 backends. Figure out alignment stuff and what else needs to be generalized in the current pipeline. 

## Compiler usability
Improve running and testing scripts (sort out the "internal stage dump" approach). Add more informative errors.

## Classic optimization
Dead code elimination, register allocation, constant propagation, the stuff.


# Notes on quitting 

If I don't want to continue the project at some point, I think it'd be nice to still add an x86 backend for the existing parts, and do the "niceness" parts (errors, better pipeline). 
And it'd be nice to get it to at least the "generics + heap + enums" stage. Also, it might be a good idea to do this polish anyway after those features.


