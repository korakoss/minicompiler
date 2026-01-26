use std::fs;
use std::env;

mod shared;
mod stages;
mod passes;
use passes::preproc::lex::*;
use passes::preproc::parse::*;
use passes::make_hir::*;
use passes::hir_to_mir::*;
use passes::mir_to_lir::*;
use passes::lir_codegen::*;

fn main() {
    
    let args: Vec<String> = env::args().collect();
    let code_filename = &args[1];
    let assembly_filename = &args[2];
    let tokens_filepath = &args[3];
    let ast_filepath = &args[4];
    let hir_filepath = &args[5];
    let mir_filepath = &args[6];
    let lir_filepath = &args[7];

    let program_text = &fs::read_to_string(code_filename).unwrap();
    let tokens = lex(program_text);
    
    fs::write(tokens_filepath, format!("{:#?}", tokens)).unwrap();

    let ast = Parser::parse_program(tokens);
    fs::write(ast_filepath, format!("{:#?}", ast)).unwrap();

    let hir = HIRBuilder::lower_ast(ast);
    fs::write(hir_filepath, format!("{:#?}", hir)).unwrap();

    let mir = MIRBuilder::lower_hir(hir);
    fs::write(mir_filepath, format!("{:#?}", mir)).unwrap();

    let lir = LIRBuilder::lower_mir(mir);
    fs::write(lir_filepath, format!("{:#?}", lir)).unwrap();

    let assembly = LIRCompiler::compile(lir);
    fs::write(assembly_filename, assembly).unwrap();
}


