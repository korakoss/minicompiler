



pub enum Type {
    Prim(PrimType),
    NewType(TypeIdentifier),
}


#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum TypeConstructor{
    Struct {
        fields: BTreeMap<String, Type>
    },
    Enum {
        variants: Vec<Type>
    }
}



#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct TypeIdentifier(pub String); 

pub struct TypeTable {
    pub topo_order: Vec<TypeIdentifier>,
    pub newtype_map: HashMap<TypeIdentifier, Type>,
}

pub enum PrimType {
    Integer,
    Bool,
    None,
}

