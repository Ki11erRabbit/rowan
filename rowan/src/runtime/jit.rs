
use std::collections::HashMap;

use codegen::ir::FuncRef;
use cranelift::prelude::*;
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{FuncId, Linkage, Module, ModuleResult};
use rowan_shared::bytecode::linked::Bytecode;

use rowan_shared::TypeTag;
use super::{tables::{string_table::StringTable, symbol_table::{SymbolEntry, SymbolTable}, vtable::{Function, FunctionValue}}, Context, Symbol};
use std::sync::Arc;




pub struct JITController {
    builder_context: FunctionBuilderContext,
    context: codegen::Context,
    module: JITModule,
    jit_utility_func: Arc<HashMap<String, FuncRef>>,
}


impl Default for JITController {
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
        let mut builder = JITBuilder::with_isa(isa, cranelift_module::default_libcall_names());
        builder.symbol("get_virtual_function", super::get_virtual_function as *const u8);
        let mut module = JITModule::new(builder);

        let mut context = module.make_context();
        let mut builder_context = FunctionBuilderContext::new();

        let mut get_virt_func = module.make_signature();
        get_virt_func.params.push(AbiParam::new(cranelift::codegen::ir::types::I64));
        get_virt_func.params.push(AbiParam::new(cranelift::codegen::ir::types::I64));
        get_virt_func.params.push(AbiParam::new(cranelift::codegen::ir::types::I64));
        get_virt_func.params.push(AbiParam::new(cranelift::codegen::ir::types::I64));
        get_virt_func.returns.push(AbiParam::new(cranelift::codegen::ir::types::I64));

        let fn_id = module.declare_function("get_virtual_function", Linkage::Import, &get_virt_func).unwrap();
        
        let func_builder = FunctionBuilder::new(&mut context.func, &mut builder_context);
        let func = module.declare_func_in_func(fn_id, func_builder.func);

        let mut jit_utility_func = HashMap::new();
        jit_utility_func.insert(String::from("get_virtual_function"), func);

        

        Self {
            builder_context,
            context,
            module,
            jit_utility_func: Arc::new(jit_utility_func),
        }
    }
}

impl JITController {

    pub fn create_signature(&self, args: &[TypeTag], return_type: &TypeTag) -> Signature {
        let mut signature = self.module.make_signature();
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

            signature.params.push(AbiParam::new(ty));
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

            signature.returns.push(AbiParam::new(ty));
            break;
        }
        signature
    }
    
    pub fn declare_function(&mut self, name: &str, signature: &Signature) -> ModuleResult<FuncId> {
        self.module.declare_function(name, Linkage::Export, signature)
    }

    pub fn new_context(&self) -> codegen::Context {
        self.module.make_context()
    }

    pub fn get_utility_functions(&self) -> Arc<HashMap<String, FuncRef>> {
        self.jit_utility_func.clone()
    }
}

unsafe impl Send for JITController {}
unsafe impl Sync for JITController {}


pub struct JITCompiler {
    builder_context: FunctionBuilderContext,
    context: codegen::Context,
    jit_utility_func: Arc<HashMap<String, FuncRef>>,
}

impl JITCompiler {
    pub fn new(context: codegen::Context, jit_utility_func: Arc<HashMap<String, FuncRef>>) -> JITCompiler {
        JITCompiler {
            builder_context: FunctionBuilderContext::new(),
            context,
            jit_utility_func

        }
    }

    pub fn compile(
        &mut self,
        function: &mut Function,
        module: &mut JITModule,
    ) -> Result<(), String> {

        let Ok(mut value) = function.value.write() else {
            panic!("Lock poisoned");
        };

        let FunctionValue::Bytecode(bytecode, id, sig) = &*value else {
            todo!("add error handling for non-bytecode value");
        };

        self.translate(&bytecode)?;


        module
            .define_function(*id, &mut self.context)
            .map_err(|e| e.to_string())?;

        module.clear_context(&mut self.context);

        module.finalize_definitions().unwrap();

        let code = module.get_finalized_function(*id);

        let new_function_value = FunctionValue::Compiled(code as *const (), sig.clone());

        *value = new_function_value;
        
        Ok(())
    }

