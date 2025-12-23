use crate::common::*;
use std::{collections::HashMap};


#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct VarId(pub usize);

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct ScopeId(pub usize);

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct FuncId(pub usize);

#[derive(Clone, Debug)]
pub enum Place {
    Variable(VarId),
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
    pub args: Vec<Variable>,
    pub body: ScopeId,
    pub ret_type: Type,
}

#[derive(Clone, Debug)]
pub struct Scope {      
    pub parent_id: Option<ScopeId>,
    pub scope_vars: HashMap<String, VarId>,
    pub within_func: bool,
    pub within_loop: bool,
    pub statements: Vec<HIRStatement>,
}

// TODO: could make this have a nice partially exposed interface
#[derive(Clone, Debug)]
pub struct HIRProgram {
    pub scopes: HashMap<ScopeId, Scope>,
    pub scopetree: HashMap<ScopeId, Vec<ScopeId>>,      
    pub variables: HashMap<VarId, Variable>,
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

    pub fn collect_scope_vars(&self, scope_id: &ScopeId) -> HashMap<String, VarId> {
        let mut result = HashMap::new();

        // Collect variables from descendant scopes
        for child_scope_id in self.scopetree.get(scope_id).unwrap() {
            result.extend(self.collect_scope_vars(child_scope_id)); 
        }
       
        // Walk up the ancestor chain to collect from there
        let mut curr_id = Some(scope_id.clone());
        while let Some(id) = curr_id {
            let scope = self.scopes.get(&id).unwrap();
            result.extend(scope.scope_vars.clone());
            curr_id = scope.parent_id.clone();
        }
        result
    }
}
