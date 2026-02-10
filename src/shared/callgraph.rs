use std::collections::{BTreeMap, HashMap, HashSet};

use crate::shared::{
    ids::FuncId,
    tables::GenericTypetable,
    typing::{ConcreteType, GenericType, TypevarId},
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
        type_params: Vec<ConcreteType>
    ) -> Vec<(FuncId, Vec<ConcreteType>)> {
        let caller_typevars = self.typevar_map[caller].clone(); 
        if type_params.len() != caller_typevars.len() {
            panic!("Attempted monomorphization with wrong number of type parameters");
        }
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

    fn pop_next(&mut self) -> Option<(FuncId, Vec<ConcreteType>)> {
        if self.stack.is_empty() {
            None
        } else if self.stack.last_mut().unwrap().callees.is_empty() {
            let tip_node = self.stack.pop().unwrap();
            Some((tip_node.func, tip_node.type_params))
        } else {
            let tip_node = self.stack.last_mut().unwrap();
            Some(tip_node.callees.pop().unwrap())
        }
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
    let mut mono_stack = MonoStack::new(
        MonoNode {
            func: *entry,
            type_params: vec![],
            callees: call_graph.get_concrete_callees(entry, vec![]),
        }
    );

    while let Some((curr_id, curr_tparams)) = mono_stack.pop_next() {
        let child_monos = call_graph.get_concrete_callees(&curr_id, curr_tparams.clone());
        
        // Checking the Pareto criterion
        for (child_id, child_tparams) in child_monos.iter() {
            let child_vector: Vec<usize> = get_rank_vector(typetable, child_tparams);
            let stack_vectors: Vec<Vec<usize>> = mono_stack
                .monos_on_stack(*child_id)
                .iter()
                .map(|tpars| get_rank_vector(typetable, tpars))
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


fn get_rank_vector(typetable: &GenericTypetable, tparams: &[ConcreteType]) -> Vec<usize> {
    tparams
        .iter()
        .map(|typ| typetable.get_genericity_rank(typ))
        .collect()
}

pub fn dominates(a: &[usize], b: &[usize]) -> bool {
    if a.len() != b.len() {
        panic!("Attempted to compare vectors of different length");
    }
    let mut strict_improvement = false;
    
    for i in 0..a.len() {
        if b[i] > a[i] {
            return false;
        } else if b[i] < a[i] {
            strict_improvement = true;
        }
    }
    strict_improvement
}
