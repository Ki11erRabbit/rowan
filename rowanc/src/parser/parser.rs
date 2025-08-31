use std::ops::Range;
use ariadne::{Color, Label, Report, ReportBuilder, ReportKind};
use lalrpop_util::{lalrpop_mod};
use crate::parser::lexer::LexerError;
use crate::trees::ast;

use super::lexer;

lalrpop_mod!(grammar, "/parser/grammar.rs");


pub fn parse<'a>(_: &'a str, path: &'a str, input: &'a str) -> Result<ast::File<'a>, ReportBuilder<'a, (&'a str, Range<usize>)>> {
    let lexer = lexer::TokenLexer::new(input);
    match grammar::FileParser::new().parse(input, lexer) {
        Err(err) => {
            let mut start = 0;
            let mut end = 0;
            let mut message = String::new();
            let mut main_label = None;
            let mut label = None;
            let mut called_once = false;

            err.map_location(|x| {
                if called_once {
                    end = x;
                    main_label = Some(
                        Label::new((path, (start..end)))
                            .with_message("here")
                            .with_color(Color::Red),
                    );
                } else {
                    message.push_str("Unexpected Token");
                    start = x;
                    called_once = true;
                }
                
            }).map_error(|err| {
                start = err.start;
                end = err.end;
                match err.error {
                    LexerError::UnexpectedCharacter(c) => {
                        message.push_str(&format!("Unexpected Character: {c}"));
                    }
                    LexerError::InvalidIdentifier(start, stop) => {
                        message.push_str("Invalid Identifier");
                        label = Some(
                            Label::new((path, start..stop))
                                .with_color(Color::Blue)
                        );
                    }
                    LexerError::UnexpectedEndOfInput => {
                        message.push_str("Unexpected End of Input");
                    }
                    LexerError::UnclosedStringLiteral => {
                        message.push_str("Unclosed String Literal");
                    }
                    LexerError::UnclosedCharLiteral => {
                        message.push_str("Unclosed Char Literal");
                    }
                    LexerError::UnclosedComment => {
                        message.push_str("Unclosed Comment");
                    }
                    LexerError::UnknownError => {
                        message.push_str("Unknown Error");
                    }
                    LexerError::InvalidOperator => {
                        message.push_str("Invalid Operator");
                    }
                    LexerError::ErrorCollection(errors) => {
                        todo!("error collection")
                    }
                    LexerError::Eof => {
                        unreachable!("Eof")
                    }
                }
            });
            
            let span = (path, start..end);
            let mut builder = Report::build(ReportKind::Error, span)
                .with_message(message);
            builder = if let Some(main_label) = main_label {
                builder.with_label(main_label)
            } else {
                builder
            };
            builder = if let Some(label) = label {
                builder.with_label(label)
            } else {
                builder
            };
            Err(
                builder
            )
        }
        Ok(ok) => Ok(ok),
    }
}
