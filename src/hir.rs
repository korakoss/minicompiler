use crate::common::*;
use std::{collections::HashMap};
use crate::ast::*;


// AFTER TYPING+VARSCOPING


#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct VarId(pub usize);

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct ScopeId(pub usize);

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct FuncId(pub usize);

#[derive(Clone, Debug)]
pub enum TypedExpressionKind {
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
    }
}

#[derive(Clone, Debug)]
pub struct HIRExpression {
    pub typ: Type,
    pub expr: TypedExpressionKind,
}

#[derive(Clone, Debug)]
pub enum Place {
    Variable(VarId),
}


#[derive(Clone, Debug)]
pub struct ScopeBlock {      
    pub parent_id: Option<ScopeId>,
    pub scope_vars: HashMap<String, VarId>,
    pub within_func: bool,
    pub within_loop: bool,
    pub statements: Vec<HIRStatement>,
}

#[derive(Clone, Debug)]
pub enum HIRStatement {
    Let {
        var: Place,     // Expected to be Variable
        value: HIRExpression,
    },
    Assign {
        target: Place,   // Expected to be l-value
        value: HIRExpression,
    },
    If {
        condition: HIRExpression,    // Expected to be Boolean
        if_body: ScopeId,    
        else_body: Option<ScopeId>,
    },
    While {
        condition: HIRExpression,
        body: ScopeId,
},
    Break,
    Continue,
    Return(HIRExpression),
    Print(HIRExpression),
}

#[derive(Clone, Debug)]
pub struct HIRFunction {
    pub args: Vec<VariableInfo>,
    pub body: ScopeId,
    pub ret_type: Type,
}

#[derive(Clone, Debug)]
pub struct HIRProgram {
    pub scopes: HashMap<ScopeId, ScopeBlock>,
    pub scopetree: HashMap<ScopeId, Vec<ScopeId>>,
    pub variables: HashMap<VarId, VariableInfo>,
    pub functions: HashMap<FuncId, HIRFunction>,
    pub global_scope: Option<ScopeId>,
}

impl HIRProgram {

    pub fn new() -> Self {
        HIRProgram {
            scopes: HashMap::new(),
            scopetree: HashMap::new(),
            variables: HashMap::new(),
            functions: HashMap::new(),
            global_scope: None
        }
    }

    pub fn get_varid(&self, varname: &String, scope_id: &ScopeId) -> Option<VarId>{
        let scope = self.scopes.get(&scope_id).unwrap();
        if let Some(&varid) = scope.scope_vars.get(varname) {
            Some(varid)
        } else if let Some(parent_id) = scope.parent_id {
            self.get_varid(varname, &parent_id)
        } else {
            None
        }
    } 

    // TODO: probably generally invoke on top level to alloc?
    pub fn collect_downstream_vars(&self, scope_id: &ScopeId) -> Vec<VarId> {
        let mut result: Vec<VarId> = Vec::new();

        // TODO: errorhandling
        let scope = self.scopes.get(scope_id).unwrap();
        let scopevars = scope.scope_vars.values(); 
        result.extend(scopevars);
        
        // TODO: better comprehension?
        for child_scope_id in self.scopetree.get(scope_id).unwrap() {
            result.extend(self.collect_downstream_vars(child_scope_id)); 
        }

        result
    }

    
}
