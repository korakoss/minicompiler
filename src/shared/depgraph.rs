
fn extract_newtype_dependencies(newtype_defs: &HashMap<NewtypeId, GenericTypeDef>) -> HashMap<NewtypeId, Vec<NewtypeId>> {
    let mut dep_graph: HashMap<NewtypeId, Vec<NewtypeId>> = HashMap::new();
    for (type_id, newtype) in newtype_defs {
        let deps: Vec<NewtypeId> = match &newtype.defn {
            NewtypeShape::Struct {fields} => fields
                .values()
                .flat_map(extract_type_id)
                .collect(),
            NewtypeShape::Enum { variants } => variants
                .iter()
                .flat_map(extract_type_id)
                .collect(),
        };
        dep_graph.insert(type_id.clone(), deps);
    }
    dep_graph
}


fn extract_type_id(t: &GenericType) -> Vec<NewtypeId> {
    match t {
        GenericType::Prim(..) => vec![],
        GenericType::NewType(id, t_params) => {
            let mut deps: Vec<NewtypeId> = t_params
                .iter()
                .flat_map(extract_type_id)
                .collect::<Vec<_>>();
            deps.push(id.clone());
            deps
        }
        GenericType::Reference(typ) => extract_type_id(typ),
        GenericType::TypeVar(..) => vec![] 
    }
}


fn toposort_depgraph<T: Clone + Eq + PartialEq + Hash>(depgraph: HashMap<T, Vec<T>>) -> Vec<T> {

    let mut indegrees: HashMap<T, usize> = depgraph
        .keys()
        .map(|k| (k.clone(),0))
        .collect();
    
   for neighbor in depgraph.values().flatten() {
        if let Some(count) = indegrees.get_mut(neighbor) {
            *count += 1;
        }
    }

    let mut queue: VecDeque<T> = depgraph
        .keys()
        .filter(|node| indegrees[node] == 0)
        .cloned()
        .collect();
    let mut result: Vec<T> = Vec::new();
    
    while let Some(node) = queue.pop_front() {
        result.push(node.clone());
        
        if let Some(neighbors) = depgraph.get(&node) {
            for neighbor in neighbors {
                if let Some(deg) = indegrees.get_mut(neighbor) {
                    *deg -= 1;
                    if *deg == 0 {
                        queue.push_back(neighbor.clone());
                    }
                }
            }
        }
    }
    if result.len() != indegrees.len() {
        panic!("Cycle detected in type definitions");
    }
    result.reverse();
    result
}
