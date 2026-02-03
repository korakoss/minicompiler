use std::collections::HashMap;
use crate::shared::ids::FuncId;
use crate::shared::typing::{ConcreteType, GenericType};


#[derive(Clone, Debug)]
pub struct CallGraph {
    calls: HashMap<FuncId, Vec<(FuncId, Vec<GenericType>)>>,    
    // Stores the callees along with a vector of type parameter indices to substitute 
}

impl CallGraph {
    
    pub fn new(ids: &Vec<FuncId>) -> Self {
        Self {
            calls: ids.iter().map(|i| (*i, vec![])).collect()
        }
    }
    /*
    pub fn get_concrete_callees(
        &self, 
        caller_id: &FuncId, 
        type_params: Vec<ConcreteType>
    ) -> Vec<(FuncId, Vec<ConcreteType>)> {
        self.calls[&caller_id]
            .iter()
            .map(|(id, param_indices)| (*id, param_indices
                .iter()
                .map(|idx| type_params[*idx].clone())
                .collect()
            ))
            .collect()
    }
    */

    pub fn add_callee(&mut self, caller: &FuncId, callee: (FuncId, Vec<GenericType>)) {
        self.calls.get_mut(caller).unwrap().push(callee);
    }
}
