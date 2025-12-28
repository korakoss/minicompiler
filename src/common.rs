use std::collections::BTreeMap;


#[derive(Debug, Clone)]
pub struct Variable<T> {
    pub name: String,
    pub typ: T,
    pub size: usize,
    // TODO: mutable, etc
}

#[derive(PartialEq,Eq, Debug, Hash, Clone)]
pub enum Type {
    Integer,
    Bool,
    None,
    NewType(NewType<Type>),
}

#[derive(Debug, Clone)]
pub enum DeferredType {
    Resolved(Type),
    Unresolved(TypeIdentifier)
}

#[derive(PartialEq,Eq, Debug, Hash, Clone)]
pub enum NewType<T> {
    Struct {
        fields: BTreeMap<String, T>,
    }
}


#[derive(Debug, Clone)]
pub enum BinaryOperator {
    Add, 
    Sub, 
    Mul, 
    Equals,
    Less,       // NOTE: represents left < right 
    Modulo

    // TODO
        //Greater, 
        //Div (later, when floats ig?),
        //NotEqual
}

// TODO: eventually also UnaryOperation (eg. negation)



#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FuncSignature {
    pub name: String,
    pub argtypes: Vec<Type>,
    // NOTE: maybe return type sometime?
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct TypeIdentifier(pub String); 





pub fn binop_typecheck(op: &BinaryOperator, left_type: &Type, right_type: &Type) -> Option<Type> {
    
    match op {
        &BinaryOperator::Add | &BinaryOperator::Sub | &BinaryOperator::Mul| &BinaryOperator::Modulo=>{
            if left_type == &Type::Integer && right_type == &Type::Integer {
                Some(Type::Integer)
            } else {
                None
            }
        }
        &BinaryOperator::Equals => {
            if left_type == right_type {
                // TODO: careful later
                Some(Type::Bool)
            } else {
                None
            }
        }
        &BinaryOperator::Less => {
            if left_type == &Type::Integer && right_type == &Type::Integer {
                Some(Type::Bool)
            } else {
                None
            }
        } 
    }
}


