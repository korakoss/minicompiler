use std::collections::{BTreeMap, HashMap};
use crate::shared::ids::FuncId;
use crate::shared::tables::GenericTypetable;
use crate::shared::typing::{ConcreteType, GenericType, TypevarId};
use crate::passes::pareto::dominates;


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



struct MonoStack {
    stack: Vec<MonoNode>,
}

impl MonoStack {

    fn new(entry: MonoNode) -> Self {
        Self { stack: vec![entry] }
    }

    fn monos_on_stack(&self, fid: FuncId) -> Vec<Vec<ConcreteType>> {
        self.stack
            .iter()
            .filter(|MonoNode{func: id, type_params: _,callees: _}| *id == fid)
            .map(|mn| mn.type_params.clone())
            .collect()
    }

    fn push(&mut self, nd: MonoNode) {
        self.stack.push(nd);
    }

    fn dfs_pop_child(&mut self) -> Option<(FuncId, Vec<ConcreteType>)> {
        // TODO: maybe some checks or whatever
        self.stack.last_mut().and_then(|nd| nd.callees.pop())
    }

    fn retreat(&mut self) {
        // TODO: maybe build some checks in
        self.stack.pop();
    }

    fn empty(&self) -> bool {
        self.stack.is_empty()
    }
}

struct MonoNode {
    func: FuncId,
    type_params: Vec<ConcreteType>,
    callees: Vec<(FuncId, Vec<ConcreteType>)>,
}

fn get_monomorphizations(
    call_graph: CallGraph, 
    typetable: GenericTypetable,
    entry: FuncId,
) -> HashMap<FuncId, Vec<Vec<ConcreteType>>> {
    let entry_node = MonoNode {
        func: entry,
        type_params: vec![],
        callees: call_graph.get_concrete_callees(&entry, vec![]),
    };
    let mut mono_stack = MonoStack::new(entry_node);

    while !mono_stack.empty() {
        let (curr_id, curr_tparams) = mono_stack.dfs_pop_child().unwrap();
        let child_monos = call_graph.get_concrete_callees(&curr_id, curr_tparams);
        
        // Checking the Pareto criterion
        for (child_id, child_tparams) in child_monos {
            let child_vector: Vec<usize> = get_rank_vector(&typetable, &child_tparams);
            let stack_vectors: Vec<Vec<usize>> = mono_stack
                .monos_on_stack(child_id)
                .iter()
                .map(|tpars| get_rank_vector(&typetable, tpars))
                .collect();
           let dominates_old = stack_vectors
                .iter()
                .any(|v| dominates(&child_vector, v));
            if dominates_old {
                panic!("Infinite cycle found in monomorphization");
            }
        }

        // TODO: check for all reqs being redundant
            // TODO: collecting required monos into a flat list (the result actually) 
    }
    unimplemented!();
}


fn get_rank_vector(typetable: &GenericTypetable, tparams: &Vec<ConcreteType>) -> Vec<usize> {
    tparams
        .iter()
        .map(|typ| typetable.get_genericity_rank(typ))
        .collect()
}
