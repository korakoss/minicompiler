use crate::shared::typing::*;
use crate::stages::ast::*;


pub struct ASTConverter {
    typetable: TypeTable, 
}

impl ASTConverter {
    
    pub fn convert_uast(uast: UASTProgram) -> TASTProgram {
        let UASTProgram{new_types, functions} = uast;
        let typetable = TypeTable::make(new_types); 
        
        let converter = ASTConverter{typetable};
        let t_functions = functions
            .into_iter()
            .map( |(sgn, func)| (converter.convert_function_signature(sgn), converter.convert_function(func)))
            .collect();
        TASTProgram { 
            typetable: converter.typetable, 
            functions: t_functions 
        }
    }

    fn convert_function_signature(&self, fsgn: DeferredFunctionSignature) -> CompleteFunctionSignature {
        let DeferredFunctionSignature{name, argtypes} = fsgn;
        CompleteFunctionSignature {
            name,
            argtypes: argtypes.into_iter().map( |ftyp| (self.typetable.convert(ftyp))).collect()
        }
    }

    
    fn convert_function(&self, func: UASTFunction) -> TASTFunction {
        let UASTFunction{name, args, body, ret_type} = func;
        TASTFunction {
            name,
            args: args
                .into_iter()
                .map(|(name, deftyp)| (name, self.typetable.convert(deftyp)))
                .collect(),
            body: body.into_iter().map(|stmt| self.convert_statement(stmt)).collect(),
            ret_type: self.typetable.convert(ret_type)
        }
    }
    
    fn convert_statement(&self, statement: UASTStatement) -> TASTStatement {
        match statement {
            UASTStatement::Let { var, value } => {
                TASTStatement::Let { 
                    var: self.convert_var(var), 
                    value: self.convert_expression(value), 
                }
            }
            UASTStatement::Assign { target, value } => {
                TASTStatement::Assign {
                    target,
                    value: self.convert_expression(value) 
                }
            }
            UASTStatement::If { condition, if_body, else_body } => {
                TASTStatement::If { 
                    condition: self.convert_expression(condition),
                    if_body: if_body.into_iter().map(|stmt| self.convert_statement(stmt)).collect(),
                    else_body: match else_body {
                        None => None,
                        Some(else_statements) => Some(else_statements.into_iter().map(|stmt| self.convert_statement(stmt)).collect()),
                    }
                }
            }
            UASTStatement::While { condition, body } => {
                TASTStatement::While { 
                    condition: self.convert_expression(condition), 
                    body: body.into_iter().map(|stmt| self.convert_statement(stmt)).collect(),
                } 
            }
            UASTStatement::Break => TASTStatement::Break,
            UASTStatement::Continue => TASTStatement::Continue,
            UASTStatement::Return(expr) => TASTStatement::Return(self.convert_expression(expr)),
            UASTStatement::Print(expr) => TASTStatement::Print(self.convert_expression(expr)),
        }
    }

    fn convert_expression(&self, expr: UASTExpression) -> TASTExpression {
        match expr {
            UASTExpression::IntLiteral(n) => TASTExpression::IntLiteral(n),
            UASTExpression::Variable(name) => TASTExpression::Variable(name),
            UASTExpression::BinOp{op, left, right} => {
                TASTExpression::BinOp{
                    op, 
                    left: Box::new(self.convert_expression(*left)), 
                    right:Box::new(self.convert_expression(*right)), 
                }
            },
            UASTExpression::FuncCall{funcname, args} => {
                TASTExpression::FuncCall{
                    funcname, 
                    args: args.into_iter().map(|expr| self.convert_expression(expr)).collect(),
                }
            }
            UASTExpression::BoolTrue => TASTExpression::BoolTrue,
            UASTExpression::BoolFalse => TASTExpression::BoolFalse,
            UASTExpression::FieldAccess{ expr, field} => {
                TASTExpression::FieldAccess{ 
                    expr: Box::new(self.convert_expression(*expr)), 
                    field, 
                }
            }
            UASTExpression::StructLiteral { typ, fields } => {
                TASTExpression::StructLiteral{ 
                    typ: self.typetable.convert(typ), 
                    fields: fields.into_iter().map(|(fname, fexpr)| (fname, self.convert_expression(fexpr))).collect(),
                }
            }
        }
    }
    
    fn convert_var(&self, var: DeferredTypeVariable) -> TypedVariable {
        let DeferredTypeVariable{name, typ} = var; 
        TypedVariable {
            name,
            typ: self.typetable.convert(typ)
        }
    }
}
