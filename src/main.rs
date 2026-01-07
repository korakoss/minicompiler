use std::fs;
use std::env;

mod shared;

mod ast;

mod lex;
use lex::*;

mod parse;
use parse::*;

mod convert_ast;
use convert_ast::*;

mod hir;

mod make_hir;
use make_hir::*;

mod mir;

mod hir_to_mir;
use hir_to_mir::*;

mod mir_to_lir;
use mir_to_lir::*;

mod lir;

mod lir_codegen;
use lir_codegen::*;


fn main() {
    
    let args: Vec<String> = env::args().collect();
    let code_filename = &args[1];
    let assembly_filename = &args[2];
    let tokens_filepath = &args[3];
    let uast_filepath = &args[4];
    let tast_filepath = &args[5];
    let hir_filepath = &args[6];
    let mir_filepath = &args[7];
    let lir_filepath = &args[8];

    let program_text = &fs::read_to_string(code_filename).unwrap();
    let tokens = lex(program_text);
    
    fs::write(tokens_filepath, format!("{:#?}", tokens)).unwrap();

    let uast = Parser::parse_program(tokens);

    fs::write(uast_filepath, format!("{:#?}", uast)).unwrap();

    let tast = ASTConverter::convert_uast(uast);
    fs::write(tast_filepath, format!("{:#?}", tast)).unwrap();

    let hir = HIRBuilder::lower_ast(tast);
    fs::write(hir_filepath, format!("{:#?}", hir)).unwrap();

    let mir = MIRBuilder::lower_hir(hir);
    fs::write(mir_filepath, format!("{:#?}", mir)).unwrap();

    let lir = LIRBuilder::lower_mir(mir);
    fs::write(lir_filepath, format!("{:#?}", lir)).unwrap();

    let assembly = LIRCompiler::compile(lir);
    fs::write(assembly_filename, assembly).unwrap();
}


