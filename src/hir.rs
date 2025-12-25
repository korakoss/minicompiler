use crate::common::*;
use std::{collections::HashMap};


#[derive(Clone, Debug)]
pub struct HIRProgram {
    pub functions: HashMap<FuncId, HIRFunction>,
    pub entry: Option<FuncId>,
}


#[derive(Clone, Debug)]
pub struct HIRFunction {
    pub args: Vec<VarId>,
    pub body: Vec<HIRStatement>,
    pub variables: HashMap<VarId, Variable>,
    pub ret_type: Type,
}

impl HIRFunction {

    pub fn new(args: Vec<Variable>, ret_type: Type) -> Self {
        let mut hir_func = HIRFunction {
            args: Vec::new(),
            body: Vec::new(),
            variables: HashMap::new(),
            ret_type: ret_type,
        };
        for arg in args {
            let arg_id = hir_func.add_var(arg);
            hir_func.args.push(arg_id);
        }
        hir_func
    }
            
    pub fn add_var(&mut self, var: Variable) -> VarId {
        let var_id = VarId(self.variables.len());
        self.variables.insert(var_id, var.clone());
        var_id 
    }
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

#[derive(Clone, Debug)]
pub struct HIRExpression {
    pub typ: Type,
    pub expr: HIRExpressionKind,
}

#[derive(Clone, Debug)]
pub enum HIRExpressionKind {
    IntLiteral(i32),
    Variable(VarId),
    BinOp {
        op: BinaryOperator,
        left: Box<HIRExpression>,
        right: Box<HIRExpression>,
    },
    FuncCall {
        funcid: FuncId,
        args: Vec<HIRExpression>,
    },
    BoolTrue,
    BoolFalse,
}

#[derive(Clone, Debug)]
pub enum Place {
    Variable(VarId),
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct VarId(pub usize);

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct FuncId(pub usize);

