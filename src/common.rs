
#[derive(PartialEq,Eq, Debug, Hash, Clone)]
pub enum Type {
    Integer,
    Bool,
}

#[derive(Debug, Clone)]
pub struct VariableInfo {
    pub name: String,
    pub typ: Type,
    // TODO: mutable, etc
}

