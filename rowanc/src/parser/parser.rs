use lalrpop_util::lalrpop_mod;

use crate::trees::ast;

use super::lexer;

lalrpop_mod!(grammar, "/parser/grammar.rs");


pub fn parse<'a>(name: &str, input: &'a str) -> Result<ast::File<'a>, ()> {
    let lexer = lexer::TokenLexer::new(input);
    let output = grammar::FileParser::new().parse(input, lexer).expect("handle errors in parser");

    Ok(output)
}
