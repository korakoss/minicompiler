use crate::shared::typing::*;
use std::{collections::HashMap};
use crate::shared::binops::*;


#[derive(Clone, Debug)]
pub struct HIRProgram {
    pub functions: HashMap<FuncId, HIRFunction>,
    pub entry: FuncId,
    pub layouts: LayoutTable,
    pub variables: HashMap<VarId, TypedVariable>,
}


#[derive(Clone, Debug)]
pub struct HIRFunction {
    pub name: String,
    pub args: Vec<VarId>,
    pub body_variables: Vec<VarId>,   // Put the info table here instead of globally in the
                                      // program
    pub body: Vec<HIRStatement>,
    pub ret_type: Type,
}


#[derive(Clone, Debug)]
pub enum HIRStatement {
    Let {
        var: Place,     
        value: HIRExpression,
    },
    Assign {
        target: Place,  
        value: HIRExpression,
    },
    If {
        condition: HIRExpression, 
        if_body: Vec<HIRStatement>,
        else_body: Option<Vec<HIRStatement>>,
    },
    While {
        condition: HIRExpression, 
        body: Vec<HIRStatement>,
},
    Break,
    Continue,
    Return(HIRExpression),
    Print(HIRExpression),
}


#[derive(Debug, Clone)]
pub enum HIRExpression {
    IntLiteral(i32),
    Variable(VarId),
    BinOp {
       op: BinaryOperator,
       left: Box<HIRExpression>,
       right: Box<HIRExpression>,
    },
    FuncCall {
        id: FuncId, 
        args: Vec<HIRExpression>,
    },
    BoolTrue,
    BoolFalse,
    
    FieldAccess {
        expr: Box<HIRExpression>,
        field: String,
    },

    StructLiteral {
        typ: Type,
        fields: HashMap<String, HIRExpression>,
    },
}

#[derive(Clone, Debug)]
pub enum Place {
    Variable(VarId),
    // TODO: StructField
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct VarId(pub usize);

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct FuncId(pub usize);

#[derive(Clone, Debug)]
pub struct HIRVariable {
    name: String,           // Not directly necessary, but for errors later
    typ: Type,
}

#[derive(Clone, Debug)]
pub enum LayoutInfo {
    Primitive(usize),               // Variable size
    Struct {
        size: usize,
        field_offsets: HashMap<String, usize>
    }
}

impl LayoutInfo {
    pub fn size(&self) -> usize {
        match self {
            &LayoutInfo::Primitive(size) => size,
            &LayoutInfo::Struct{size, ..} => size,
        }
    }
}

#[derive(Clone, Debug)]
pub struct LayoutTable {
    newtype_layouts: HashMap<DerivType, LayoutInfo>
}

impl LayoutTable {

    pub fn make(new_types: Vec<DerivType>) -> LayoutTable {
        let mut table = LayoutTable{newtype_layouts: HashMap::new()};
        for tp in new_types {
            table.newtype_layouts.insert(tp.clone(), table.make_newtype_layout(tp)); 
        }
        table
    }   

    pub fn get_layout(&self, typ: Type) -> LayoutInfo {
        match typ {
            Type::Prim(prim_tp) => self.get_primitive_layout(prim_tp),
            Type::Derived(tp_constr) => self.newtype_layouts[&tp_constr].clone(),
        }
    }

    fn get_primitive_layout(&self, prim_tp: PrimitiveType) -> LayoutInfo {
        LayoutInfo::Primitive(8)        // Temporarily so; update later
    }
    
    fn make_newtype_layout(&self, deriv_typ: DerivType) -> LayoutInfo {

        // TODO: we have to process in topo order !!!!!!!
        // Currently, I think it spills down in that order here
        // But we should make it cleaner
        
        let TypeConstructor::Struct{fields} = deriv_typ;

        let mut f_offsets: HashMap<String, usize> = HashMap::new();

        let mut curr_offset = 0;
        for (fname, ftype) in fields {
            f_offsets.insert(fname, curr_offset);
            let fsize = self.get_layout(ftype).size(); 
            curr_offset = curr_offset + fsize;
        }

        LayoutInfo::Struct { 
            size: curr_offset, 
            field_offsets: f_offsets 
        }
    }
}

