use cranelift::{codegen::Context, prelude::FunctionBuilderContext};
use cranelift_jit::JITModule;




pub struct Jit {
    builder_context: FunctionBuilderContext,
    context: Context,
    module: JITModule,
}
