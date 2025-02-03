use cranelift::prelude::*;
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{DataDescription, Linkage, Module};
use rowan_shared::bytecode::linked::Bytecode;

use super::{class::TypeTag, tables::{string_table::StringTable, symbol_table::{SymbolEntry, SymbolTable}, vtable::{Function, FunctionValue}}};




pub struct JIT {
    builder_context: FunctionBuilderContext,
    context: codegen::Context,
    module: JITModule,
    var_store_and_stack: Vec<Option<Value>>,
}


impl Default for JIT {
    fn default() -> Self {
        let mut flag_builder = settings::builder();
        flag_builder.set("use_colocated_libcalls", "false").unwrap();
        flag_builder.set("is_pic", "false").unwrap();
        let isa_builder = cranelift_native::builder().unwrap_or_else(|msg| {
            panic!("host machine is not supported: {}", msg);
        });
        let isa = isa_builder
            .finish(settings::Flags::new(flag_builder))
            .unwrap();
        let builder = JITBuilder::with_isa(isa, cranelift_module::default_libcall_names());

        let module = JITModule::new(builder);
        Self {
            builder_context: FunctionBuilderContext::new(),
            context: module.make_context(),
            module,
            var_store_and_stack: vec![None; 512]
        }
    }
}

impl JIT {
    pub fn set_argument(&mut self, pos: u8, value: Value) {
        self.var_store_and_stack[pos as usize] = Some(value);
    }


    pub fn set_var(&mut self, pos: u8, value: Value) {
        let pos = (pos as usize) + 256;
        self.var_store_and_stack[pos] = Some(value);
    }

    pub fn get_var(&mut self, pos: u8) -> Value {
        let pos = (pos as usize) + 256;
        match self.var_store_and_stack[pos] {
            Some(v) => v,
            _ => panic!("code was compiled wrong, empty value was not expected"),
        }
    }

    pub fn push(&mut self, value: Value) {
        self.var_store_and_stack.push(Some(value));
    }

    pub fn pop(&mut self) -> Value {
        if self.var_store_and_stack.len() <= 512 {
            panic!("Code was compiled wrong, stack underflow");
        }
        self.var_store_and_stack.pop().flatten().unwrap()
    }

    pub fn get_args_as_vec(&self) -> Vec<Value> {
        let mut output = Vec::new();
        for value in self.var_store_and_stack.iter() {
            match value {
                Some(v) => output.push(*v),
                None => {}
            }
        }
        output 
    }
    
    pub fn compile(
        &mut self,
        symbol_table: &SymbolTable,
        string_table: &StringTable,
        function: &mut Function
    ) -> Result<(), String> {

        let FunctionValue::Bytecode(bytecode) = &function.value else {
            todo!("add error handling for non-bytecode value");
        };

        self.translate(&function.arguments, &function.return_type, &bytecode)?;

        let function_name_symbol = symbol_table[function.name];
        let SymbolEntry::StringRef(name_index) = function_name_symbol else {
            todo!("Add error handling for symbol not being a string")
        };
        let name: &str = &string_table[name_index];

        let id = self
            .module
            .declare_function(name, Linkage::Export, &self.context.func.signature)
            .map_err(|e| e.to_string())?;

        self.module
            .define_function(id, &mut self.context)
            .map_err(|e| e.to_string())?;

        self.module.clear_context(&mut self.context);

        self.module.finalize_definitions().unwrap();

        let code = self.module.get_finalized_function(id);

        let value = FunctionValue::Compiled(code as *const ());

        function.value = value;
        
        Ok(())
    }

    pub fn translate(
        &mut self,
        args: &[TypeTag],
        return_type: &TypeTag,
        bytecode: &[Bytecode]
    ) -> Result<(), String> {

        for ty in args {
            let ty = match ty {
                TypeTag::U8 | TypeTag::I8 => cranelift::codegen::ir::types::I8,
                TypeTag::U16 | TypeTag::I16 => cranelift::codegen::ir::types::I16,
                TypeTag::U32 | TypeTag::I32 => cranelift::codegen::ir::types::I32,
                TypeTag::U64 | TypeTag::I64 | TypeTag::Object | TypeTag::Str => cranelift::codegen::ir::types::I64,
                TypeTag::F32 => cranelift::codegen::ir::types::F32,
                TypeTag::F64 => cranelift::codegen::ir::types::F64,
                _ => unreachable!("void in argument types"),
            };

            self.context.func.signature.params.push(AbiParam::new(ty));
        }
        loop {
            let ty = match return_type {
                TypeTag::U8 | TypeTag::I8 => cranelift::codegen::ir::types::I8,
                TypeTag::U16 | TypeTag::I16 => cranelift::codegen::ir::types::I16,
                TypeTag::U32 | TypeTag::I32 => cranelift::codegen::ir::types::I32,
                TypeTag::U64 | TypeTag::I64 | TypeTag::Object | TypeTag::Str => cranelift::codegen::ir::types::I64,
                TypeTag::F32 => cranelift::codegen::ir::types::F32,
                TypeTag::F64 => cranelift::codegen::ir::types::F64,
                TypeTag::Void => break,
            };

            self.context.func.signature.returns.push(AbiParam::new(ty));
            break;
        }

        let mut builder = FunctionBuilder::new(&mut self.context.func, &mut self.builder_context);

        

        Ok(())
    }

}
