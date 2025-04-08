
use std::collections::HashMap;

use codegen::{ir::{self, FuncRef}, CodegenError};
use cranelift::prelude::*;
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{FuncId, FuncOrDataId, Linkage, Module, ModuleError, ModuleResult};
use rowan_shared::bytecode::linked::Bytecode;

use rowan_shared::TypeTag;
use super::{tables::{string_table::StringTable, symbol_table::{SymbolEntry, SymbolTable}, vtable::{Function, FunctionValue}}, Context, Symbol};
use std::sync::Arc;
use crate::runtime;

pub struct JITController {
    builder_context: FunctionBuilderContext,
    context: codegen::Context,
    pub module: JITModule,
    jit_utility_func: Arc<HashMap<String, (FuncId, ir::function::Function)>>,
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
        builder.symbol("new_object", super::new_object as *const u8);
        builder.symbol("array64_init", super::stdlib::array64_init as *const u8);
        builder.symbol("array64_set", super::stdlib::array64_set as *const u8);
        builder.symbol("array64_get", super::stdlib::array64_set as *const u8);
        let mut module = JITModule::new(builder);

        let mut context = module.make_context();
        let mut builder_context = FunctionBuilderContext::new();

        let mut get_virt_func = module.make_signature();
        get_virt_func.params.push(AbiParam::new(cranelift::codegen::ir::types::I64));
        get_virt_func.params.push(AbiParam::new(cranelift::codegen::ir::types::I64));
        get_virt_func.params.push(AbiParam::new(cranelift::codegen::ir::types::I64));
        get_virt_func.params.push(AbiParam::new(cranelift::codegen::ir::types::I64));
        get_virt_func.returns.push(AbiParam::new(cranelift::codegen::ir::types::I64));

        let fn_id_get_virt_func = module.declare_function("get_virtual_function", Linkage::Import, &get_virt_func).unwrap();
        let mut get_virt_func_func = context.func.clone();
        get_virt_func_func.signature = get_virt_func;
        
        //let func_builder = FunctionBuilder::new(&mut context.func, &mut builder_context);
        //let get_virt_func_func = module.declare_func_in_func(fn_id, func_builder.func);

        //module.clear_context(&mut context);

        let mut new_object = module.make_signature();
        new_object.params.push(AbiParam::new(cranelift::codegen::ir::types::I64));
        new_object.returns.push(AbiParam::new(cranelift::codegen::ir::types::I64));

        let fn_id_new_object = module.declare_function("new_object", Linkage::Import, &new_object).unwrap();
        let mut new_object_func = context.func.clone();
        new_object_func.signature = new_object;


        let mut array64_init = module.make_signature();
        array64_init.params.push(AbiParam::new(cranelift::codegen::ir::types::I64));
        array64_init.params.push(AbiParam::new(cranelift::codegen::ir::types::I64));

        let fn_id_array64_init = module.declare_function("array64_init", Linkage::Import, &array64_init).unwrap();
        let mut array64_init_func = context.func.clone();
        array64_init_func.signature = array64_init;

        //let func_builder = FunctionBuilder::new(&mut context.func, &mut builder_context);
        //let new_object_func = module.declare_func_in_func(fn_id, func_builder.func);

        //module.clear_context(&mut context);

        let mut jit_utility_func = HashMap::new();
        jit_utility_func.insert(String::from("get_virtual_function"), (fn_id_get_virt_func, get_virt_func_func));
        jit_utility_func.insert(String::from("new_object"), (fn_id_new_object, new_object_func));
        jit_utility_func.insert(String::from("array64_init"), (fn_id_array64_init, array64_init_func));
        //module.finalize_definitions().unwrap();


        /*let func_id = module.declare_function("main", Linkage::Export, &new_object).unwrap();
        let mut func_builder = FunctionBuilder::new(&mut context.func, &mut builder_context);
        let block = func_builder.create_block();
        func_builder.append_block_params_for_function_params(block);
        func_builder.switch_to_block(block);
        func_builder.seal_block(block);
        let value = func_builder.ins().iconst(cranelift::codegen::ir::types::I64, 0);
        let result = func_builder.ins().call(new_object_func, &[value]);
        let value = func_builder.inst_results(result)[0];
        func_builder.ins().return_(&[]);
        func_builder.seal_all_blocks();
        module.define_function(func_id, &mut context).unwrap();
        module.clear_context(&mut context);*/
        
        
        //module.finalize_definitions().unwrap();

        Self {
            builder_context,
            context,
            module,
            jit_utility_func: Arc::new(jit_utility_func),
        }
    }
}

impl JITController {

    pub fn create_test_function(&mut self) {

        /*let mut new_object = self.module.make_signature();
        new_object.params.push(AbiParam::new(cranelift::codegen::ir::types::I64));
        new_object.returns.push(AbiParam::new(cranelift::codegen::ir::types::I64));
        let mut context = self.module.make_context();
        let mut builder_context = FunctionBuilderContext::new();
        
        let mut func_builder = FunctionBuilder::new(&mut context.func, &mut builder_context);
        let block = func_builder.create_block();
        func_builder.append_block_params_for_function_params(block);
        func_builder.switch_to_block(block);
        func_builder.seal_block(block);
        let value = func_builder.ins().iconst(cranelift::codegen::ir::types::I64, 0);
        let result = func_builder.ins().call(*self.jit_utility_func.get("new_object").unwrap(), &[value]);
        let value = func_builder.inst_results(result)[0];
        func_builder.ins().return_(&[]);
        func_builder.seal_all_blocks();
        let func_id = self.module.declare_function("main2", Linkage::Export, &new_object).unwrap();
        self.module.define_function(func_id, &mut context).unwrap();
        self.module.clear_context(&mut context);*/

    }

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

