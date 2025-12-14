
// Type should really go into AST later
#[derive(Clone)]
enum Type {
    Int,
    Bool,
    // Pointer,
    // Struct,
    // Array
}


#[derive(Clone)]
struct Variable {
    name: String,
    vartype: Type,
}


struct Scope {
    external_variables: Vec<Variable>,
    scope_variables: Vec<Variable>,
    inside_func: bool,
    inside_loop: bool,
}

impl Scope {

    fn create_global() -> Scope {               // Phase out for main()
        Scope {
            external_variables: Vec::new(),
            scope_variables: Vec::new(),
            inside_func: false,
            inside_loop: false,
        }
    }

    fn descend(ancestor_scope: Scope, loop_entry: bool) -> Scope {
       let mut new_externals = ancestor_scope.external_variables.clone();
       new_externals.extend(ancestor_scope.scope_variables);
       let new_inloop = loop_entry || ancestor_scope.inside_loop;
       Scope {
            external_variables: new_externals,
            scope_variables: Vec::new(),
            inside_func: ancestor_scope.inside_func,
            inside_loop: new_inloop,
       }
    } 

    fn is_defined_variable(&self, name: String) -> bool {
        self.external_variables.iter().any(|var| var.name == name)|| self.scope_variables.iter().any(|var| var.name == name)
    }
}


