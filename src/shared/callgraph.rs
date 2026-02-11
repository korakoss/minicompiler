use std::collections::{BTreeMap, HashMap, HashSet};

use crate::shared::{
    ids::{FuncId, TypevarId},
    tables::GenericTypetable,
    typing::{ConcreteType, GenericType},
};


#[derive(Clone, Debug)]
pub struct CallGraph {
    typevar_map: HashMap<FuncId, Vec<TypevarId>>,
    calls: HashMap<FuncId, Vec<(FuncId, Vec<GenericType>)>>,    
}

impl CallGraph {
    
    pub fn new(funcs: &[(FuncId, Vec<TypevarId>)]) -> Self {
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
        type_params: &[ConcreteType]
    ) -> Vec<(FuncId, Vec<ConcreteType>)> {
        let caller_typevars = self.typevar_map.get(caller).unwrap(); 
        assert_eq!(type_params.len(), caller_typevars.len(), "Attempted monomorphization with wrong number of type parameters");
        let tparam_bindings: BTreeMap<TypevarId, ConcreteType> = caller_typevars
            .iter()
            .cloned()
            .zip(type_params.iter().cloned())
            .collect();
        self.calls[caller]
            .iter()
            .cloned()
            .map(|(id, tps)| (
                id, 
                tps
                    .iter()
                    .cloned()
                    .map(|tp| tp.monomorphize(&tparam_bindings).unwrap())
                    .collect()
            ))
            .collect()
    }
}


#[derive(Debug)]
struct MonoNode {
    func: FuncId,
    type_params: Vec<ConcreteType>,
    callees: Vec<(FuncId, Vec<ConcreteType>)>,
}

pub fn get_monomorphizations(
    call_graph: &CallGraph, 
    typetable: &GenericTypetable,
    entry: &FuncId,
) -> HashSet<(FuncId, Vec<ConcreteType>)> {
    
    let mut required_monos: HashSet<(FuncId, Vec<ConcreteType>)> = [(*entry, vec![])].into();
    let mut mono_stack: Vec<MonoNode> = vec![MonoNode {
        func: *entry,
        type_params: vec![],
        callees: call_graph.get_concrete_callees(entry, &[]),
    }];

    while let Some(stack_tip) = mono_stack.last_mut() {
        let (curr_id, curr_tparams) = match stack_tip.callees.pop() {
            Some(callee) => callee,
            None => {
                let tip_node = mono_stack.pop().unwrap();
                (tip_node.func, tip_node.type_params)
            }
        };
        let child_monos = call_graph.get_concrete_callees(&curr_id, &curr_tparams);
        if child_monos
            .iter()
            .any(|(child_id, child_tparams)| {
                mono_stack
                    .iter()
                    .filter(|MonoNode{func: id, type_params: _,callees: _}| id == child_id)
                    .any(|node| dominates_typeslice(typetable, child_tparams, &node.type_params))
            }) 
        {
            panic!("Infinite cycle found in monomorphization");
        }
        if child_monos.iter().all(|child| required_monos.contains(child)) {
            continue;
        }
        mono_stack.push(MonoNode { 
            func: curr_id, 
            type_params: curr_tparams, 
            callees: child_monos.clone(), 
        });
        required_monos.extend(child_monos.into_iter());
    }
    required_monos
}


fn dominates_typeslice(typetable: &GenericTypetable, types1: &[ConcreteType], types2: &[ConcreteType]) -> bool {
    assert_eq!(types1.len(), types2.len(), "Attempted to compare type slices of different length");
    
    let mut strict_impr = false;
    for (t1_i, t2_i) in types1.iter().zip(types2) {
        let rank1 = typetable.get_genericity_rank(t1_i);
        let rank2 = typetable.get_genericity_rank(t2_i);
        if rank2 > rank1 { return false; }
        strict_impr |= rank2 < rank1;
    }
    strict_impr
}
