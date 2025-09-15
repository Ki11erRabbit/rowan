use crate::backend::pre_compilation::box_closure_captures::BoxClosureCapture;
use crate::backend::pre_compilation::fix_types_after_boxing::FixTypesAfterBoxing;
use crate::backend::pre_compilation::inline_imports::InlineImports;
use crate::backend::pre_compilation::ir_conversion::IRConverter;
use crate::backend::pre_compilation::specialize_generics::SpecializeGenerics;
use crate::trees::{ast, ir};

mod box_closure_captures;
mod ir_conversion;
mod fix_types_after_boxing;
mod inline_imports;
mod specialize_generics;

pub fn ir_pass1<'ir_pass>(file: ast::File<'ir_pass>) -> Result<ir::File<'ir_pass>, ()> {
    let mut converter = IRConverter::new();
    let mut file = converter.convert(file)?;
    
    let mut boxer = BoxClosureCapture::new();
    file = boxer.box_closures(file);
    
    let mut fix_types_after_boxing = FixTypesAfterBoxing::new();
    file = fix_types_after_boxing.fix_file(file);
    
    Ok(file)
}

pub fn ir_pass2<'ir_pass>(files: Vec<ir::File<'ir_pass>>) -> Vec<ir::File<'ir_pass>> {
    let mut specialize_generics = SpecializeGenerics::new();
    specialize_generics.specialize_generics(files)
}

pub fn ir_pass3<'ir_pass>(file: ir::File<'ir_pass>) -> ir::File<'ir_pass> {
    let mut inliner = InlineImports::new();

    inliner.inline_import(file)
}