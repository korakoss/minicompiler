use crate::ast::*;

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

#[derive(Clone)]
struct Scope {
    scope_variables: Vec<String>,   // later: variable
    inside_func: bool,
    inside_loop: bool,
}


pub struct Analyzer {
    functions: Vec<String>,      // TODO: later also mark type signature (turn into HashMap)
    errors: Vec<String>,        // Could turn into struct
}


// let errors propagate up
// TODO: could make some struct for it

impl Analyzer {
    
    pub fn new() -> Self {
        Analyzer {
            functions: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn analyze_program(&mut self, program: Program) {
        
        let Program{functions, main_statements} = program;

        // Collect defined methods (later: signatures too)
        for func in &functions {
            self.functions.push(func.name.clone()); 
        }
        
        // Analyze functions
        for func in functions {
            self.analyze_function(func);
        }
        
        let global_scope = Scope{
            scope_variables: Vec::new(), 
            inside_func: false,
            inside_loop: false,
        };
        self.analyze_statement_block(global_scope, main_statements);
    }

    
    fn analyze_function(&mut self, function: Function) {
        
        let Function{name, args, body} = function;
        let func_scope = Scope {
            scope_variables: args,
            inside_func: true,
            inside_loop: false,
        };
        self.analyze_statement_block(func_scope, body);
    }
    

    fn analyze_statement_block(&mut self, mut scope: Scope, statements: Vec<Statement>) {
        for statement in statements {
            match statement {
               Statement::If{condition, if_body, else_body} => {
                   // LATER: check that condition is boolean
                    self.analyze_statement_block(scope.clone(), if_body);
                    if let Some(else_stms) = else_body {
                        self.analyze_statement_block(scope.clone(), else_stms);
                    }
               }
               Statement::While{condition, body} => {
                   // LATER: check that condition is boolean
                   let mut loop_scope = scope.clone();
                   loop_scope.inside_loop = true;
                   self.analyze_statement_block(loop_scope, body);
                }
                Statement::Break | Statement::Continue => {
                    if !scope.inside_loop {
                        panic!("Break or  continue statement detected outside loop body");
                        // TODO: add more info
                        // TODO: push to Errors
                    }
                }
                Statement::Return(expr) => {
                    self.analyze_expression(scope.clone(), expr);
                }
                Statement::Assign{varname, value} => {
                    // TODO: some kind of clash checks somehow
                    // maybe if we had let, othwerwise we just think its overwrite
                    // (modulo type clashes, later)
                    self.analyze_expression(scope.clone(), value);

                    // TODO: only if it wasn't already there
                    scope.scope_variables.push(varname);
                }
                Statement::Print(expr) => {
                    // NOTE: later, not all types will be natively printable
                    // so we need to check for that then
                    self.analyze_expression(scope.clone(), expr);
                }
            } 
        }
    }    

    fn analyze_expression(&mut self, scope: Scope, expression: Expression) {
        match expression {
            Expression::IntLiteral(_) => {}, 
            Expression::Variable(name) => {
                if !scope.scope_variables.contains(&name) {
                    panic!("Variable not found in scope: {}", &name); //TODO: nicer
                } 
            },
            Expression::BinOp{op, left, right} => {
                // LATER: type checking
                self.analyze_expression(scope.clone(), *left);
                self.analyze_expression(scope.clone(), *right);
            },
            Expression::FuncCall{funcname, args} => {
                // TODO later: type checks
                if !self.functions.contains(&funcname) {
                    panic!("Unrecognized funcname"); // TBD
                }
                for expr in args {
                    self.analyze_expression(scope.clone(), *expr);
                }
            }
        }
    }
}
