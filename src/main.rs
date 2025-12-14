use std::fs;
use std::env;

mod ast;
use ast::*;

mod lexing;
use lexing::*;

mod parsing;
use parsing::*;

mod compiling;
use compiling::*;


// --------------- Experimental stufff -------------
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


//--------- Experimental stuff over ----------------


fn main() {
    
    let args: Vec<String> = env::args().collect();
    let code_filename = &args[1];
    let assembly_filename = &args[2];
    let program_text = &fs::read_to_string(code_filename).unwrap();
    let tokens = lex(program_text);

    for tok in &tokens {
        println!("{:?}", tok);
    }

    let parser = Parser::new(tokens);
    let program = parser.parse_program();
    println!("{:?}", program);
    let mut compiler = Compiler::new();
    let assembly = compiler.compile_program(program);
    fs::write(assembly_filename, assembly).unwrap();
}


