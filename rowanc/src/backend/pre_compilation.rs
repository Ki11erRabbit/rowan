use crate::backend::pre_compilation::ir_conversion::IRConverter;
use crate::trees::{ast, ir};

mod box_closure_captures;
mod ir_conversion;


pub fn ir_pass<'ir_pass>(file: ast::File<'ir_pass>) -> Result<ir::File<'ir_pass>, ()> {
    let mut converter = IRConverter::new();
    
    converter.convert(file)
}