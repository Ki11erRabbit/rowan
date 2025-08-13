use std::any::Any;
use std::collections::{HashMap, HashSet};
use std::ffi::CStr;
use std::sync::LazyLock;
use std::sync::mpsc::{Receiver, Sender};
use codegen::{ir::self, CodegenError};
use cranelift::prelude::*;
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{FuncId, FuncOrDataId, Linkage, Module, ModuleError, ModuleResult};
use rowan_shared::bytecode::linked::Bytecode;

use rowan_shared::TypeTag;
use super::{tables::vtable::{Function, FunctionValue}, Runtime, Symbol};
use cranelift::codegen::ir::BlockArg;
use cranelift_codegen::isa::unwind::UnwindInfo;
use log::trace;
use crate::context::{BytecodeContext, MethodName};
use crate::fake_lock::FakeLock;
use crate::runtime;

static JIT_SENDER: LazyLock<FakeLock<Option<Sender<MethodName>>>> = LazyLock::new(|| {
    FakeLock::new(None)
});

pub fn set_jit_sender(sender: Sender<MethodName>) {
    JIT_SENDER.write().replace(sender);
}

pub fn request_to_jit_method(name: MethodName) {
    JIT_SENDER.read()
        .as_ref()
        .map(|sender| sender.send(name));
}

pub struct JITController {
    builder_context: FunctionBuilderContext,
    context: codegen::Context,
    pub module: JITModule,
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
        builder.symbol("call_virtual_function", super::call_virtual_function as *const u8);
        builder.symbol("call_static_function", super::call_static_function as *const u8);
        builder.symbol("check_and_do_garbage_collection", Runtime::check_and_do_garbage_collection as *const u8);
        builder.symbol("new_object", super::new_object as *const u8);
        builder.symbol("array8_init", super::core::array8_init as *const u8);
        builder.symbol("array8_set", super::core::array8_set as *const u8);
        builder.symbol("array8_get", super::core::array8_get as *const u8);
        builder.symbol("array16_init", super::core::array16_init as *const u8);
        builder.symbol("array16_set", super::core::array16_set as *const u8);
        builder.symbol("array16_get", super::core::array16_get as *const u8);
        builder.symbol("array32_init", super::core::array32_init as *const u8);
        builder.symbol("array32_set", super::core::array32_set as *const u8);
        builder.symbol("array32_get", super::core::array32_get as *const u8);
        builder.symbol("array64_init", super::core::array64_init as *const u8);
        builder.symbol("array64_set", super::core::array64_set as *const u8);
        builder.symbol("array64_get", super::core::array64_get as *const u8);
        builder.symbol("arrayobject_set", super::core::arrayobject_set as *const u8);
        builder.symbol("arrayobject_get", super::core::arrayobject_get as *const u8);
        builder.symbol("arrayf32_init", super::core::arrayf32_init as *const u8);
        builder.symbol("arrayf32_set", super::core::arrayf32_set as *const u8);
        builder.symbol("arrayf32_get", super::core::arrayf32_get as *const u8);
        builder.symbol("arrayf64_init", super::core::arrayf64_init as *const u8);
        builder.symbol("arrayf64_set", super::core::arrayf64_set as *const u8);
        builder.symbol("arrayf64_get", super::core::arrayf64_get as *const u8);
        builder.symbol("context_should_unwind", Runtime::should_unwind as *const u8);
        builder.symbol("context_normal_return", Runtime::normal_return as *const u8);
        builder.symbol("member8_get", super::object::Object::get_8 as *const u8);
        builder.symbol("member16_get", super::object::Object::get_16 as *const u8);
        builder.symbol("member32_get", super::object::Object::get_32 as *const u8);
        builder.symbol("member64_get", super::object::Object::get_64 as *const u8);
        builder.symbol("memberobject_get", super::object::Object::get_object as *const u8);
        builder.symbol("memberf32_get", super::object::Object::get_f32 as *const u8);
        builder.symbol("memberf64_get", super::object::Object::get_f64 as *const u8);
        builder.symbol("member8_set", super::object::Object::set_8 as *const u8);
        builder.symbol("member16_set", super::object::Object::set_16 as *const u8);
        builder.symbol("member32_set", super::object::Object::set_32 as *const u8);
        builder.symbol("member64_set", super::object::Object::set_64 as *const u8);
        builder.symbol("memberobject_set", super::object::Object::set_object as *const u8);
        builder.symbol("memberf32_set", super::object::Object::set_f32 as *const u8);
        builder.symbol("memberf64_set", super::object::Object::set_f64 as *const u8);
        builder.symbol("static_member8_get", super::get_static_member8 as *const u8);
        builder.symbol("static_member16_get", super::get_static_member16 as *const u8);
        builder.symbol("static_member32_get", super::get_static_member32 as *const u8);
        builder.symbol("static_member64_get", super::get_static_member64 as *const u8);
        builder.symbol("static_memberf32_get", super::get_static_memberf32 as *const u8);
        builder.symbol("static_memberf64_get", super::get_static_memberf64 as *const u8);
        builder.symbol("static_memberobject_get", super::get_static_memberobject as *const u8);
        builder.symbol("static_member8_set", super::set_static_member8 as *const u8);
        builder.symbol("static_member16_set", super::set_static_member16 as *const u8);
        builder.symbol("static_member32_set", super::set_static_member32 as *const u8);
        builder.symbol("static_member64_set", super::set_static_member64 as *const u8);
        builder.symbol("static_memberf32_set", super::set_static_memberf32 as *const u8);
        builder.symbol("static_memberf64_set", super::set_static_memberf64 as *const u8);
        builder.symbol("static_memberobject_set", super::set_static_memberobject as *const u8);
        builder.symbol("store_argument_int8", BytecodeContext::store_argument_int8 as *const u8);
        builder.symbol("store_argument_int16", BytecodeContext::store_argument_int16 as *const u8);
        builder.symbol("store_argument_int32", BytecodeContext::store_argument_int32 as *const u8);
        builder.symbol("store_argument_int64", BytecodeContext::store_argument_int64 as *const u8);
        builder.symbol("store_argument_object", BytecodeContext::store_argument_object as *const u8);
        builder.symbol("store_argument_float32", BytecodeContext::store_argument_float32 as *const u8);
        builder.symbol("store_argument_float64", BytecodeContext::store_argument_float64 as *const u8);
        builder.symbol("fetch_argument_int8", BytecodeContext::fetch_argument_int8 as *const u8);
        builder.symbol("fetch_argument_int16", BytecodeContext::fetch_argument_int16 as *const u8);
        builder.symbol("fetch_argument_int32", BytecodeContext::fetch_argument_int32 as *const u8);
        builder.symbol("fetch_argument_int64", BytecodeContext::fetch_argument_int64 as *const u8);
        builder.symbol("fetch_argument_object", BytecodeContext::fetch_argument_object as *const u8);
        builder.symbol("fetch_argument_float32", BytecodeContext::fetch_argument_float32 as *const u8);
        builder.symbol("fetch_argument_float64", BytecodeContext::fetch_argument_float64 as *const u8);
        builder.symbol("fetch_return_int8", BytecodeContext::fetch_return_int8 as *const u8);
        builder.symbol("fetch_return_int16", BytecodeContext::fetch_return_int16 as *const u8);
        builder.symbol("fetch_return_int32", BytecodeContext::fetch_return_int32 as *const u8);
        builder.symbol("fetch_return_int64", BytecodeContext::fetch_return_int64 as *const u8);
        builder.symbol("fetch_return_object", BytecodeContext::fetch_return_object as *const u8);
        builder.symbol("fetch_return_float32", BytecodeContext::fetch_return_float32 as *const u8);
        builder.symbol("fetch_return_float64", BytecodeContext::fetch_return_float64 as *const u8);
        let mut module = JITModule::new(builder);

        let mut context = module.make_context();
        let mut builder_context = FunctionBuilderContext::new();



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
        }
    }
}

impl JITController {

