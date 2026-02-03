use std::collections::{BTreeMap, HashMap};
use crate::shared::ids::FuncId;
use crate::shared::typing::{ConcreteType, GenericType, TypevarId};


#[derive(Clone, Debug)]
pub struct CallGraph {
    typevar_map: HashMap<FuncId, Vec<TypevarId>>,
    calls: HashMap<FuncId, Vec<(FuncId, Vec<GenericType>)>>,    
}

impl CallGraph {
    
    pub fn new(funcs: &Vec<(FuncId, Vec<TypevarId>)>) -> Self {
        Self {
            typevar_map: funcs.iter().cloned().collect(),
            calls: funcs.iter().map(|(id, _)| (*id, vec![])).collect()
        }
    }

    pub fn add_callee(&mut self, caller: &FuncId, callee: (FuncId, Vec<GenericType>)) {
        self.calls.get_mut(caller).unwrap().push(callee);
    }

    pub fn get_concrete_callees(
        &self, 
        caller: &FuncId, 
        type_params: Vec<ConcreteType>
    ) -> Vec<(FuncId, Vec<ConcreteType>)> {
        let caller_typevars = self.typevar_map[&caller].clone(); 
        if type_params.len() != caller_typevars.len() {
            panic!("Attempted monomorphization with wrong number of type parameters");
        }
        let tparam_bindings: BTreeMap<TypevarId, ConcreteType> = caller_typevars
            .iter()
            .cloned()
            .zip(type_params.iter().cloned())
            .collect();
        self.calls[&caller]
            .iter()
            .cloned()
            .map(|(id, tps)| (
                id, 
                tps
                    .iter()
                    .cloned()
                    .map(|tp| tp.monomorphize(&tparam_bindings))
                    .collect()
            ))
            .collect()
    }
}