    pub fn translate(
        &mut self,
        bytecode: &[Bytecode]
    ) -> Result<(), String> {

        let mut function_translator = FunctionTranslator::new(
            &mut self.context,
            &mut self.builder_context,
            &self.jit_utility_func
        );

        function_translator.translate(bytecode)?;
        

        Ok(())
    }

}


pub struct FunctionTranslator<'a> {
    builder: FunctionBuilder<'a>,
    var_store_and_stack: Vec<Option<Value>>,
    blocks: Vec<Block>,
    current_block: usize,
    jit_utility_func: &'a HashMap<String, FuncRef>,
}

impl FunctionTranslator<'_> {
    pub fn new<'a>(
        context: &'a mut codegen::Context,
        builder_context: &'a mut FunctionBuilderContext,
        jit_utility_func: &'a HashMap<String, FuncRef>,
    ) -> FunctionTranslator<'a> {
        let mut builder = FunctionBuilder::new(&mut context.func, builder_context);

        let entry_block = builder.create_block();

        builder.append_block_params_for_function_params(entry_block);
        builder.switch_to_block(entry_block);
        builder.seal_block(entry_block);
        let mut var_store_and_stack = vec![None; 512];
        for (i, value) in builder.block_params(entry_block).iter().enumerate() {
            var_store_and_stack[i + 256] = Some(value.clone());
        }

        FunctionTranslator {
            builder,
            var_store_and_stack,
            blocks: vec![entry_block],
            current_block: 0,
            jit_utility_func
        }
    }
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

    pub fn dup(&mut self) {
        if self.var_store_and_stack.len() <= 512 {
            panic!("Code was compiled wrong, no variables on the stack");
        }
        let last = self.var_store_and_stack.last().cloned().flatten();
        self.var_store_and_stack.push(last);
    }

    pub fn swap(&mut self) {
        if self.var_store_and_stack.len() <= 512 {
            panic!("Code was compiled wrong, no variables on the stack");
        }
        if self.var_store_and_stack.len() < 514 {
            panic!("Code was compiled wrong, missing two values on stack");
        }
        let top = self.var_store_and_stack.pop().flatten();
        let bottom = self.var_store_and_stack.pop().flatten();
        self.var_store_and_stack.push(bottom);
        self.var_store_and_stack.push(top);
    }

    pub fn get_call_arguments_as_vec(&self) -> Vec<Value> {
        let mut output = Vec::new();
        for (i, value) in self.var_store_and_stack.iter().enumerate() {
            if i >= 256 {
                break;
            }
            match value {
                Some(v) => output.push(*v),
                None => break,
            }
        }


        output
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


    pub fn translate(&mut self, bytecode: &[Bytecode]) -> Result<(), String> {

        for bytecode in bytecode.iter() {
            match bytecode {
                Bytecode::Nop | Bytecode::Breakpoint => {}
                Bytecode::LoadU8(value) => {
                    let value = self.builder.ins().iconst(cranelift::codegen::ir::types::I8, i64::from(*value));
                    self.push(value);
                }
                Bytecode::LoadI8(value) => {
                    let value = self.builder.ins().iconst(cranelift::codegen::ir::types::I8, i64::from(*value));
                    self.push(value);
                }
                Bytecode::LoadU16(value) => {
                    let value = self.builder.ins().iconst(cranelift::codegen::ir::types::I16, i64::from(*value));
                    self.push(value);
                }
                Bytecode::LoadI16(value) => {
                    let value = self.builder.ins().iconst(cranelift::codegen::ir::types::I16, i64::from(*value));
                    self.push(value);
                }
                Bytecode::LoadU32(value) => {
                    let value = self.builder.ins().iconst(cranelift::codegen::ir::types::I32, i64::from(*value));
                    self.push(value);
                }
                Bytecode::LoadI32(value) => {
                    let value = self.builder.ins().iconst(cranelift::codegen::ir::types::I32, i64::from(*value));
                    self.push(value);
                }
                Bytecode::LoadU64(value) => {
                    let value = self.builder.ins().iconst(cranelift::codegen::ir::types::I64, i64::from(i64::from_le_bytes(value.to_le_bytes())));
                    self.push(value);
                }
                Bytecode::LoadI64(value) => {
                    let value = self.builder.ins().iconst(cranelift::codegen::ir::types::I64, i64::from(*value));
                    self.push(value);
                }
                Bytecode::LoadF32(value) => {
                    let value = self.builder.ins().f32const(*value);
                    self.push(value);
                }
                Bytecode::LoadF64(value) => {
                    let value = self.builder.ins().f64const(*value);
                    self.push(value);
                }
                Bytecode::Pop => {
                    self.pop();
                }
                Bytecode::Dup => {
                    self.dup();
                }
                Bytecode::Swap => {
                    self.swap();
                }
                Bytecode::StoreLocal(index) => {
                    let value = self.pop();
                    self.set_var(*index, value);
                }
                Bytecode::LoadLocal(index) => {
                    let value = self.get_var(*index);
                    self.push(value);
                }
                Bytecode::StoreArgument(index) => {
                    let value = self.pop();
                    self.set_argument(*index, value);
                }
                Bytecode::Add => {
                    let value_rhs = self.pop();
                    let value_lhs = self.pop();
                    let value_out = self.builder.ins().iadd(value_lhs, value_rhs);
                    self.push(value_out);
                }
                Bytecode::Sub => {
                    let value_rhs = self.pop();
                    let value_lhs = self.pop();
                    let value_out = self.builder.ins().isub(value_lhs, value_rhs);
                    self.push(value_out);
                }
                Bytecode::Mul => {
                    let value_rhs = self.pop();
                    let value_lhs = self.pop();
                    let value_out = self.builder.ins().imul(value_lhs, value_rhs);
                    self.push(value_out);
                }
                Bytecode::Div => {
                    let value_rhs = self.pop();
                    let value_lhs = self.pop();
                    let value_out = self.builder.ins().udiv(value_lhs, value_rhs);
                    self.push(value_out);
                }
                Bytecode::Mod => {
                    let value_rhs = self.pop();
                    let value_lhs = self.pop();
                    let value_out = self.builder.ins().urem(value_lhs, value_rhs);
                    self.push(value_out);
                }
                Bytecode::SatAdd => {
                    let value_rhs = self.pop();
                    let value_lhs = self.pop();
                    let value_out = self.builder.ins().uadd_sat(value_lhs, value_rhs);
                    self.push(value_out);
                }
                Bytecode::SatSub => {
                    let value_rhs = self.pop();
                    let value_lhs = self.pop();
                    let value_out = self.builder.ins().usub_sat(value_lhs, value_rhs);
                    self.push(value_out);
                }
                Bytecode::SatMul => {
                    let value_rhs = self.pop();
                    let value_lhs = self.pop();
                    let value_out = self.builder.ins().imul(value_lhs, value_rhs);
                    self.push(value_out);
                }
                Bytecode::SatDiv => {
                    let value_rhs = self.pop();
                    let value_lhs = self.pop();
                    let value_out = self.builder.ins().udiv(value_lhs, value_rhs);
                    self.push(value_out);
                }
                Bytecode::SatMod => {
                    let value_rhs = self.pop();
                    let value_lhs = self.pop();
                    let value_out = self.builder.ins().urem(value_lhs, value_rhs);
                    self.push(value_out);
                }
                Bytecode::And => {
                    let value_rhs = self.pop();
                    let value_lhs = self.pop();
                    let value_out = self.builder.ins().band(value_lhs, value_rhs);
                    self.push(value_out);
                }
                Bytecode::Or => {
                    let value_rhs = self.pop();
                    let value_lhs = self.pop();
                    let value_out = self.builder.ins().bor(value_lhs, value_rhs);
                    self.push(value_out);
                }
                Bytecode::Xor => {
                    let value_rhs = self.pop();
                    let value_lhs = self.pop();
                    let value_out = self.builder.ins().bxor(value_lhs, value_rhs);
                    self.push(value_out);
                }
                Bytecode::Not => {
                    let value = self.pop();
                    let value = self.builder.ins().bnot(value);
                    self.push(value);
                }
                Bytecode::AShl => {
                    let value_rhs = self.pop();
                    let value_lhs = self.pop();
                    let value_out = self.builder.ins().ishl(value_lhs, value_rhs);
                    self.push(value_out);
                }
                Bytecode::LShl => {
                    let value_rhs = self.pop();
                    let value_lhs = self.pop();
                    let value_out = self.builder.ins().ishl(value_lhs, value_rhs);
                    self.push(value_out);
                }
                Bytecode::AShr => {
                    let value_rhs = self.pop();
                    let value_lhs = self.pop();
                    let value_out = self.builder.ins().sshr(value_lhs, value_rhs);
                    self.push(value_out);
                }
                Bytecode::LShr => {
                    let value_rhs = self.pop();
                    let value_lhs = self.pop();
                    let value_out = self.builder.ins().ushr(value_lhs, value_rhs);
                    self.push(value_out);
                }
                Bytecode::Neg => {
                    let value = self.pop();
                    let value = self.builder.ins().ineg(value);
                    self.push(value);
                }
                Bytecode::Equal => {
                    let value_rhs = self.pop();
                    let value_lhs = self.pop();
                    let value_out = self.builder.ins().icmp(IntCC::Equal, value_lhs, value_rhs);
                    self.push(value_out);
                }
                Bytecode::NotEqual => {
                    let value_rhs = self.pop();
                    let value_lhs = self.pop();
                    let value_out = self.builder.ins().icmp(IntCC::NotEqual, value_lhs, value_rhs);
                    self.push(value_out);
                }
                Bytecode::Greater => {
                    let value_rhs = self.pop();
                    let value_lhs = self.pop();
                    let value_out = self.builder.ins().icmp(IntCC::UnsignedGreaterThan, value_lhs, value_rhs);
                    self.push(value_out);
                }
                Bytecode::Less => {
                    let value_rhs = self.pop();
                    let value_lhs = self.pop();
                    let value_out = self.builder.ins().icmp(IntCC::UnsignedLessThan, value_lhs, value_rhs);
                    self.push(value_out);
                }
                Bytecode::GreaterOrEqual => {
                    let value_rhs = self.pop();
                    let value_lhs = self.pop();
                    let value_out = self.builder.ins().icmp(IntCC::UnsignedGreaterThanOrEqual, value_lhs, value_rhs);
                    self.push(value_out);
                }
                Bytecode::LessOrEqual => {
                    let value_rhs = self.pop();
                    let value_lhs = self.pop();
                    let value_out = self.builder.ins().icmp(IntCC::UnsignedLessThanOrEqual, value_lhs, value_rhs);
                    self.push(value_out);
                }
                // TODO: implement conversions
                // TODO: implement array ops
                // TODO: implement object ops
                Bytecode::InvokeVirt(class_name, source_class, method_name) => {
                    let ctx = Context::new();
                    let sig = ctx.get_method_signature(*class_name as Symbol, *method_name as Symbol);
                    
                    let class_name_value = self.builder
                        .ins()
                        .iconst(cranelift::codegen::ir::types::I64, i64::from(i64::from_le_bytes(class_name.to_le_bytes())));
                    let source_class_value = match source_class {
                        Some(source_class) => {
                            self.builder
                                .ins()
                                .iconst(cranelift::codegen::ir::types::I64, i64::from(i64::from_le_bytes(source_class.to_le_bytes())))
                        }
                        None => {
                            self.builder
                                .ins()
                                .iconst(cranelift::codegen::ir::types::I64, i64::from(-1))
                        }
                    };
                    let method_name_value = self.builder
                        .ins()
                        .iconst(cranelift::codegen::ir::types::I64, i64::from(i64::from_le_bytes(method_name.to_le_bytes())));

                    let func_id = self.jit_utility_func.get("get_virtual_function").expect("get_virtual_function not loaded");
                    let method_args = self.get_call_arguments_as_vec();
                    let method_instructions = self.builder
                        .ins()
                        .call(*func_id, &[
                            method_args[0],
                            class_name_value,
                            source_class_value,
                            method_name_value,
                        ]);
                    let method_value = self.builder.inst_results(method_instructions)[0];
                    let call_args = self.get_call_arguments_as_vec();
                    let sig = self.builder.import_signature(sig);
                    
                    self.builder.ins().call_indirect(sig, method_value, &call_args);
                }
                // TODO: implement signal ops
                x => todo!("remaining ops"),
            }

        }

        Ok(())
    }
}