    pub fn jit_thread(recv: Receiver<MethodName>) {
        loop {
            let result = match recv.recv() {
                Ok(method) => method,
                Err(_) => break,
            };
            //println!("doing jit");

            match result {
                MethodName::StaticMethod { class_symbol, method_name } => {
                    Runtime::jit_static_method(class_symbol, method_name);
                }
                MethodName::VirtualMethod {
                    object_class_symbol,
                    class_symbol,
                    source_class,
                    method_name
                } => {
                    Runtime::jit_virtual_method(object_class_symbol, class_symbol, source_class, method_name);
                }
            }
            //println!("done");
        }
    }

    pub fn create_signature(&self, args: &[TypeTag], return_type: &TypeTag) -> Signature {
        let mut signature = self.module.make_signature();
        signature.params.push(AbiParam::new(types::I64));
        for ty in args {
            let ty = match ty {
                TypeTag::U8 | TypeTag::I8 => cranelift::codegen::ir::types::I8,
                TypeTag::U16 | TypeTag::I16 => cranelift::codegen::ir::types::I16,
                TypeTag::U32 | TypeTag::I32 => cranelift::codegen::ir::types::I32,
                TypeTag::U64 | TypeTag::I64 | TypeTag::Object | TypeTag::Str => cranelift::codegen::ir::types::I64,
                TypeTag::F32 => cranelift::codegen::ir::types::F32,
                TypeTag::F64 => cranelift::codegen::ir::types::F64,
                TypeTag::Native => unreachable!("Native Type not ABI compatible"),
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
                TypeTag::Native => unreachable!("Native Type not ABI compatible"),
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

}

unsafe impl Send for JITController {}
unsafe impl Sync for JITController {}


pub struct JITCompiler {
    builder_context: FunctionBuilderContext,
    context: codegen::Context,
}

impl JITCompiler {
    pub fn new(context: codegen::Context) -> JITCompiler {
        JITCompiler {
            builder_context: FunctionBuilderContext::new(),
            context,
        }
    }

    pub fn compile(
        &mut self,
        function: &Function,
        module: &mut JITModule,
        name: &str,
    ) -> Result<(), String> {


        let bytecode = function.bytecode.as_ref();
        let id = {
            let value = &*function.value.lock().unwrap();
            match value {
                FunctionValue::Bytecode(id) => {
                    *id
                }
                _ => unreachable!("can compile only bytecode functions"),
            }
        };

        trace!("[Translating]");
        self.translate(&function.arguments, &function.return_type, &bytecode, module)?;



        //println!("[Defining]");
        module
            .define_function(id, &mut self.context)
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


        let compiled_code = self.context.compiled_code().unwrap();
        let stack_maps = compiled_code.buffer.user_stack_maps();
        let mut object_locations = Vec::new();
        for (location, _, map) in stack_maps {
            let objects = map.entries()
                .map(|(_, offset)| offset)
                .collect::<Vec<_>>();
            object_locations.push((*location, objects));
        }
        let locations = object_locations;
        let size = compiled_code.buffer.total_size();
        trace!("resulting function:\n{}", self.context.func);
        module.clear_context(&mut self.context);

        module.finalize_definitions().unwrap();
        
        let code = module.get_finalized_function(id) as *const ();
        rowan_unwind::register(code, size as usize);
        //println!("code: {:x}", code as usize);
        let mut object_locations = HashMap::new();
        locations.into_iter()
            .for_each(|(offset, objects)| {
                //println!("offset: {offset:x}");
                object_locations.insert(offset as usize + code as usize, objects);
            });

        //println!("object locations: {:#x?}", object_locations);

        let new_function_value = FunctionValue::Compiled(code, object_locations);

        *function.value.lock().unwrap() = new_function_value;

        Ok(())
    }

    pub fn translate(
        &mut self,
        arg_types: &[runtime::class::TypeTag],
        return_type: &runtime::class::TypeTag,
        bytecode: &[Bytecode],
        module: &mut JITModule
    ) -> Result<(), String> {

        self.context.func.signature.params.push(AbiParam::new(types::I64));

        for ty in arg_types {
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
                runtime::class::TypeTag::Sized(_) => unreachable!("Native Members are not ABI Compatible"),
            };

            self.context.func.signature.params.push(AbiParam::new(ty));

        }
        match return_type {
            runtime::class::TypeTag::U8 | runtime::class::TypeTag::I8 => self.context.func.signature.returns.push(AbiParam::new(ir::types::I8)),
            runtime::class::TypeTag::U16 | runtime::class::TypeTag::I16 => self.context.func.signature.returns.push(AbiParam::new(ir::types::I16)),
            runtime::class::TypeTag::U32 | runtime::class::TypeTag::I32 => self.context.func.signature.returns.push(AbiParam::new(ir::types::I32)),
            runtime::class::TypeTag::U64 | runtime::class::TypeTag::I64 => self.context.func.signature.returns.push(AbiParam::new(ir::types::I64)),
            runtime::class::TypeTag::F32 => self.context.func.signature.returns.push(AbiParam::new(ir::types::F32)),
            runtime::class::TypeTag::F64 => self.context.func.signature.returns.push(AbiParam::new(ir::types::F64)),
            runtime::class::TypeTag::Str => self.context.func.signature.returns.push(AbiParam::new(ir::types::I64)),
            runtime::class::TypeTag::Void => (),
            runtime::class::TypeTag::Object => self.context.func.signature.returns.push(AbiParam::new(ir::types::I64)),
            runtime::class::TypeTag::Sized(_) => unreachable!("Native Members are not ABI Compatible"),
        }

        let mut function_translator = FunctionTranslator::new(
            arg_types,
            return_type.clone(),
            &mut self.context,
            &mut self.builder_context,
        );

        //println!("[JIT] Translating function");
        function_translator.translate(bytecode, module)?;
        function_translator.builder.seal_all_blocks();
        function_translator.builder.finalize();
        

        Ok(())
    }

    pub fn compile_bytecode(
        &mut self,
        code: &Vec<Bytecode>,
        module: &mut JITModule,
        id: FuncId,
    ) -> Result<*const (), String> {
        self.context.func.signature.params.push(AbiParam::new(types::I64));

        let mut function_translator = FunctionTranslator::new(
            &[],
            runtime::class::TypeTag::Void,
            &mut self.context,
            &mut self.builder_context,
        );

        //println!("[JIT] Translating function");
        function_translator.translate(code, module)?;
        function_translator.builder.seal_all_blocks();
        function_translator.builder.finalize();

        module
            .define_function(id, &mut self.context)
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

        let code = module.get_finalized_function(id);

        Ok(code as *const ())
    }

}



pub struct FunctionTranslator<'a> {
    return_type: runtime::class::TypeTag,
    builder: FunctionBuilder<'a>,
    context_var: Variable,
    call_args: [Option<(ir::Type, bool)>; 256],
    variables: [Option<(Variable, ir::Type, bool)>; 256],
    current_variable: usize,
    stack: Vec<Option<(Value, ir::Type, bool)>>,
    blocks: Vec<Block>,
    current_block: usize,
    block_arg_types: HashMap<usize, Vec<ir::Type>>,
}

impl FunctionTranslator<'_> {
    pub fn new<'a>(
        arg_types: &[runtime::class::TypeTag],
        return_type: runtime::class::TypeTag,
        context: &'a mut codegen::Context,
        builder_context: &'a mut FunctionBuilderContext,
    ) -> FunctionTranslator<'a> {
        let mut builder = FunctionBuilder::new(&mut context.func, builder_context);


        let entry_block = builder.create_block();

        builder.append_block_params_for_function_params(entry_block);
        builder.switch_to_block(entry_block);
        builder.seal_block(entry_block);
        let start_block = builder.create_block();
        builder.ins().jump(start_block, &[]);

        let mut block_arg_types = HashMap::new();

        let mut variables = [None; 256];
        let mut current_variable = 1;
        let mut start_block_args = Vec::new();

        let context_var = Variable::new(0);
        builder.declare_var(context_var, types::I64);
        builder.def_var(context_var, builder.block_params(entry_block)[0]);

        let block_params = builder.block_params(entry_block).iter().cloned().collect::<Vec<_>>();
        for (i, (value, ty)) in block_params.iter().skip(1).zip(arg_types.iter()).enumerate() {
            let (ty, is_object) = match ty {
                runtime::class::TypeTag::U8 | runtime::class::TypeTag::I8 => (ir::types::I8, false),
                runtime::class::TypeTag::U16 | runtime::class::TypeTag::I16 => (ir::types::I16, false),
                runtime::class::TypeTag::U32 | runtime::class::TypeTag::I32 => (ir::types::I32, false),
                runtime::class::TypeTag::U64 | runtime::class::TypeTag::I64 => (ir::types::I64, false),
                runtime::class::TypeTag::F32 => (ir::types::F32, false),
                runtime::class::TypeTag::F64 => (ir::types::F64, false),
                runtime::class::TypeTag::Str => (ir::types::I64, false),
                runtime::class::TypeTag::Void => (ir::types::I64, false),
                runtime::class::TypeTag::Object => (ir::types::I64, true),
                runtime::class::TypeTag::Sized(_) => unreachable!("Native Members are not ABI Compatible"),
            };
            start_block_args.push(ty.clone());
            let var = Variable::new(i + 1);
            builder.declare_var(var, ty);
            builder.def_var(var, *value);
            if is_object {
                builder.declare_var_needs_stack_map(var);
            }
            variables[i] = Some((var, ty, is_object));
            current_variable += 1;
        }

        block_arg_types.insert(0, start_block_args);

        FunctionTranslator {
            return_type,
            builder,
            context_var,
            call_args: [None; 256],
            variables,
            current_variable,
            stack: Vec::new(),
            blocks: vec![start_block],
            current_block: 0,
            block_arg_types,
        }
    }

    fn add_block(&mut self) {
        self.blocks.push(self.builder.create_block());
    }

    pub fn set_argument(&mut self, module: &mut JITModule, pos: u8, value: Value, ty: ir::Type, is_object: bool) {
        // println!("setting argument");
        if let Some((arg_ty, arg_is_object)) = &mut self.call_args[pos as usize] {
            *arg_ty = ty;
            *arg_is_object = is_object;

            let name = match ty {
                types::I8 => "store_argument_int8",
                types::I16 => "store_argument_int16",
                types::I32 => "store_argument_int32",
                types::I64 if is_object => "store_argument_object",
                types::I64 if !is_object => "store_argument_int64",
                types::F32 => "store_argument_float32",
                types::F64 => "store_argument_float64",
                _ => unreachable!()
            };

            let store_argument = if let Some(id) = module.get_name(name) {
                match id {
                    FuncOrDataId::Func(id) => id,
                    _ => unreachable!("cannot create array object from data id"),
                }
            } else {
                let mut store_argument = module.make_signature();
                store_argument.params.push(AbiParam::new(cranelift::codegen::ir::types::I64));
                store_argument.params.push(AbiParam::new(ty));

                let fn_id = module.declare_function(name, Linkage::Import, &store_argument).unwrap();
                fn_id
            };

            let store_argument = module.declare_func_in_func(store_argument, self.builder.func);

            let context_value = self.builder.use_var(self.context_var);

            let _ = self.builder.ins()
                .call(store_argument, &[context_value, value]);
        }
    }

    pub fn set_var(&mut self, pos: u8, value: Value, ty: ir::Type, is_object: bool) {
        //println!("setting var");
        if let Some((var, var_ty, var_is_object)) = &mut self.variables[pos as usize] {
            if var_ty != &ty {
                //println!("duplicate variable slot");
                self.current_variable += 1;
                let new_arg = Variable::new(self.current_variable);
                if is_object {
                    self.builder.declare_var_needs_stack_map(new_arg);
                }
                self.builder.declare_var(new_arg, ty);
                self.builder.def_var(new_arg, value);
                *var = new_arg;
                *var_ty = ty;
                *var_is_object = is_object

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
            if is_object {
                self.builder.declare_var_needs_stack_map(var);
            }
            self.variables[pos as usize] = Some((var, ty, is_object));
        }
    }

    pub fn get_var(&mut self, pos: u8) -> (Value, ir::Type, bool) {
        match self.variables[pos as usize] {
            Some((var, ty, is_object)) => {
                let value = self.builder.use_var(var);
                
                if is_object {
                    self.builder.declare_value_needs_stack_map(value);
                }
                
                (value, ty, is_object)
            },
            _ => panic!("code was compiled wrong, empty value was not expected"),
        }
    }

    pub fn push(&mut self, value: Value, ty: ir::Type, is_object: bool) {
        //println!("\tpushing");
        self.stack.push(Some((value, ty, is_object)));
    }

    pub fn pop(&mut self) -> (Value, ir::Type, bool) {
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

    pub fn get_args_as_vec(&self) -> Vec<Value> {
        //println!("{:?}", self.stack);
        let mut output = Vec::new();
        let mut iterator = self.stack.iter().flatten();
        while let Some((value, _,_)) = iterator.next() {
            output.push(*value);
        }
        output
    }

    pub fn get_args_as_vec_type(&self) -> Vec<Type> {
        let mut output = Vec::new();
        let mut iterator = self.stack.iter().flatten();
        while let Some((_, ty,_)) = iterator.next() {
            output.push(*ty);
        }
        output
    }

    fn restore_stack(&mut self, block_index: usize, stack: &[Value]) {
        return;
        //println!("restoring");
        /*let stack_iter = stack.iter();
        for ((value, arg_value), ty) in self.stack.iter_mut().zip(stack_iter).zip(self.block_arg_types.get(&block_index).unwrap()) {
            *value = Some((*arg_value, *ty,));
        }
        let remaining = stack.iter()
            .zip(self.block_arg_types.get(&block_index).unwrap())
            .map(|(arg_value, ty)| Some((*arg_value, *ty)));
        self.stack.extend(remaining.skip(self.stack.len()));*/

    }


    pub fn translate(&mut self, bytecode: &[Bytecode], module: &mut JITModule) -> Result<(), String> {

        //println!("\nBytecode: {:#?}", bytecode);

        for bytecode in bytecode.iter() {
            //println!("{:?}", bytecode);
            match bytecode {
                Bytecode::Nop | Bytecode::Breakpoint => {}
                Bytecode::LoadU8(value) => {
                    let value = self.builder.ins().iconst(cranelift::codegen::ir::types::I8, i64::from(*value));
                    self.push(value, ir::types::I8, false);
                }
                Bytecode::LoadI8(value) => {
                    let value = self.builder.ins().iconst(cranelift::codegen::ir::types::I8, i64::from(*value));
                    self.push(value, ir::types::I8, false);
                }
                Bytecode::LoadU16(value) => {
                    let value = self.builder.ins().iconst(cranelift::codegen::ir::types::I16, i64::from(*value));
                    self.push(value, ir::types::I16, false);
                }
                Bytecode::LoadI16(value) => {
                    let value = self.builder.ins().iconst(cranelift::codegen::ir::types::I16, i64::from(*value));
                    self.push(value, ir::types::I16, false);
                }
                Bytecode::LoadU32(value) => {
                    let value = self.builder.ins().iconst(cranelift::codegen::ir::types::I32, i64::from(*value));
                    self.push(value, ir::types::I32, false);
                }
                Bytecode::LoadI32(value) => {
                    let value = self.builder.ins().iconst(cranelift::codegen::ir::types::I32, i64::from(*value));
                    self.push(value, ir::types::I32, false);
                }
                Bytecode::LoadU64(value) => {
                    let value = self.builder.ins().iconst(cranelift::codegen::ir::types::I64, i64::from(i64::from_le_bytes(value.to_le_bytes())));
                    self.push(value, ir::types::I64, false);
                }
                Bytecode::LoadI64(value) => {
                    let value = self.builder.ins().iconst(cranelift::codegen::ir::types::I64, i64::from(*value));
                    self.push(value, ir::types::I64, false);
                }
                Bytecode::LoadF32(value) => {
                    let value = self.builder.ins().f32const(*value);
                    self.push(value, ir::types::F32, false);
                }
                Bytecode::LoadF64(value) => {
                    let value = self.builder.ins().f64const(*value);
                    self.push(value, ir::types::F64, false);
                }
                Bytecode::LoadSymbol(symbol) => {
                    let value = self.builder.ins().iconst(cranelift::codegen::ir::types::I64, i64::from(i64::from_le_bytes(symbol.to_le_bytes())));
                    self.push(value, ir::types::I64, false);
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
                    let (value, ty, is_object) = self.pop();
                    self.set_var(*index, value, ty, is_object);
                }
                Bytecode::LoadLocal(index) => {
                    let (value, ty, is_object) = self.get_var(*index);
                    self.push(value, ty, is_object);
                }
                Bytecode::StoreArgument(index) => {
                    let (value, ty, is_object) = self.pop();
                    self.set_argument(module, *index, value, ty, is_object);
                }
                Bytecode::AddInt => {
                    let (value_rhs, ty, _) = self.pop();
                    let (value_lhs, _, _) = self.pop();
                    let value_out = self.builder.ins().iadd(value_lhs, value_rhs);
                    self.push(value_out, ty, false);
                }
                Bytecode::SubInt => {
                    let (value_rhs, ty, _) = self.pop();
                    let (value_lhs, _, _) = self.pop();
                    let value_out = self.builder.ins().isub(value_lhs, value_rhs);
                    self.push(value_out, ty, false);
                }
                Bytecode::MulInt => {
                    let (value_rhs, ty, _) = self.pop();
                    let (value_lhs, _, _) = self.pop();
                    let value_out = self.builder.ins().imul(value_lhs, value_rhs);
                    self.push(value_out, ty, false);
                }
                Bytecode::DivSigned => {
                    let (value_rhs, ty, _) = self.pop();
                    let (value_lhs, _, _) = self.pop();
                    let value_out = self.builder.ins().sdiv(value_lhs, value_rhs);
                    self.push(value_out, ty, false);
                }
                Bytecode::DivUnsigned => {
                    let (value_rhs, ty, _) = self.pop();
                    let (value_lhs, _, _) = self.pop();
                    let value_out = self.builder.ins().udiv(value_lhs, value_rhs);
                    self.push(value_out, ty, false);
                }
                Bytecode::ModSigned => {
                    let (value_rhs, ty, _) = self.pop();
                    let (value_lhs, _, _) = self.pop();
                    let value_out = self.builder.ins().srem(value_lhs, value_rhs);
                    self.push(value_out, ty, false);
                }
                Bytecode::ModUnsigned => {
                    let (value_rhs, ty, _) = self.pop();
                    let (value_lhs, _, _) = self.pop();
                    let value_out = self.builder.ins().urem(value_lhs, value_rhs);
                    self.push(value_out, ty, false);
                }
                Bytecode::SatAddIntUnsigned => {
                    let (value_rhs, ty, _) = self.pop();
                    let (value_lhs, _, _) = self.pop();
                    let value_out = self.builder.ins().uadd_sat(value_lhs, value_rhs);
                    self.push(value_out, ty, false);
                }
                Bytecode::SatSubIntUnsigned => {
                    let (value_rhs, ty, _) = self.pop();
                    let (value_lhs, _, _) = self.pop();
                    let value_out = self.builder.ins().usub_sat(value_lhs, value_rhs);
                    self.push(value_out, ty, false);
                }
                Bytecode::AddFloat => {
                    let (value_rhs, ty, _) = self.pop();
                    let (value_lhs, _, _) = self.pop();
                    let value_out = self.builder.ins().fadd(value_lhs, value_rhs);
                    self.push(value_out, ty, false);
                }
                Bytecode::SubFloat => {
                    let (value_rhs, ty, _) = self.pop();
                    let (value_lhs, _, _) = self.pop();
                    let value_out = self.builder.ins().fsub(value_lhs, value_rhs);
                    self.push(value_out, ty, false);
                }
                Bytecode::MulFloat => {
                    let (value_rhs, ty, _) = self.pop();
                    let (value_lhs, _, _) = self.pop();
                    let value_out = self.builder.ins().fmul(value_lhs, value_rhs);
                    self.push(value_out, ty, false);
                }
                Bytecode::DivFloat => {
                    let (value_rhs, ty, _) = self.pop();
                    let (value_lhs, _, _) = self.pop();
                    let value_out = self.builder.ins().fdiv(value_lhs, value_rhs);
                    self.push(value_out, ty, false);
                }
                Bytecode::ModFloat => {
                    let (value_rhs, ty, _) = self.pop();
                    let (value_lhs, _, _) = self.pop();
                    let value_out = self.builder.ins().fdiv(value_lhs, value_rhs);
                    self.push(value_out, ty, false);
                }
                Bytecode::And => {
                    let (value_rhs, ty, _) = self.pop();
                    let (value_lhs, _, _) = self.pop();
                    let value_out = self.builder.ins().band(value_lhs, value_rhs);
                    self.push(value_out, ty, false);
                }
                Bytecode::Or => {
                    let (value_rhs, ty, _) = self.pop();
                    let (value_lhs, _, _) = self.pop();
                    let value_out = self.builder.ins().bor(value_lhs, value_rhs);
                    self.push(value_out, ty, false);
                }
                Bytecode::Xor => {
                    let (value_rhs, ty, _) = self.pop();
                    let (value_lhs, _, _) = self.pop();
                    let value_out = self.builder.ins().bxor(value_lhs, value_rhs);
                    self.push(value_out, ty, false);
                }
                Bytecode::Not => {
                    let (value, ty, _) = self.pop();
                    let value = self.builder.ins().bnot(value);
                    self.push(value, ty, false);
                }
                Bytecode::Shl => {
                    let (value_rhs, ty, _) = self.pop();
                    let (value_lhs, _, _) = self.pop();
                    let value_out = self.builder.ins().ishl(value_lhs, value_rhs);
                    self.push(value_out, ty, false);
                }
                Bytecode::AShr => {
                    let (value_rhs, ty, _) = self.pop();
                    let (value_lhs, _, _) = self.pop();
                    let value_out = self.builder.ins().sshr(value_lhs, value_rhs);
                    self.push(value_out, ty, false);
                }
                Bytecode::LShr => {
                    let (value_rhs, ty, _) = self.pop();
                    let (value_lhs, _, _) = self.pop();
                    let value_out = self.builder.ins().ushr(value_lhs, value_rhs);
                    self.push(value_out, ty, false);
                }
                Bytecode::Neg => {
                    let (value, ty, _) = self.pop();
                    let value = self.builder.ins().ineg(value);
                    self.push(value, ty, false);
                }
                Bytecode::EqualSigned => {
                    let (value_rhs, _, _) = self.pop();
                    let (value_lhs, _, _) = self.pop();
                    let value_out = self.builder.ins().icmp(IntCC::Equal, value_lhs, value_rhs);
                    self.push(value_out, ir::types::I8, false);
                }
                Bytecode::NotEqualSigned => {
                    let (value_rhs, _, _) = self.pop();
                    let (value_lhs, _, _) = self.pop();
                    let value_out = self.builder.ins().icmp(IntCC::NotEqual, value_lhs, value_rhs);
                    self.push(value_out, ir::types::I8, false);
                }
                Bytecode::EqualUnsigned => {
                    let (value_rhs, _, _) = self.pop();
                    let (value_lhs, _, _) = self.pop();
                    let value_out = self.builder.ins().icmp(IntCC::Equal, value_lhs, value_rhs);
                    self.push(value_out, ir::types::I8, false);
                }
                Bytecode::NotEqualUnsigned => {
                    let (value_rhs, _, _) = self.pop();
                    let (value_lhs, _, _) = self.pop();
                    let value_out = self.builder.ins().icmp(IntCC::NotEqual, value_lhs, value_rhs);
                    self.push(value_out, ir::types::I8, false);
                }
                Bytecode::GreaterUnsigned => {
                    let (value_rhs, _, _) = self.pop();
                    let (value_lhs, _, _) = self.pop();
                    let value_out = self.builder.ins().icmp(IntCC::UnsignedGreaterThan, value_lhs, value_rhs);
                    self.push(value_out, ir::types::I8, false);
                }
                Bytecode::LessUnsigned => {
                    let (value_rhs, _, _) = self.pop();
                    let (value_lhs, _, _) = self.pop();
                    let value_out = self.builder.ins().icmp(IntCC::UnsignedLessThan, value_lhs, value_rhs);
                    self.push(value_out, ir::types::I8, false);
                }
                Bytecode::GreaterOrEqualUnsigned => {
                    let (value_rhs, _, _) = self.pop();
                    let (value_lhs, _, _) = self.pop();
                    let value_out = self.builder.ins().icmp(IntCC::UnsignedGreaterThanOrEqual, value_lhs, value_rhs);
                    self.push(value_out, ir::types::I8, false);
                }
                Bytecode::LessOrEqualUnsigned => {
                    let (value_rhs, _, _) = self.pop();
                    let (value_lhs, _, _) = self.pop();
                    let value_out = self.builder.ins().icmp(IntCC::UnsignedLessThanOrEqual, value_lhs, value_rhs);
                    self.push(value_out, ir::types::I8, false);
                }
                Bytecode::GreaterSigned => {
                    let (value_rhs, _, _) = self.pop();
                    let (value_lhs, _, _) = self.pop();
                    let value_out = self.builder.ins().icmp(IntCC::SignedGreaterThan, value_lhs, value_rhs);
                    self.push(value_out, ir::types::I8, false);
                }
                Bytecode::LessSigned => {
                    let (value_rhs, _, _) = self.pop();
                    let (value_lhs, _, _) = self.pop();
                    let value_out = self.builder.ins().icmp(IntCC::SignedLessThan, value_lhs, value_rhs);
                    self.push(value_out, ir::types::I8, false);
                }
                Bytecode::GreaterOrEqualSigned => {
                    let (value_rhs, _, _) = self.pop();
                    let (value_lhs, _, _) = self.pop();
                    let value_out = self.builder.ins().icmp(IntCC::SignedGreaterThanOrEqual, value_lhs, value_rhs);
                    self.push(value_out, ir::types::I8, false);
                }
                Bytecode::LessOrEqualSigned => {
                    let (value_rhs, _, _) = self.pop();
                    let (value_lhs, _, _) = self.pop();
                    let value_out = self.builder.ins().icmp(IntCC::SignedLessThanOrEqual, value_lhs, value_rhs);
                    self.push(value_out, ir::types::I8, false);
                }
                Bytecode::EqualFloat => {
                    let (value_rhs, _, _) = self.pop();
                    let (value_lhs, _, _) = self.pop();
                    let value_out = self.builder.ins().fcmp(FloatCC::Equal, value_lhs, value_rhs);
                    self.push(value_out, ir::types::I8, false);
                }
                Bytecode::NotEqualFloat => {
                    let (value_rhs, _, _) = self.pop();
                    let (value_lhs, _, _) = self.pop();
                    let value_out = self.builder.ins().fcmp(FloatCC::NotEqual, value_lhs, value_rhs);
                    self.push(value_out, ir::types::I8, false);
                }
                Bytecode::GreaterFloat => {
                    let (value_rhs, _, _) = self.pop();
                    let (value_lhs, _, _) = self.pop();
                    let value_out = self.builder.ins().fcmp(FloatCC::GreaterThan, value_lhs, value_rhs);
                    self.push(value_out, ir::types::I8, false);
                }
                Bytecode::LessFloat => {
                    let (value_rhs, _, _) = self.pop();
                    let (value_lhs, _, _) = self.pop();
                    let value_out = self.builder.ins().fcmp(FloatCC::LessThan, value_lhs, value_rhs);
                    self.push(value_out, ir::types::I8, false);
                }
                Bytecode::GreaterOrEqualFloat => {
                    let (value_rhs, _, _) = self.pop();
                    let (value_lhs, _, _) = self.pop();
                    let value_out = self.builder.ins().fcmp(FloatCC::GreaterThanOrEqual, value_lhs, value_rhs);
                    self.push(value_out, ir::types::I8, false);
                }
                Bytecode::LessOrEqualFloat => {
                    let (value_rhs, _, _) = self.pop();
                    let (value_lhs, _, _) = self.pop();
                    let value_out = self.builder.ins().fcmp(FloatCC::UnorderedOrLessThanOrEqual, value_lhs, value_rhs);
                    self.push(value_out, ir::types::I8, false);
                }
                // TODO: implement conversions
                Bytecode::CreateArray(tag) => {
                    // println!("create array");
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
                            self.builder.ins().iconst(ir::types::I64, Runtime::get_class_symbol("Array8") as i64)
                        }
                        TypeTag::U16 | TypeTag::I16 => {
                            self.builder.ins().iconst(ir::types::I64, Runtime::get_class_symbol("Array16") as i64)
                        }
                        TypeTag::U32 | TypeTag::I32 => {
                            self.builder.ins().iconst(ir::types::I64, Runtime::get_class_symbol("Array32") as i64)
                        }
                        TypeTag::U64 | TypeTag::I64 => {
                            self.builder.ins().iconst(ir::types::I64, Runtime::get_class_symbol("Array64") as i64)
                        }
                        TypeTag::Object | TypeTag::Str | TypeTag::Void => {
                            self.builder.ins().iconst(ir::types::I64, Runtime::get_class_symbol("Arrayobject") as i64)
                        }
                        TypeTag::F32 => {
                            self.builder.ins().iconst(ir::types::I64, Runtime::get_class_symbol("Arrayf32") as i64)
                        }
                        TypeTag::F64 => {
                            self.builder.ins().iconst(ir::types::I64, Runtime::get_class_symbol("Arrayf64") as i64)
                        }
                        TypeTag::Native => unreachable!("Native Type not ABI compatible"),
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
                        TypeTag::Native => unreachable!("Native Type not ABI compatible"),
                    };

                    let initialize_array_id = if let Some(id) = module.get_name(fun_name) {
                        // println!("initialize array {}", fun_name);
                        match id {
                            FuncOrDataId::Func(id) => id,
                            _ => unreachable!("cannot initialize array object from data id"),
                        }
                    } else {
                        let mut new_object = module.make_signature();
                        new_object.params.push(AbiParam::new(cranelift::codegen::ir::types::I64));
                        new_object.params.push(AbiParam::new(cranelift::codegen::ir::types::I64));
                        new_object.params.push(AbiParam::new(cranelift::codegen::ir::types::I64));

                        let fn_id = module.declare_function(fun_name, Linkage::Import, &new_object).unwrap();
                        fn_id
                    };
                    let context_value = self.builder.use_var(self.context_var);

                    let (array_size, _, _) = self.pop();

                    let initialize_array = module.declare_func_in_func(initialize_array_id, self.builder.func);
                    let init_array = self.builder.ins().call(initialize_array, &[context_value, value, array_size]);
                    self.builder.inst_results(init_array);

                    self.builder.declare_value_needs_stack_map(value);
                    self.push(value, ir::types::I64, true);

                }
                Bytecode::ArraySet(type_tag) => {
                    // println!("array set");
                    let fun_name = match type_tag {
                        TypeTag::U8 | TypeTag::I8 => "array8_set",
                        TypeTag::U16 | TypeTag::I16 => "array16_set",
                        TypeTag::U32 | TypeTag::I32 => "array32_set",
                        TypeTag::U64 | TypeTag::I64 => "array64_set",
                        TypeTag::Object | TypeTag::Str | TypeTag::Void => "arrayobject_set",
                        TypeTag::F32 => "arrayf32_set",
                        TypeTag::F64 => "arrayf64_set",
                        TypeTag::Native => unreachable!("Native Type not ABI compatible"),
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
                        new_object.params.push(AbiParam::new(cranelift::codegen::ir::types::I64));

                        let ty = match type_tag {
                            TypeTag::U8 | TypeTag::I8 => types::I8,
                            TypeTag::U16 | TypeTag::I16 => types::I16,
                            TypeTag::U32 | TypeTag::I32 => types::I32,
                            TypeTag::U64 | TypeTag::I64 => types::I64,
                            TypeTag::Object | TypeTag::Str | TypeTag::Void => types::I64,
                            TypeTag::F32 => types::F32,
                            TypeTag::F64 => types::F64,
                            TypeTag::Native => unreachable!("Native Type not ABI compatible"),
                        };
                        new_object.params.push(AbiParam::new(ty));

                        let fn_id = module.declare_function(fun_name, Linkage::Import, &new_object).unwrap();
                        fn_id
                    };

                    let context_value = self.builder.use_var(self.context_var);

                    let (value, _, _) = self.pop();
                    let (index, _, _) = self.pop();
                    let (array, _, _) = self.pop();

                    let array_set = module.declare_func_in_func(array_set, self.builder.func);
                    let array_set = self.builder.ins().call(array_set, &[context_value, array, index, value]);
                    self.builder.inst_results(array_set);

                    self.create_bail_block(module, None, &[]);
                }
                Bytecode::ArrayGet(type_tag) => {
                    // println!("array get");
                    let mut is_object = false;
                    let fun_name = match type_tag {
                        TypeTag::U8 | TypeTag::I8 => "array8_get",
                        TypeTag::U16 | TypeTag::I16 => "array16_get",
                        TypeTag::U32 | TypeTag::I32 => "array32_get",
                        TypeTag::U64 | TypeTag::I64 => "array64_get",
                        TypeTag::Object | TypeTag::Str | TypeTag::Void => {
                            is_object = true;
                            "arrayobject_get"
                        },
                        TypeTag::F32 => "arrayf32_get",
                        TypeTag::F64 => "arrayf64_get",
                        TypeTag::Native => unreachable!("Native Type not ABI compatible"),
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
                        new_object.params.push(AbiParam::new(cranelift::codegen::ir::types::I64));

                        let ty = match type_tag {
                            TypeTag::U8 | TypeTag::I8 => types::I8,
                            TypeTag::U16 | TypeTag::I16 => types::I16,
                            TypeTag::U32 | TypeTag::I32 => types::I32,
                            TypeTag::U64 | TypeTag::I64 => types::I64,
                            TypeTag::Object | TypeTag::Str | TypeTag::Void => types::I64,
                            TypeTag::F32 => types::F32,
                            TypeTag::F64 => types::F64,
                            TypeTag::Native => unreachable!("Native Type not ABI compatible"),
                        };
                        new_object.returns.push(AbiParam::new(ty));

                        let fn_id = module.declare_function(fun_name, Linkage::Import, &new_object).unwrap();
                        fn_id
                    };
                    let context_value = self.builder.use_var(self.context_var);

                    let (index, _, _) = self.pop();
                    let (array, _, _) = self.pop();

                    let array_get = module.declare_func_in_func(array_get, self.builder.func);
                    let array_get = self.builder.ins().call(array_get, &[context_value, array, index]);
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
                        TypeTag::Native => unreachable!("Native Type not ABI compatible"),
                    };

                    if is_object {
                        self.builder.declare_value_needs_stack_map(value);
                    }

                    self.create_bail_block(module, Some(ty), &[BlockArg::Value(value)]);

                    self.push(value, ty, is_object);
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
                    self.builder.declare_value_needs_stack_map(value);
                    self.push(value, ir::types::I64, true);
                }
                Bytecode::GetField(class_name, parent_symbol, offset, type_tag) => {
                    let mut is_object = false;
                    let fun_name = match type_tag {
                        TypeTag::U8 | TypeTag::I8 => "member8_get",
                        TypeTag::U16 | TypeTag::I16 => "member16_get",
                        TypeTag::U32 | TypeTag::I32 => "member32_get",
                        TypeTag::U64 | TypeTag::I64 => "member64_get",
                        TypeTag::Object | TypeTag::Str | TypeTag::Void => {
                            is_object = true;
                            "memberobject_get"
                        },
                        TypeTag::F32 => "memberf32_get",
                        TypeTag::F64 => "memberf64_get",
                        TypeTag::Native => unreachable!("Native Type not ABI compatible"),
                    };

                    let member_get = if let Some(id) = module.get_name(fun_name) {
                        match id {
                            FuncOrDataId::Func(id) => id,
                            _ => unreachable!("cannot initialize array object from data id"),
                        }
                    } else {
                        let mut new_object = module.make_signature();
                        new_object.params.push(AbiParam::new(cranelift::codegen::ir::types::I64));
                        new_object.params.push(AbiParam::new(cranelift::codegen::ir::types::I64));
                        new_object.params.push(AbiParam::new(cranelift::codegen::ir::types::I64));
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
                            TypeTag::Native => unreachable!("Native Type not ABI compatible"),
                        };
                        new_object.returns.push(AbiParam::new(ty));

                        let fn_id = module.declare_function(fun_name, Linkage::Import, &new_object).unwrap();
                        fn_id
                    };

                    let context_value = self.builder.use_var(self.context_var);
                    let (this_value, _, _) = self.pop();
                    self.builder.declare_value_needs_stack_map(this_value);
                    
                    let class_symbol = self.builder.ins().iconst(cranelift::codegen::ir::types::I64, i64::from_le_bytes(class_name.to_le_bytes()));
                    let parent_symbol = self.builder.ins().iconst(cranelift::codegen::ir::types::I64, i64::from_le_bytes(parent_symbol.to_le_bytes()));
                    let offset = self.builder.ins().iconst(cranelift::codegen::ir::types::I64, i64::from_le_bytes(offset.to_le_bytes()));

                    let member_get = module.declare_func_in_func(member_get, self.builder.func);
                    let member_get = self.builder.ins().call(member_get, &[context_value, this_value, class_symbol, parent_symbol, offset]);
                    let value = self.builder.inst_results(member_get)[0];
                    // TODO: add code for handling an index out of bounds exception

                    let ty = match type_tag {
                        TypeTag::U8 | TypeTag::I8 => types::I8,
                        TypeTag::U16 | TypeTag::I16 => types::I16,
                        TypeTag::U32 | TypeTag::I32 => types::I32,
                        TypeTag::U64 | TypeTag::I64 => types::I64,
                        TypeTag::Object | TypeTag::Str | TypeTag::Void => types::I64,
                        TypeTag::F32 => types::F32,
                        TypeTag::F64 => types::F64,
                        TypeTag::Native => unreachable!("Native Type not ABI compatible"),
                    };

                    if is_object {
                        self.builder.declare_value_needs_stack_map(value);
                    }

                    self.create_bail_block(module, Some(ty), &[BlockArg::Value(value)]);

                    self.push(value, ty, is_object);
                }
                Bytecode::SetField(class_name, parent_symbol, offset, type_tag) => {
                    let fun_name = match type_tag {
                        TypeTag::U8 | TypeTag::I8 => "member8_set",
                        TypeTag::U16 | TypeTag::I16 => "member16_set",
                        TypeTag::U32 | TypeTag::I32 => "member32_set",
                        TypeTag::U64 | TypeTag::I64 => "member64_set",
                        TypeTag::Object | TypeTag::Str | TypeTag::Void => "memberobject_set",
                        TypeTag::F32 => "memberf32_set",
                        TypeTag::F64 => "memberf64_set",
                        TypeTag::Native => unreachable!("Native Type not ABI compatible"),
                    };

                    let member_set = if let Some(id) = module.get_name(fun_name) {
                        match id {
                            FuncOrDataId::Func(id) => id,
                            _ => unreachable!("cannot initialize array object from data id"),
                        }
                    } else {
                        let mut new_object = module.make_signature();
                        new_object.params.push(AbiParam::new(cranelift::codegen::ir::types::I64));
                        new_object.params.push(AbiParam::new(cranelift::codegen::ir::types::I64));
                        new_object.params.push(AbiParam::new(cranelift::codegen::ir::types::I64));
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
                            TypeTag::Native => unreachable!("Native Type not ABI compatible"),
                        };
                        new_object.params.push(AbiParam::new(ty));

                        let fn_id = module.declare_function(fun_name, Linkage::Import, &new_object).unwrap();
                        fn_id
                    };

                    let context_value = self.builder.use_var(self.context_var);
                    let (value, _, _) = self.pop();
                    let (this_value, _, _) = self.pop();
                    self.builder.declare_value_needs_stack_map(this_value);

                    let class_symbol = self.builder.ins().iconst(cranelift::codegen::ir::types::I64, i64::from_le_bytes(class_name.to_le_bytes()));
                    let parent_symbol = self.builder.ins().iconst(cranelift::codegen::ir::types::I64, i64::from_le_bytes(parent_symbol.to_le_bytes()));
                    let offset = self.builder.ins().iconst(cranelift::codegen::ir::types::I64, i64::from_le_bytes(offset.to_le_bytes()));

                    let member_set = module.declare_func_in_func(member_set, self.builder.func);
                    let _ = self.builder.ins().call(member_set, &[context_value, this_value, class_symbol, parent_symbol, offset, value]);

                    self.create_bail_block(module, None, &[]);
                }
                Bytecode::InvokeVirt(class_name, source_class, method_name) => {

                    let mut call_virt_func = module.make_signature();
                    call_virt_func.params.push(AbiParam::new(cranelift::codegen::ir::types::I64));
                    call_virt_func.params.push(AbiParam::new(cranelift::codegen::ir::types::I64));
                    call_virt_func.params.push(AbiParam::new(cranelift::codegen::ir::types::I64));
                    call_virt_func.params.push(AbiParam::new(cranelift::codegen::ir::types::I64));

                    let fn_id = module.declare_function("call_virtual_function", Linkage::Import, &call_virt_func).unwrap();

                    let call_virt_func_func = module.declare_func_in_func(fn_id, self.builder.func);
                    
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

                    let context_value = self.builder.use_var(self.context_var);

                    let _ = self.builder
                        .ins()
                        .call(call_virt_func_func, &[
                            context_value,
                            class_name_value,
                            source_class_value,
                            method_name_value,
                        ]);


                    let (sig, returns_object) = Runtime::get_virtual_method_signature(*class_name as Symbol, *method_name as Symbol);
                    if !sig.returns.is_empty() {

                        let name = match sig.returns[0].value_type {
                            types::I8 => "fetch_return_int8",
                            types::I16 => "fetch_return_int16",
                            types::I32 => "fetch_return_int32",
                            types::I64 if returns_object => "fetch_return_object",
                            types::I64 if !returns_object => "fetch_return_int64",
                            types::F32 => "fetch_return_float32",
                            types::F64 => "fetch_return_float64",
                            _ => unreachable!()
                        };

                        let fetch_return = if let Some(id) = module.get_name(name) {
                            match id {
                                FuncOrDataId::Func(id) => id,
                                _ => unreachable!("cannot create array object from data id"),
                            }
                        } else {
                            let mut fetch_return = module.make_signature();
                            fetch_return.params.push(AbiParam::new(cranelift::codegen::ir::types::I64));
                            fetch_return.returns.push(sig.returns[0]);

                            let fn_id = module.declare_function(name, Linkage::Import, &fetch_return).unwrap();
                            fn_id
                        };

                        let fetch_return = module.declare_func_in_func(fetch_return, self.builder.func);

                        let context_value = self.builder.use_var(self.context_var);

                        let results = self.builder.ins()
                            .call(fetch_return, &[context_value]);

                        let value = self.builder.inst_results(results)[0];
                        self.push(value , sig.returns[0].value_type, returns_object)
                    }

                    //let method_value = self.builder.inst_results(method_instructions)[0];
                    //self.create_bail_block(module, Some(types::I64), &[BlockArg::Value(method_value)]);
                }
                Bytecode::InvokeStatic(class_name, method_name) => {
                    let mut call_static_func = module.make_signature();
                    call_static_func.params.push(AbiParam::new(cranelift::codegen::ir::types::I64));
                    call_static_func.params.push(AbiParam::new(cranelift::codegen::ir::types::I64));
                    call_static_func.params.push(AbiParam::new(cranelift::codegen::ir::types::I64));

                    let fn_id = module.declare_function("call_static_function", Linkage::Import, &call_static_func).unwrap();

                    let call_static_func_func = module.declare_func_in_func(fn_id, self.builder.func);

                    let (sig, is_object) = Runtime::get_static_method_signature(*class_name as Symbol, *method_name as Symbol);

                    let class_name_value = self.builder
                        .ins()
                        .iconst(cranelift::codegen::ir::types::I64, i64::from(i64::from_le_bytes(class_name.to_le_bytes())));
                    
                    let method_name_value = self.builder
                        .ins()
                        .iconst(cranelift::codegen::ir::types::I64, i64::from(i64::from_le_bytes(method_name.to_le_bytes())));


                    let context_value = self.builder.use_var(self.context_var);
                    let _ = self.builder
                        .ins()
                        .call(call_static_func_func, &[
                            context_value,
                            class_name_value,
                            method_name_value,
                        ]);

                    let (sig, returns_object) = Runtime::get_static_method_signature(*class_name as Symbol, *method_name as Symbol);
                    if !sig.returns.is_empty() {

                        let name = match sig.returns[0].value_type {
                            types::I8 => "fetch_return_int8",
                            types::I16 => "fetch_return_int16",
                            types::I32 => "fetch_return_int32",
                            types::I64 if returns_object => "fetch_return_object",
                            types::I64 if !returns_object => "fetch_return_int64",
                            types::F32 => "fetch_return_float32",
                            types::F64 => "fetch_return_float64",
                            _ => unreachable!()
                        };

                        let fetch_return = if let Some(id) = module.get_name(name) {
                            match id {
                                FuncOrDataId::Func(id) => id,
                                _ => unreachable!("cannot create array object from data id"),
                            }
                        } else {
                            let mut fetch_return = module.make_signature();
                            fetch_return.params.push(AbiParam::new(cranelift::codegen::ir::types::I64));
                            fetch_return.returns.push(sig.returns[0]);

                            let fn_id = module.declare_function(name, Linkage::Import, &fetch_return).unwrap();
                            fn_id
                        };

                        let fetch_return = module.declare_func_in_func(fetch_return, self.builder.func);

                        let context_value = self.builder.use_var(self.context_var);

                        let results = self.builder.ins()
                            .call(fetch_return, &[context_value]);

                        let value = self.builder.inst_results(results)[0];
                        self.push(value , sig.returns[0].value_type, returns_object)
                    }
                    //let method_value = self.builder.inst_results(method_instructions)[0];
                    //self.create_bail_block(module, Some(types::I64), &[BlockArg::Value(method_value)]);
                }
                Bytecode::GetStaticMember(class_name, index, type_tag) => {
                    let mut is_object = false;
                    let fun_name = match type_tag {
                        TypeTag::U8 | TypeTag::I8 => "static_member8_get",
                        TypeTag::U16 | TypeTag::I16 => "static_member16_get",
                        TypeTag::U32 | TypeTag::I32 => "static_member32_get",
                        TypeTag::U64 | TypeTag::I64 => "static_member64_get",
                        TypeTag::Object | TypeTag::Str | TypeTag::Void => {
                            is_object = true;
                            "static_memberobject_get"
                        },
                        TypeTag::F32 => "static_memberf32_get",
                        TypeTag::F64 => "static_memberf64_get",
                        TypeTag::Native => unreachable!("Native Type not ABI compatible"),
                    };

                    let member_get = if let Some(id) = module.get_name(fun_name) {
                        match id {
                            FuncOrDataId::Func(id) => id,
                            _ => unreachable!("cannot initialize array object from data id"),
                        }
                    } else {
                        let mut new_object = module.make_signature();
                        new_object.params.push(AbiParam::new(cranelift::codegen::ir::types::I64));
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
                            TypeTag::Native => unreachable!("Native Type not ABI compatible"),
                        };
                        new_object.returns.push(AbiParam::new(ty));

                        let fn_id = module.declare_function(fun_name, Linkage::Import, &new_object).unwrap();
                        fn_id
                    };

                    let context_value = self.builder.use_var(self.context_var);

                    let class_symbol = self.builder.ins().iconst(cranelift::codegen::ir::types::I64, i64::from_le_bytes(class_name.to_le_bytes()));
                    let index = self.builder.ins().iconst(cranelift::codegen::ir::types::I64, i64::from_le_bytes(index.to_le_bytes()));

                    let member_get = module.declare_func_in_func(member_get, self.builder.func);
                    let member_get = self.builder.ins().call(member_get, &[context_value, class_symbol, index, ]);
                    let value = self.builder.inst_results(member_get)[0];
                    // TODO: add code for handling a missing member exception

                    let ty = match type_tag {
                        TypeTag::U8 | TypeTag::I8 => types::I8,
                        TypeTag::U16 | TypeTag::I16 => types::I16,
                        TypeTag::U32 | TypeTag::I32 => types::I32,
                        TypeTag::U64 | TypeTag::I64 => types::I64,
                        TypeTag::Object | TypeTag::Str | TypeTag::Void => types::I64,
                        TypeTag::F32 => types::F32,
                        TypeTag::F64 => types::F64,
                        TypeTag::Native => unreachable!("Native Type not ABI compatible"),
                    };

                    if is_object {
                        self.builder.declare_value_needs_stack_map(value);
                    }

                    self.create_bail_block(module, Some(ty), &[BlockArg::Value(value)]);

                    self.push(value, ty, is_object);
                }
                Bytecode::SetStaticMember(class_name, index, type_tag) => {
                    let fun_name = match type_tag {
                        TypeTag::U8 | TypeTag::I8 => "static_member8_set",
                        TypeTag::U16 | TypeTag::I16 => "static_member16_set",
                        TypeTag::U32 | TypeTag::I32 => "static_member32_set",
                        TypeTag::U64 | TypeTag::I64 => "static_member64_set",
                        TypeTag::Object | TypeTag::Str | TypeTag::Void => "static_memberobject_set",
                        TypeTag::F32 => "static_memberf32_set",
                        TypeTag::F64 => "static_memberf64_set",
                        TypeTag::Native => unreachable!("Native Type not ABI compatible"),
                    };

                    let member_set = if let Some(id) = module.get_name(fun_name) {
                        match id {
                            FuncOrDataId::Func(id) => id,
                            _ => unreachable!("cannot initialize array object from data id"),
                        }
                    } else {
                        let mut new_object = module.make_signature();
                        new_object.params.push(AbiParam::new(cranelift::codegen::ir::types::I64));
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
                            TypeTag::Native => unreachable!("Native Type not ABI compatible"),
                        };
                        new_object.params.push(AbiParam::new(ty));

                        let fn_id = module.declare_function(fun_name, Linkage::Import, &new_object).unwrap();
                        fn_id
                    };

                    let context_value = self.builder.use_var(self.context_var);
                    let (value, _, _) = self.pop();

                    let class_symbol = self.builder.ins().iconst(cranelift::codegen::ir::types::I64, i64::from_le_bytes(class_name.to_le_bytes()));
                    let index = self.builder.ins().iconst(cranelift::codegen::ir::types::I64, i64::from_le_bytes(index.to_le_bytes()));

                    let member_set = module.declare_func_in_func(member_set, self.builder.func);
                    let _ = self.builder.ins().call(member_set, &[context_value, class_symbol, index, value]);

                    self.create_bail_block(module, None, &[]);
                }
                Bytecode::GetStrRef(string_symbol) => {
                    let symbol_value = self.builder.ins().iconst(cranelift::codegen::ir::types::I64, i64::from_le_bytes(string_symbol.to_le_bytes()));

                    self.push(symbol_value, ir::types::I64, false);
                }
                Bytecode::Return => {
                    let normal_return_id = if let Some(id) = module.get_name("context_normal_return") {
                        match id {
                            FuncOrDataId::Func(id) => id,
                            _ => unreachable!("cannot create array object from data id"),
                        }
                    } else {
                        let mut new_object = module.make_signature();
                        new_object.params.push(AbiParam::new(cranelift::codegen::ir::types::I64));

                        let fn_id = module.declare_function("context_normal_return", Linkage::Import, &new_object).unwrap();
                        fn_id
                    };

                    let normal_return = module.declare_func_in_func(normal_return_id, self.builder.func);

                    let context_value = self.builder.use_var(self.context_var);

                    let should_unwind_result = self.builder.ins()
                        .call(normal_return, &[context_value]);

                    let _ = self.builder.inst_results(should_unwind_result);

                    let (return_value, _, _) = self.pop();
                    self.builder.ins().return_(&[return_value]);
                }
                Bytecode::ReturnVoid => {
                    let normal_return_id = if let Some(id) = module.get_name("context_normal_return") {
                        match id {
                            FuncOrDataId::Func(id) => id,
                            _ => unreachable!("cannot create array object from data id"),
                        }
                    } else {
                        let mut new_object = module.make_signature();
                        new_object.params.push(AbiParam::new(cranelift::codegen::ir::types::I64));

                        let fn_id = module.declare_function("context_normal_return", Linkage::Import, &new_object).unwrap();
                        fn_id
                    };

                    let normal_return = module.declare_func_in_func(normal_return_id, self.builder.func);

                    let context_value = self.builder.use_var(self.context_var);

                    let should_unwind_result = self.builder.ins()
                        .call(normal_return, &[context_value]);

                    let _ = self.builder.inst_results(should_unwind_result);
                    self.builder.ins().return_(&[]);
                }
                Bytecode::StartBlock(index) => {
                    let block= self.blocks[*index as usize];
                    let params = self.builder.block_params(block).to_vec();
                    self.restore_stack(*index as usize, &params);
                    self.builder.switch_to_block(block);
                    self.current_block = *index as usize;

                    let check_gc = if let Some(id) = module.get_name("check_and_do_garbage_collection") {
                        match id {
                            FuncOrDataId::Func(id) => id,
                            _ => unreachable!("cannot create array object from data id"),
                        }
                    } else {
                        let mut check_gc = module.make_signature();
                        check_gc.params.push(AbiParam::new(cranelift::codegen::ir::types::I64));

                        let fn_id = module.declare_function("check_and_do_garbage_collection", Linkage::Import, &check_gc).unwrap();
                        fn_id
                    };

                    let check_gc = module.declare_func_in_func(check_gc, self.builder.func);

                    let context_value = self.builder.use_var(self.context_var);

                    let _ = self.builder.ins()
                        .call(check_gc, &[context_value]);

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
                    self.builder.ins().jump(*block, &[]);
                }
                Bytecode::If(then_offset, else_offset) => {
                    let (value, _, _) = self.pop();
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
                        &[],
                        else_block,
                        &[],
                    );
                }
                x => todo!("remaining ops {:?}", x),
            }
        }
        Ok(())
    }

    fn create_bail_block(&mut self, module: &mut JITModule, return_type: Option<Type>, return_value: &[BlockArg]) {
        let should_unwind_id = if let Some(id) = module.get_name("context_should_unwind") {
            match id {
                FuncOrDataId::Func(id) => id,
                _ => unreachable!("cannot create array object from data id"),
            }
        } else {
            let mut new_object = module.make_signature();
            new_object.params.push(AbiParam::new(cranelift::codegen::ir::types::I64));
            new_object.returns.push(AbiParam::new(cranelift::codegen::ir::types::I8));

            let fn_id = module.declare_function("context_should_unwind", Linkage::Import, &new_object).unwrap();
            fn_id
        };

        let should_unwind = module.declare_func_in_func(should_unwind_id, self.builder.func);

        let context_value = self.builder.use_var(self.context_var);

        let should_unwind_result = self.builder.ins()
            .call(should_unwind, &[context_value]);

        let boolean = self.builder.inst_results(should_unwind_result)[0];
        let bail_block = self.builder.create_block();
        let new_block = self.builder.create_block();
        if let Some(ret_type) = return_type {
            self.builder.append_block_param(new_block, ret_type);
        }

        self.builder.ins()
            .brif(boolean, bail_block, &[], new_block, return_value);

        self.builder.switch_to_block(bail_block);

        let returns: &[Value] = match self.return_type {
            runtime::class::TypeTag::U8 | runtime::class::TypeTag::I8 => &[self.builder.ins().iconst(types::I8, 0)],
            runtime::class::TypeTag::U16 | runtime::class::TypeTag::I16 => &[self.builder.ins().iconst(types::I16, 0)],
            runtime::class::TypeTag::U32 | runtime::class::TypeTag::I32 => &[self.builder.ins().iconst(types::I32, 0)],
            runtime::class::TypeTag::U64 | runtime::class::TypeTag::I64 => &[self.builder.ins().iconst(types::I64, 0)],
            runtime::class::TypeTag::F32  => &[self.builder.ins().f32const(0.0)],
            runtime::class::TypeTag::F64 => &[self.builder.ins().f64const(0.0)],
            runtime::class::TypeTag::Object | runtime::class::TypeTag::Str => &[self.builder.ins().iconst(types::I64, 0)],
            runtime::class::TypeTag::Void => &[],
            runtime::class::TypeTag::Sized(_) => unreachable!("Native Members are not ABI Compatible"),
        };
        let ret_result = self.builder.ins().return_(returns);
        self.builder.inst_results(ret_result);
        self.builder.seal_block(bail_block);
        self.builder.switch_to_block(new_block);

        // TODO: add check to see if exception was caught and jump to the right block
    }
}
