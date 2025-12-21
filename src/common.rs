
#[derive(PartialEq,Eq, Debug)]
pub enum Type {
    Integer,
    Bool,
}

#[derive(Debug)]
pub struct VariableInfo {
    pub name: String,
    pub typ: Type,
    // TODO: mutable, etc
}