    pub fn get_utility_functions(&self) -> Arc<HashMap<String, (FuncId, ir::function::Function)>> {
        self.jit_utility_func.clone()
    }
}

unsafe impl Send for JITController {}
unsafe impl Sync for JITController {}


pub struct JITCompiler {
    builder_context: FunctionBuilderContext,
    context: codegen::Context,
    jit_utility_func: Arc<HashMap<String, (FuncId, ir::function::Function)>>,
}

impl JITCompiler {
    pub fn new(context: codegen::Context, jit_utility_func: Arc<HashMap<String, (FuncId, ir::function::Function)>>) -> JITCompiler {
        JITCompiler {
            builder_context: FunctionBuilderContext::new(),
            context,
            jit_utility_func

        }
    }

    pub fn compile(
        &mut self,
        function: &Function,
        module: &mut JITModule,
    ) -> Result<(), String> {

        let Ok(mut value) = function.value.write() else {
            panic!("Lock poisoned");
        };

        let FunctionValue::Bytecode(bytecode, id, sig) = &*value else {
            todo!("add error handling for non-bytecode value");
        };

        //println!("[Translating]");
        self.translate(&function.arguments, &bytecode, module)?;



        //println!("[Defining]");
        module
            .define_function(*id, &mut self.context)
            .map_err(|e| {
                match e {
                    ModuleError::Compilation(e) => {
                        match e {
                            CodegenError::Verifier(es) => {
                                es.0.iter().map(|e| format!("{}", e)).collect::<Vec<String>>().join("\n")
                            }
                            e => {
                                format!("{}", e)
                            }
                        }
                    }
                    e => format!("{}", e)
                }
            })?;

        module.clear_context(&mut self.context);

        module.finalize_definitions().unwrap();

        let code = module.get_finalized_function(*id);

        let new_function_value = FunctionValue::Compiled(code as *const (), sig.clone());

        *value = new_function_value;
        
        Ok(())
    }

    pub fn translate(
        &mut self,
        arg_types: &[runtime::class::TypeTag],
        bytecode: &[Bytecode],
        module: &mut JITModule
    ) -> Result<(), String> {

        let mut function_translator = FunctionTranslator::new(
            arg_types,
            &mut self.context,
            &mut self.builder_context,
            &self.jit_utility_func
        );

        //println!("[JIT] Translating function");
        function_translator.translate(bytecode, module)?;
        function_translator.builder.seal_all_blocks();
        function_translator.builder.finalize();
        

        Ok(())
    }

}


pub struct FunctionTranslator<'a> {
    builder: FunctionBuilder<'a>,
    call_args: [Option<(Variable, ir::Type)>; 256],
    variables: [Option<(Variable, ir::Type)>; 256],
    current_variable: usize,
    stack: Vec<Option<(Value, ir::Type)>>,
    blocks: Vec<Block>,
    current_block: usize,
    block_arg_types: HashMap<usize, Vec<ir::Type>>,
    jit_utility_func: &'a HashMap<String, (FuncId, ir::function::Function)>,
}

