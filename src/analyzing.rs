use crate::ast::*;
use crate::typing::*;


#[derive(Clone)]
pub struct Variable {
    name: String,
    vartype: Type,
}

#[derive(Clone)]
struct Scope {
    scope_variables: Vec<String>,   // later: variable
    inside_func: bool,
    inside_loop: bool,
}

struct FuncDef {
    name: String,
    args: Vec<Variable>,
    ret_type: Type,
}


pub struct Analyzer {
    functions: Vec<FuncDef>,     
    errors: Vec<String>,        // Could turn into struct
}


// analyzed content and errors propagate up
// print linenums, we should be able to do that
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
        // TODO: watch out for name clashes
        // though we may allow type polymorhpy later?
        
        self.functions = functions.iter().map(|func| self.analyze_function(func.clone())).collect();
                
        let global_scope = Scope{
            scope_variables: Vec::new(), 
            inside_func: false,
            inside_loop: false,
        };
        self.analyze_statement_block(global_scope, main_statements);
    }

    
    fn analyze_function(&mut self, function: Function) -> FuncDef{
        
        let Function{name, args, body} = function;
        let func_scope = Scope {
            scope_variables: args.clone(),
            inside_func: true,
            inside_loop: false,
        };
        self.analyze_statement_block(func_scope, body);
        let arg_variables = args.iter().map(|argname| Variable{name: argname.clone(), vartype: Type::Int}).collect();          // NOTE: int is stopgap, change later
        FuncDef{
            name: name,
            args: arg_variables,
            ret_type: Type::Int              // TODO: update this later too
        }
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
                    if !scope.inside_func {
                        panic!("Return statement detected outside function body");
                    }
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
                if !self.functions.iter().any(|funcdef| funcdef.name == funcname) {
                    panic!("Unrecognized funcname"); // TBD
                }
                for expr in args {
                    self.analyze_expression(scope.clone(), *expr);
                }
            }
        }
    }
}
