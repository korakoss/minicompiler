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
    pub within_loop: bool,
    pub statements: Vec<HIRStatement>,
}

#[derive(Clone, Debug)]
pub struct HIRProgram {
    pub scopes: HashMap<ScopeId, Scope>,
    pub scopetree: HashMap<ScopeId, Vec<ScopeId>>,    // TODO: turn these to Vecs?  
    pub variables: HashMap<VarId, Variable>,
    pub functions: HashMap<FuncId, HIRFunction>,
    pub global_scope: Option<ScopeId>,
    function_signature_map: HashMap<(String, Vec<Type>), FuncId>, 
    function_topscope_map: HashMap<ScopeId, FuncId>
}

impl HIRProgram {

    pub fn new() -> Self {
        HIRProgram {
            scopes: HashMap::new(),
            scopetree: HashMap::new(),
            variables: HashMap::new(),
            functions: HashMap::new(),
            global_scope: None,
            function_signature_map: HashMap::new(),
            function_topscope_map: HashMap::new(),
        }
    }

    pub fn add_scope(&mut self, scope: Scope) -> ScopeId {
        let scope_id = ScopeId(self.scopes.len());

        if let Some(parent_id) = scope.parent_id.clone() {
            self.scopetree.get_mut(&parent_id).unwrap().push(scope_id.clone());
        };

        self.scopes.insert(scope_id, scope);
        self.scopetree.insert(scope_id, Vec::new());
        scope_id
    }

    pub fn add_func(&mut self, name: String, func: HIRFunction){
        let func_id = FuncId(self.functions.len());
        self.functions.insert(func_id, func.clone());
        self.function_signature_map.insert((name, func.args.into_iter().map(|x| x.typ).collect()), func_id);
    }
    
    pub fn add_var(&mut self, var: Variable, active_scope: &ScopeId) -> VarId {
        if self.get_varid(&var.name, active_scope) != None {
            panic!("Variable name exists in scope");
        }    
        let var_id = VarId(self.variables.len());
        self.variables.insert(var_id, var.clone());
        self.scopes.get_mut(active_scope).unwrap().scope_vars.insert(var.name, var_id);
        var_id 
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

    pub fn get_funcid_from_signature(&self, name: String, argtypes: Vec<Type>) -> Option<&FuncId> {
        self.function_signature_map.get(&(name, argtypes)).clone()
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

    fn find_top_ancestor(&self, scope_id:&ScopeId) -> ScopeId {
        let mut curr_id = None;
        let mut parent_id = Some(scope_id.clone());
        while parent_id.is_some() {
            curr_id = parent_id;
            parent_id = self.scopes.get(&curr_id.unwrap()).unwrap().parent_id;
        }
        curr_id.unwrap()
    }

    pub fn get_scope_ret_type(&self, scope_id: &ScopeId) -> Option<Type> {
        let scope_ancestor_id = self.find_top_ancestor(scope_id);
        let container_func = self.function_topscope_map.get(&scope_ancestor_id);
        match container_func {
            None => None,
            Some(func_id) => {
                Some(self.functions.get(func_id).unwrap().ret_type.clone())
            }
        }
    }
}
