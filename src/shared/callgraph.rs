use std::collections::{BTreeMap, HashMap, HashSet};
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

    pub fn funcs(&self) -> Vec<FuncId> {
        self.typevar_map.keys().cloned().collect()
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

    fn goto_unprocessed(&mut self) {
        while self.stack.last().is_some_and(|nd| nd.callees.is_empty()) {
            self.stack.pop();
        }
    }

    fn pop_next(&mut self) -> Option<(FuncId, Vec<ConcreteType>)> {
        self.goto_unprocessed();
        let Some(tip_node) = self.stack.last_mut() else { return None; };
        tip_node.callees.pop()
    }
}

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
    let entry_node = MonoNode {
        func: *entry,
        type_params: vec![],
        callees: call_graph.get_concrete_callees(&entry, vec![]),
    };

    let mut required_monos: HashSet<(FuncId, Vec<ConcreteType>)> = call_graph.funcs()
        .iter()
        .map(|k| (*k, [].into()))
        .collect(); // TODO: maybe have funcs() return the iter itself?
    let mut mono_stack = MonoStack::new(entry_node);

    while let Some((curr_id, curr_tparams)) = mono_stack.pop_next() {
        let child_monos = call_graph.get_concrete_callees(&curr_id, curr_tparams);
        
        // Checking the Pareto criterion
        for (child_id, child_tparams) in child_monos.iter() {
            let child_vector: Vec<usize> = get_rank_vector(&typetable, child_tparams);
            let stack_vectors: Vec<Vec<usize>> = mono_stack
                .monos_on_stack(*child_id)
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

        // TODO: fuse these two loops, above and below, if code proves to be stable
        // Check for children completeness
        let mut exist_nonredund_child = false;
        for (child_id, child_tparams) in child_monos.iter() {
            let prev_monos: Vec<Vec<ConcreteType>> = required_monos
                .iter()
                .filter(|(id, _)| id == child_id)
                .map(|(_, tpars)| tpars)
                .cloned()
                .collect();
            if !prev_monos.contains(&child_tparams) {
                exist_nonredund_child = true;
                break;
            }
        }
        
        if !exist_nonredund_child {
            continue;
        }

        required_monos.extend(child_monos.into_iter());
    }
    required_monos
}


fn get_rank_vector(typetable: &GenericTypetable, tparams: &Vec<ConcreteType>) -> Vec<usize> {
    tparams
        .iter()
        .map(|typ| typetable.get_genericity_rank(typ))
        .collect()
}