impl FunctionTranslator<'_> {
    pub fn new<'a>(
        arg_types: &[runtime::class::TypeTag],
        context: &'a mut codegen::Context,
        builder_context: &'a mut FunctionBuilderContext,
        jit_utility_func: &'a HashMap<String, (FuncId, ir::function::Function)>,
    ) -> FunctionTranslator<'a> {
        let mut builder = FunctionBuilder::new(&mut context.func, builder_context);

        let entry_block = builder.create_block();

        builder.append_block_params_for_function_params(entry_block);
        builder.switch_to_block(entry_block);
        //builder.seal_block(entry_block);

        let mut block_arg_types = HashMap::new();

        let mut stack = vec![];
        let mut variables = [None; 256];
        let mut current_variable = 0;
        let mut start_block_args = Vec::new();
        let block_params = builder.block_params(entry_block).iter().cloned().collect::<Vec<_>>();
        for (i, (value, ty)) in block_params.iter().zip(arg_types.iter()).enumerate() {
            let ty = match ty {
                runtime::class::TypeTag::U8 | runtime::class::TypeTag::I8 => ir::types::I8,
                runtime::class::TypeTag::U16 | runtime::class::TypeTag::I16 => ir::types::I16,
                runtime::class::TypeTag::U32 | runtime::class::TypeTag::I32 => ir::types::I32,
                runtime::class::TypeTag::U64 | runtime::class::TypeTag::I64 => ir::types::I64,
                runtime::class::TypeTag::F32 => ir::types::F32,
                runtime::class::TypeTag::F64 => ir::types::F64,
                runtime::class::TypeTag::Str => ir::types::I64,
                runtime::class::TypeTag::Void => ir::types::I64,
                runtime::class::TypeTag::Object => ir::types::I64,
            };
            start_block_args.push(ty.clone());
            let var = Variable::new(i);
            builder.declare_var(var, ty);
            builder.def_var(var, *value);
            variables[i] = Some((var, ty));
            current_variable += 1;
        }

        block_arg_types.insert(0, start_block_args);

        FunctionTranslator {
            builder,
            call_args: [None; 256],
            variables,
            current_variable,
            stack,
            blocks: vec![entry_block],
            current_block: 0,
            block_arg_types,
            jit_utility_func
        }
    }

    fn add_block(&mut self) {
        self.blocks.push(self.builder.create_block());
    }

    pub fn set_argument(&mut self, pos: u8, value: Value, ty: ir::Type) {
        println!("setting argument");
        if let Some((arg, arg_ty)) = &mut self.call_args[pos as usize] {
            if arg_ty != &ty {
                self.current_variable += 1;
                let new_arg = Variable::new(self.current_variable);
                self.builder.declare_var(new_arg, ty);
                self.builder.def_var(new_arg, value);
                *arg = new_arg;
                *arg_ty = ty;
            } else {
                self.builder.def_var(*arg, value);
            }
        } else {
            println!("creating new call argument");
            self.current_variable += 1;
            let new_arg = Variable::new(self.current_variable);
            self.builder.declare_var(new_arg, ty);
            self.builder.def_var(new_arg, value);
            self.call_args[pos as usize] = Some((new_arg, ty));
        }
    }

    pub fn set_var(&mut self, pos: u8, value: Value, ty: ir::Type) {
        //println!("setting var");
        if let Some((var, var_ty)) = &mut self.variables[pos as usize] {
            if var_ty != &ty {
                //println!("duplicate variable slot");
                self.current_variable += 1;
                let new_arg = Variable::new(self.current_variable);
                self.builder.declare_var(new_arg, ty);
                self.builder.def_var(new_arg, value);
                *var = new_arg;
                *var_ty = ty;
            } else {
                //println!("assigning to duplicate variable slot");
                self.builder.def_var(*var, value);
            }
        } else {
            //println!("creating new variable");
            self.current_variable += 1;
            let var = Variable::new(self.current_variable);
            self.builder.declare_var(var, ty);
            self.builder.def_var(var, value);
            self.variables[pos as usize] = Some((var, ty));
        }
    }

    pub fn get_var(&mut self, pos: u8) -> (Value, ir::Type) {
        match self.variables[pos as usize] {
            Some((var, ty)) => {
                (self.builder.use_var(var), ty)
            },
            _ => panic!("code was compiled wrong, empty value was not expected"),
        }
    }

    pub fn push(&mut self, value: Value, ty: ir::Type) {
        //println!("pushing");
        self.stack.push(Some((value, ty)));
    }

    pub fn pop(&mut self) -> (Value, ir::Type) {
        //println!("\tpopping");
        let out = self.stack.pop().flatten().unwrap();
        out
    }

    pub fn dup(&mut self) {
        let last = self.stack.last().cloned().flatten();
        self.stack.push(last);
    }

    pub fn swap(&mut self) {
        let top = self.stack.pop().flatten();
        let bottom = self.stack.pop().flatten();
        self.stack.push(bottom);
        self.stack.push(top);
    }

    pub fn get_call_arguments_as_vec(&mut self) -> Vec<Value> {
        println!("getting call arguments");
        let mut output = Vec::new();
        for value in self.call_args.iter_mut() {
            println!("\t{:?}", value);
            match value {
                Some((v, _)) => {
                    let value = self.builder.use_var(*v);
                    output.push(value)
                },
                None => break,
            }
            *value = None;
        }
        println!("\toutput: {:?}", output);
        output
    }

    pub fn get_args_as_vec(&self) -> Vec<Value> {
        //println!("{:?}", self.stack);
        let mut output = Vec::new();
        let mut iterator = self.stack.iter().flatten();
        while let Some((value, _)) = iterator.next() {
            output.push(*value);
        }
        output
    }

    pub fn get_args_as_vec_type(&self) -> Vec<Type> {
        let mut output = Vec::new();
        let mut iterator = self.stack.iter().flatten();
        while let Some((_, ty)) = iterator.next() {
            output.push(*ty);
        }
        output
    }

    fn restore_stack(&mut self, block_index: usize, stack: &[Value]) {
        //println!("restoring");
        let stack_iter = stack.iter();
        for ((value, arg_value), ty) in self.stack.iter_mut().zip(stack_iter).zip(self.block_arg_types.get(&block_index).unwrap()) {
            *value = Some((*arg_value, *ty));
        }
        let remaining = stack.iter()
            .zip(self.block_arg_types.get(&block_index).unwrap())
            .map(|(arg_value, ty)| Some((*arg_value, *ty)));
        self.stack.extend(remaining.skip(self.stack.len()));

    }


    pub fn translate(&mut self, bytecode: &[Bytecode], module: &mut JITModule) -> Result<(), String> {

        println!("Bytecode: {:#?}", bytecode);

        for bytecode in bytecode.iter() {
            match bytecode {
                Bytecode::Nop | Bytecode::Breakpoint => {}
                Bytecode::LoadU8(value) => {
                    let value = self.builder.ins().iconst(cranelift::codegen::ir::types::I8, i64::from(*value));
                    self.push(value, ir::types::I8);
                }
                Bytecode::LoadI8(value) => {
                    let value = self.builder.ins().iconst(cranelift::codegen::ir::types::I8, i64::from(*value));
                    self.push(value, ir::types::I8);
                }
                Bytecode::LoadU16(value) => {
                    let value = self.builder.ins().iconst(cranelift::codegen::ir::types::I16, i64::from(*value));
                    self.push(value, ir::types::I16);
                }
                Bytecode::LoadI16(value) => {
                    let value = self.builder.ins().iconst(cranelift::codegen::ir::types::I16, i64::from(*value));
                    self.push(value, ir::types::I16);
                }
                Bytecode::LoadU32(value) => {
                    let value = self.builder.ins().iconst(cranelift::codegen::ir::types::I32, i64::from(*value));
                    self.push(value, ir::types::I32);
                }
                Bytecode::LoadI32(value) => {
                    let value = self.builder.ins().iconst(cranelift::codegen::ir::types::I32, i64::from(*value));
                    self.push(value, ir::types::I32);
                }
                Bytecode::LoadU64(value) => {
                    let value = self.builder.ins().iconst(cranelift::codegen::ir::types::I64, i64::from(i64::from_le_bytes(value.to_le_bytes())));
                    self.push(value, ir::types::I64);
                }
                Bytecode::LoadI64(value) => {
                    let value = self.builder.ins().iconst(cranelift::codegen::ir::types::I64, i64::from(*value));
                    self.push(value, ir::types::I64);
                }
                Bytecode::LoadF32(value) => {
                    let value = self.builder.ins().f32const(*value);
                    self.push(value, ir::types::F32);
                }
                Bytecode::LoadF64(value) => {
                    let value = self.builder.ins().f64const(*value);
                    self.push(value, ir::types::F64);
                }
                Bytecode::LoadSymbol(symbol) => {
                    let value = self.builder.ins().iconst(cranelift::codegen::ir::types::I64, i64::from(i64::from_le_bytes(symbol.to_le_bytes())));
                    self.push(value, ir::types::I64);
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
                    let (value, ty) = self.pop();
                    self.set_var(*index, value, ty);
                }
                Bytecode::LoadLocal(index) => {
                    let (value, ty) = self.get_var(*index);
                    self.push(value, ty);
                }
                Bytecode::StoreArgument(index) => {
                    let (value, ty) = self.pop();
                    self.set_argument(*index, value, ty);
                }
                Bytecode::AddInt => {
                    let (value_rhs, ty) = self.pop();
                    let (value_lhs, _) = self.pop();
                    let value_out = self.builder.ins().iadd(value_lhs, value_rhs);
                    self.push(value_out, ty);
                }
                Bytecode::SubInt => {
                    let (value_rhs, ty) = self.pop();
                    let (value_lhs, _) = self.pop();
                    let value_out = self.builder.ins().isub(value_lhs, value_rhs);
                    self.push(value_out, ty);
                }
                Bytecode::MulInt => {
                    let (value_rhs, ty) = self.pop();
                    let (value_lhs, _) = self.pop();
                    let value_out = self.builder.ins().imul(value_lhs, value_rhs);
                    self.push(value_out, ty);
                }
                Bytecode::DivSigned => {
                    let (value_rhs, ty) = self.pop();
                    let (value_lhs, _) = self.pop();
                    let value_out = self.builder.ins().sdiv(value_lhs, value_rhs);
                    self.push(value_out, ty);
                }
                Bytecode::DivUnsigned => {
                    let (value_rhs, ty) = self.pop();
                    let (value_lhs, _) = self.pop();
                    let value_out = self.builder.ins().udiv(value_lhs, value_rhs);
                    self.push(value_out, ty);
                }
                Bytecode::ModSigned => {
                    let (value_rhs, ty) = self.pop();
                    let (value_lhs, _) = self.pop();
                    let value_out = self.builder.ins().srem(value_lhs, value_rhs);
                    self.push(value_out, ty);
                }
                Bytecode::ModUnsigned => {
                    let (value_rhs, ty) = self.pop();
                    let (value_lhs, _) = self.pop();
                    let value_out = self.builder.ins().urem(value_lhs, value_rhs);
                    self.push(value_out, ty);
                }
                Bytecode::SatAddIntUnsigned => {
                    let (value_rhs, ty) = self.pop();
                    let (value_lhs, _) = self.pop();
                    let value_out = self.builder.ins().uadd_sat(value_lhs, value_rhs);
                    self.push(value_out, ty);
                }
                Bytecode::SatSubIntUnsigned => {
                    let (value_rhs, ty) = self.pop();
                    let (value_lhs, _) = self.pop();
                    let value_out = self.builder.ins().usub_sat(value_lhs, value_rhs);
                    self.push(value_out, ty);
                }
                Bytecode::And => {
                    let (value_rhs, ty) = self.pop();
                    let (value_lhs, _) = self.pop();
                    let value_out = self.builder.ins().band(value_lhs, value_rhs);
                    self.push(value_out, ty);
                }
                Bytecode::Or => {
                    let (value_rhs, ty) = self.pop();
                    let (value_lhs, _) = self.pop();
                    let value_out = self.builder.ins().bor(value_lhs, value_rhs);
                    self.push(value_out, ty);
                }
                Bytecode::Xor => {
                    let (value_rhs, ty) = self.pop();
                    let (value_lhs, _) = self.pop();
                    let value_out = self.builder.ins().bxor(value_lhs, value_rhs);
                    self.push(value_out, ty);
                }
                Bytecode::Not => {
                    let (value, ty) = self.pop();
                    let value = self.builder.ins().bnot(value);
                    self.push(value, ty);
                }
                Bytecode::Shl => {
                    let (value_rhs, ty) = self.pop();
                    let (value_lhs, _) = self.pop();
                    let value_out = self.builder.ins().ishl(value_lhs, value_rhs);
                    self.push(value_out, ty);
                }
                Bytecode::AShr => {
                    let (value_rhs, ty) = self.pop();
                    let (value_lhs, _) = self.pop();
                    let value_out = self.builder.ins().sshr(value_lhs, value_rhs);
                    self.push(value_out, ty);
                }
                Bytecode::LShr => {
                    let (value_rhs, ty) = self.pop();
                    let (value_lhs, _) = self.pop();
                    let value_out = self.builder.ins().ushr(value_lhs, value_rhs);
                    self.push(value_out, ty);
                }
                Bytecode::Neg => {
                    let (value, ty) = self.pop();
                    let value = self.builder.ins().ineg(value);
                    self.push(value, ty);
                }
                Bytecode::EqualSigned => {
                    let (value_rhs,_) = self.pop();
                    let (value_lhs, _) = self.pop();
                    let value_out = self.builder.ins().icmp(IntCC::Equal, value_lhs, value_rhs);
                    self.push(value_out, ir::types::I8);
                }
                Bytecode::NotEqualSigned => {
                    let (value_rhs,_) = self.pop();
                    let (value_lhs, _) = self.pop();
                    let value_out = self.builder.ins().icmp(IntCC::NotEqual, value_lhs, value_rhs);
                    self.push(value_out, ir::types::I8);
                }
                Bytecode::EqualUnsigned => {
                    let (value_rhs,_) = self.pop();
                    let (value_lhs, _) = self.pop();
                    let value_out = self.builder.ins().icmp(IntCC::Equal, value_lhs, value_rhs);
                    self.push(value_out, ir::types::I8);
                }
                Bytecode::NotEqualUnsigned => {
                    let (value_rhs,_) = self.pop();
                    let (value_lhs, _) = self.pop();
                    let value_out = self.builder.ins().icmp(IntCC::NotEqual, value_lhs, value_rhs);
                    self.push(value_out, ir::types::I8);
                }
                Bytecode::GreaterUnsigned => {
                    let (value_rhs,_) = self.pop();
                    let (value_lhs, _) = self.pop();
                    let value_out = self.builder.ins().icmp(IntCC::UnsignedGreaterThan, value_lhs, value_rhs);
                    self.push(value_out, ir::types::I8);
                }
                Bytecode::LessUnsigned => {
                    let (value_rhs,_) = self.pop();
                    let (value_lhs, _) = self.pop();
                    let value_out = self.builder.ins().icmp(IntCC::UnsignedLessThan, value_lhs, value_rhs);
                    self.push(value_out, ir::types::I8);
                }
                Bytecode::GreaterOrEqualUnsigned => {
                    let (value_rhs,_) = self.pop();
                    let (value_lhs, _) = self.pop();
                    let value_out = self.builder.ins().icmp(IntCC::UnsignedGreaterThanOrEqual, value_lhs, value_rhs);
                    self.push(value_out, ir::types::I8);
                }
                Bytecode::LessOrEqualUnsigned => {
                    let (value_rhs,_) = self.pop();
                    let (value_lhs, _) = self.pop();
                    let value_out = self.builder.ins().icmp(IntCC::UnsignedLessThanOrEqual, value_lhs, value_rhs);
                    self.push(value_out, ir::types::I8);
                }
                Bytecode::GreaterSigned => {
                    let (value_rhs,_) = self.pop();
                    let (value_lhs, _) = self.pop();
                    let value_out = self.builder.ins().icmp(IntCC::SignedGreaterThan, value_lhs, value_rhs);
                    self.push(value_out, ir::types::I8);
                }
                Bytecode::LessSigned => {
                    let (value_rhs,_) = self.pop();
                    let (value_lhs, _) = self.pop();
                    let value_out = self.builder.ins().icmp(IntCC::SignedLessThan, value_lhs, value_rhs);
                    self.push(value_out, ir::types::I8);
                }
                Bytecode::GreaterOrEqualSigned => {
                    let (value_rhs,_) = self.pop();
                    let (value_lhs, _) = self.pop();
                    let value_out = self.builder.ins().icmp(IntCC::SignedGreaterThanOrEqual, value_lhs, value_rhs);
                    self.push(value_out, ir::types::I8);
                }
                Bytecode::LessOrEqualSigned => {
                    let (value_rhs,_) = self.pop();
                    let (value_lhs, _) = self.pop();
                    let value_out = self.builder.ins().icmp(IntCC::SignedLessThanOrEqual, value_lhs, value_rhs);
                    self.push(value_out, ir::types::I8);
                }
                Bytecode::EqualFloat => {
                    let (value_rhs,_) = self.pop();
                    let (value_lhs, _) = self.pop();
                    let value_out = self.builder.ins().fcmp(FloatCC::Equal, value_lhs, value_rhs);
                    self.push(value_out, ir::types::I8);
                }
                Bytecode::NotEqualFloat => {
                    let (value_rhs,_) = self.pop();
                    let (value_lhs, _) = self.pop();
                    let value_out = self.builder.ins().fcmp(FloatCC::NotEqual, value_lhs, value_rhs);
                    self.push(value_out, ir::types::I8);
                }
                Bytecode::GreaterFloat => {
                    let (value_rhs,_) = self.pop();
                    let (value_lhs, _) = self.pop();
                    let value_out = self.builder.ins().fcmp(FloatCC::GreaterThan, value_lhs, value_rhs);
                    self.push(value_out, ir::types::I8);
                }
                Bytecode::LessFloat => {
                    let (value_rhs,_) = self.pop();
                    let (value_lhs, _) = self.pop();
                    let value_out = self.builder.ins().fcmp(FloatCC::LessThan, value_lhs, value_rhs);
                    self.push(value_out, ir::types::I8);
                }
                Bytecode::GreaterOrEqualFloat => {
                    let (value_rhs,_) = self.pop();
                    let (value_lhs, _) = self.pop();
                    let value_out = self.builder.ins().fcmp(FloatCC::GreaterThanOrEqual, value_lhs, value_rhs);
                    self.push(value_out, ir::types::I8);
                }
                Bytecode::LessOrEqualFloat => {
                    let (value_rhs,_) = self.pop();
                    let (value_lhs, _) = self.pop();
                    let value_out = self.builder.ins().fcmp(FloatCC::UnorderedOrLessThanOrEqual, value_lhs, value_rhs);
                    self.push(value_out, ir::types::I8);
                }
                // TODO: implement conversions
                Bytecode::CreateArray(tag) => {
                    println!("create array");
                    let new_object_id = if let Some(id) = module.get_name("new_object") {
                        match id {
                            FuncOrDataId::Func(id) => id,
                            _ => unreachable!("cannot create array object from data id"),
                        }
                    } else {
                        let mut new_object = module.make_signature();
                        new_object.params.push(AbiParam::new(cranelift::codegen::ir::types::I64));
                        new_object.returns.push(AbiParam::new(cranelift::codegen::ir::types::I64));

                        let fn_id = module.declare_function("new_object", Linkage::Import, &new_object).unwrap();
                        fn_id
                    };

                    let new_object_func = module.declare_func_in_func(new_object_id, self.builder.func);

                    let array_symbol = match tag {
                        TypeTag::U8 | TypeTag::I8 => {
                            self.builder.ins().iconst(ir::types::I64, 8)
                        }
                        TypeTag::U16 | TypeTag::I16 => {
                            self.builder.ins().iconst(ir::types::I64, 14)
                        }
                        TypeTag::U32 | TypeTag::I32 => {
                            self.builder.ins().iconst(ir::types::I64, 18)
                        }
                        TypeTag::U64 | TypeTag::I64 => {
                            self.builder.ins().iconst(ir::types::I64, 22)
                        }
                        TypeTag::Object | TypeTag::Str | TypeTag::Void => {
                            self.builder.ins().iconst(ir::types::I64, 22)
                        }
                        TypeTag::F32 => {
                            self.builder.ins().iconst(ir::types::I64, 30)
                        }
                        TypeTag::F64 => {
                            self.builder.ins().iconst(ir::types::I64, 34)
                        }
                    };

                    let new_object = self.builder.ins().call(new_object_func, &[array_symbol]);
                    let value = self.builder.inst_results(new_object)[0];

                    let fun_name = match tag {
                        TypeTag::U8 | TypeTag::I8 => "array8_init",
                        TypeTag::U16 | TypeTag::I16 => "array16_init",
                        TypeTag::U32 | TypeTag::I32 => "array32_init",
                        TypeTag::U64 | TypeTag::I64 => "array64_init",
                        TypeTag::Object | TypeTag::Str | TypeTag::Void => "array64_init",
                        TypeTag::F32 => "arrayf32_init",
                        TypeTag::F64 => "arrayf64_init",
                    };

                    let initialize_array_id = if let Some(id) = module.get_name(fun_name) {
                        println!("initialize array {}", fun_name);
                        match id {
                            FuncOrDataId::Func(id) => id,
                            _ => unreachable!("cannot initialize array object from data id"),
                        }
                    } else {
                        let mut new_object = module.make_signature();
                        new_object.params.push(AbiParam::new(cranelift::codegen::ir::types::I64));
                        new_object.params.push(AbiParam::new(cranelift::codegen::ir::types::I64));

                        let fn_id = module.declare_function(fun_name, Linkage::Import, &new_object).unwrap();
                        fn_id
                    };



                    let (array_size, _) = self.pop();

                    let initialize_array = module.declare_func_in_func(initialize_array_id, self.builder.func);
                    let init_array = self.builder.ins().call(initialize_array, &[value, array_size]);
                    self.builder.inst_results(init_array);

                    self.push(value, ir::types::I64);

                }
                Bytecode::ArraySet(type_tag) => {
                    println!("array set");
                    let fun_name = match type_tag {
                        TypeTag::U8 | TypeTag::I8 => "array8_set",
                        TypeTag::U16 | TypeTag::I16 => "array16_set",
                        TypeTag::U32 | TypeTag::I32 => "array32_set",
                        TypeTag::U64 | TypeTag::I64 => "array64_set",
                        TypeTag::Object | TypeTag::Str | TypeTag::Void => "arrayobject_set",
                        TypeTag::F32 => "arrayf32_set",
                        TypeTag::F64 => "arrayf64_set",
                    };

                    let array_set = if let Some(id) = module.get_name(fun_name) {
                        match id {
                            FuncOrDataId::Func(id) => id,
                            _ => unreachable!("cannot initialize array object from data id"),
                        }
                    } else {
                        let mut new_object = module.make_signature();
                        new_object.params.push(AbiParam::new(cranelift::codegen::ir::types::I64));
                        new_object.params.push(AbiParam::new(cranelift::codegen::ir::types::I64));

                        let ty = match type_tag {
                            TypeTag::U8 | TypeTag::I8 => types::I8,
                            TypeTag::U16 | TypeTag::I16 => types::I16,
                            TypeTag::U32 | TypeTag::I32 => types::I32,
                            TypeTag::U64 | TypeTag::I64 => types::I64,
                            TypeTag::Object | TypeTag::Str | TypeTag::Void => types::I64,
                            TypeTag::F32 => types::F32,
                            TypeTag::F64 => types::F64,
                        };
                        new_object.params.push(AbiParam::new(ty));

                        let fn_id = module.declare_function(fun_name, Linkage::Import, &new_object).unwrap();
                        fn_id
                    };

                    let (value, _) = self.pop();
                    let (index, _) = self.pop();
                    let (array, _) = self.pop();

                    let array_set = module.declare_func_in_func(array_set, self.builder.func);
                    let array_set = self.builder.ins().call(array_set, &[array, index, value]);
                    self.builder.inst_results(array_set);
                    // TODO: add code for handling an index out of bounds exception
                }
                Bytecode::ArrayGet(type_tag) => {
                    println!("array get");
                    let fun_name = match type_tag {
                        TypeTag::U8 | TypeTag::I8 => "array8_get",
                        TypeTag::U16 | TypeTag::I16 => "array16_get",
                        TypeTag::U32 | TypeTag::I32 => "array32_get",
                        TypeTag::U64 | TypeTag::I64 => "array64_get",
                        TypeTag::Object | TypeTag::Str | TypeTag::Void => "arrayobject_get",
                        TypeTag::F32 => "arrayf32_get",
                        TypeTag::F64 => "arrayf64_get",
                    };

                    let array_get = if let Some(id) = module.get_name(fun_name) {
                        match id {
                            FuncOrDataId::Func(id) => id,
                            _ => unreachable!("cannot initialize array object from data id"),
                        }
                    } else {
                        let mut new_object = module.make_signature();
                        new_object.params.push(AbiParam::new(cranelift::codegen::ir::types::I64));
                        new_object.params.push(AbiParam::new(cranelift::codegen::ir::types::I64));

                        let ty = match type_tag {
                            TypeTag::U8 | TypeTag::I8 => types::I8,
                            TypeTag::U16 | TypeTag::I16 => types::I16,
                            TypeTag::U32 | TypeTag::I32 => types::I32,
                            TypeTag::U64 | TypeTag::I64 => types::I64,
                            TypeTag::Object | TypeTag::Str | TypeTag::Void => types::I64,
                            TypeTag::F32 => types::F32,
                            TypeTag::F64 => types::F64,
                        };
                        new_object.returns.push(AbiParam::new(ty));

                        let fn_id = module.declare_function(fun_name, Linkage::Import, &new_object).unwrap();
                        fn_id
                    };

                    let (index, _) = self.pop();
                    let (array, _) = self.pop();

                    let array_get = module.declare_func_in_func(array_get, self.builder.func);
                    let array_get = self.builder.ins().call(array_get, &[array, index]);
                    let value = self.builder.inst_results(array_get)[0];
                    // TODO: add code for handling an index out of bounds exception

                    let ty = match type_tag {
                        TypeTag::U8 | TypeTag::I8 => types::I8,
                        TypeTag::U16 | TypeTag::I16 => types::I16,
                        TypeTag::U32 | TypeTag::I32 => types::I32,
                        TypeTag::U64 | TypeTag::I64 => types::I64,
                        TypeTag::Object | TypeTag::Str | TypeTag::Void => types::I64,
                        TypeTag::F32 => types::F32,
                        TypeTag::F64 => types::F64,
                    };
                    self.push(value, ty);
                }
                // TODO: implement object ops
                Bytecode::NewObject(symbol) => {
                    let new_object_id = if let Some(id) = module.get_name("new_object") {
                        match id {
                            FuncOrDataId::Func(id) => id,
                            _ => unreachable!("cannot create array object from data id"),
                        }
                    } else {
                        let mut new_object = module.make_signature();
                        new_object.params.push(AbiParam::new(cranelift::codegen::ir::types::I64));
                        new_object.returns.push(AbiParam::new(cranelift::codegen::ir::types::I64));

                        let fn_id = module.declare_function("new_object", Linkage::Import, &new_object).unwrap();
                        fn_id
                    };

                    let new_object_func = module.declare_func_in_func(new_object_id, self.builder.func);

                    
                    
                    let object_symbol = self.builder.ins().iconst(cranelift::codegen::ir::types::I64, i64::from_le_bytes(symbol.to_le_bytes()));
                    let new_object = self.builder.ins().call(new_object_func, &[object_symbol]);
                    let value = self.builder.inst_results(new_object)[0];
                    self.push(value, ir::types::I64);
                }
                Bytecode::InvokeVirt(class_name, source_class, method_name) => {

                    let mut get_virt_func = module.make_signature();
                    get_virt_func.params.push(AbiParam::new(cranelift::codegen::ir::types::I64));
                    get_virt_func.params.push(AbiParam::new(cranelift::codegen::ir::types::I64));
                    get_virt_func.params.push(AbiParam::new(cranelift::codegen::ir::types::I64));
                    get_virt_func.params.push(AbiParam::new(cranelift::codegen::ir::types::I64));
                    get_virt_func.returns.push(AbiParam::new(cranelift::codegen::ir::types::I64));

                    let fn_id = module.declare_function("get_virtual_function", Linkage::Import, &get_virt_func).unwrap();

                    let get_virt_func_func = module.declare_func_in_func(fn_id, self.builder.func);


                    
                    let ctx = Context::new();
                    //println!("[translate] class_name from invoke virt: {}", class_name);
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

                    let method_args = self.get_call_arguments_as_vec();
                    let method_instructions = self.builder
                        .ins()
                        .call(get_virt_func_func, &[
                            method_args[0],
                            class_name_value,
                            source_class_value,
                            method_name_value,
                        ]);
                    let method_value = self.builder.inst_results(method_instructions)[0];
                    let return_type = sig.returns.first().cloned();
                    let sig = self.builder.import_signature(sig);
                    
                    let result = self.builder.ins().call_indirect(sig, method_value, &method_args);
                    let return_value = self.builder.inst_results(result);
                    if return_value.len() != 0 {
                        self.push(return_value[0], return_type.unwrap().value_type)
                    }
                }
                // TODO: implement signal ops
                Bytecode::StartBlock(index) => {
                    let block= self.blocks[*index as usize];
                    let params = self.builder.block_params(block).to_vec();
                    self.restore_stack(*index as usize, &params);
                    self.builder.switch_to_block(block);
                    self.current_block = *index as usize;
                }
                Bytecode::Goto(offset) => {
                    let block = (self.current_block as i64 + *offset) as usize;

                    while self.blocks.len() <= block + 1 {
                        self.add_block();
                    }

                    let block_args = self.get_args_as_vec_type();
                    self.block_arg_types.insert(block, block_args);

                    let stack = self.get_args_as_vec();

                    let block_index = block;
                    let block = &mut self.blocks[block];
                    if self.builder.block_params(*block).len() == 0 {
                        for ty in self.block_arg_types.get(&block_index).unwrap().iter() {
                            self.builder.append_block_param(*block, *ty);
                        }
                    }

                    self.builder.ins().jump(*block, &stack);
                }
                Bytecode::If(then_offset, else_offset) => {
                    let (value, _) = self.pop();
                    let then_block = (self.current_block as i64 + *then_offset) as usize;
                    let else_block = (self.current_block as i64 + *else_offset) as usize;

                    while self.blocks.len() <= then_block + 1 || self.blocks.len() <= else_block + 1 {
                        self.add_block();
                    }

                    let current_stack = self.get_args_as_vec();

                    let block_args = self.get_args_as_vec_type();
                    self.block_arg_types.insert(then_block, block_args);

                    let block_args= self.get_args_as_vec_type();
                    self.block_arg_types.insert(else_block, block_args);


                    let then_index = then_block;
                    let then_block = self.blocks[then_block];
                    if self.builder.block_params(then_block).len() == 0 {
                        for ty in self.block_arg_types.get(&then_index).unwrap().iter() {
                            self.builder.append_block_param(then_block, *ty);
                        }
                    }

                    let else_index = else_block;
                    let else_block = self.blocks[else_block];
                    if self.builder.block_params(else_block).len() == 0 {
                        for ty in self.block_arg_types.get(&else_index).unwrap().iter() {
                            self.builder.append_block_param(else_block, *ty);
                        }
                    }


                    self.builder.ins().brif(
                        value,
                        then_block,
                        &current_stack,
                        else_block,
                        &current_stack,
                    );
                }
                x => todo!("remaining ops {:?}", x),
            }

        }


        self.builder.ins().return_(&[]);

        Ok(())
    }
}
