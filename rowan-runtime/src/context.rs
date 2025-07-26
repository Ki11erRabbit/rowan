mod interpreter;

pub use interpreter::BytecodeContext;
use crate::context::interpreter::StackValue;

#[cfg(unix)]
#[cfg(target_arch = "x86_64")]
#[macro_export]
macro_rules! call_function_pointer {
    ($context:ident, $call_args:expr, $fn_ptr:ident, $return_type:expr, $return_value:expr) => {
        let mut integer_index = 1; // context takes the first slot
        let mut float_index = 0;
        let stack_byte_size = super::get_stack_byte_size($call_args);
        unsafe {
            std::arch::asm!(
                "push rsp",
                "add rsp, {offset}",
                "mov rdi, {ctx}",
                ctx = in(reg) $context,
                offset = in(reg) stack_byte_size,
            );
        }
        for arg in $call_args {
            match arg {
                StackValue::Blank => break,
                StackValue::Int8(value) => {
                    place_value_in_int_reg!(value, integer_index, u8);
                    integer_index += 1;
                }
                StackValue::Int16(value) => {
                    place_value_in_int_reg!(value, integer_index);
                    integer_index += 1;
                }
                StackValue::Int32(value) => {
                    place_value_in_int_reg!(value, integer_index);
                    integer_index += 1;
                }
                StackValue::Int64(value) => {
                    place_value_in_int_reg!(value, integer_index);
                    integer_index += 1;
                }
                StackValue::Reference(value) => {
                    place_value_in_int_reg!(value, integer_index);
                    integer_index += 1;
                }
                StackValue::Float32(value) => {
                    place_value_in_float_reg!(value, float_index, f32);
                    float_index += 1;
                }
                StackValue::Float64(value) => {
                    place_value_in_float_reg!(value, float_index, f64);
                    float_index += 1;
                }
            }
        }
        unsafe {
            std::arch::asm!(
                "call {ptr}",
                "pop rsp",
                ptr = in(reg) $context,
            );
        }
        match $return_type {
            runtime::class::TypeTag::U8 | runtime::class::TypeTag::I8 => {
                let mut output: u8 = 0;
                unsafe {
                    std::arch::asm!(
                        "nop",
                        out("al") output,
                    );
                }
                $return_value = StackValue::Int8(output);
            }
            runtime::class::TypeTag::U16 | runtime::class::TypeTag::I16 => {
                let mut output: u16 = 0;
                unsafe {
                    std::arch::asm!(
                        "nop",
                        out("ax") output,
                    );
                }
                $return_value = StackValue::Int16(output);
            }
            runtime::class::TypeTag::U32 | runtime::class::TypeTag::I32 => {
                let mut output: u32 = 0;
                unsafe {
                    std::arch::asm!(
                        "nop",
                        out("eax") output,
                    );
                }
                $return_value = StackValue::Int32(output);
            }
            runtime::class::TypeTag::U64 | runtime::class::TypeTag::I64 => {
                let mut output: u64 = 0;
                unsafe {
                    std::arch::asm!(
                        "nop",
                        out("rax") output,
                    );
                }
                $return_value = StackValue::Int64(output);
            }
            runtime::class::TypeTag::F32=> {
                let mut output: f32 = 0.0;
                unsafe {
                    std::arch::asm!(
                        "movss [{out}], xmm0",
                        out = out(reg) output,
                    );
                }
                $return_value = StackValue::Float32(output);
            }
            runtime::class::TypeTag::F64 => {
                let mut output: f64 = 0.0;
                unsafe {
                    std::arch::asm!(
                        "movsd [{out}], xmm0",
                        out = out(reg) output,
                    );
                }
                $return_value = StackValue::Float64(output);
            }
            _ => unreachable!("invalid return type"),
        }
    };
}

#[cfg(target_arch = "x86_64")]
#[macro_export]
macro_rules! place_value_in_int_reg {
    ($val:ident, $reg_index:ident) => {
        match $reg_index {
            0 => unreachable!("We entered a state that should not have happened"),
            1 => unsafe {
                std::arch::asm!(
                    "mov rsi, {val}",
                    val = in(reg) *$val,
                );
            }
            2 => unsafe {
                std::arch::asm!(
                    "mov rdx, {val}",
                    val = in(reg) *$val,
                );
            }
            3 => unsafe {
                std::arch::asm!(
                    "mov rcx, {val}",
                    val = in(reg) *$val,
                );
            }
            4 => unsafe {
                std::arch::asm!(
                    "mov r8, {val}",
                    val = in(reg) *$val,
                );
            }
            5 => unsafe {
                std::arch::asm!(
                    "mov r9, {val}",
                    val = in(reg) *$val,
                );
            }
            _ => unsafe {
                std::arch::asm!(
                    "push {val}",
                    val = in(reg) *$val,
                );
            }
        }
    };
    ($val:ident, $reg_index:ident, u8) => {
        match $reg_index {
            0 => unreachable!("We entered a state that should not have happened"),
            1 => unsafe {
                std::arch::asm!(
                    "mov al, 0",
                    in("sil") *$val,
                );
            }
            2 => unsafe {
                std::arch::asm!(
                    "mov dl, 0",
                    in("dl") *$val,
                );
            }
            3 => unsafe {
                std::arch::asm!(
                    "mov cl, 0",
                    in("cl") *$val,
                );
            }
            4 => unsafe {
                std::arch::asm!(
                    "mov r8b, 0",
                    in("r8b") *$val,
                );
            }
            5 => unsafe {
                std::arch::asm!(
                    "mov r9b, 0",
                    in("r9b") *$val,
                );
            }
            _ => unsafe {
                std::arch::asm!(
                    "push rax",
                    in("al") *$val,
                );
            }
        }
    };
}

#[cfg(target_arch = "x86_64")]
#[macro_export]
macro_rules! place_value_in_float_reg {
    ($val:ident, $reg_index:ident, f32) => {
        match $reg_index {
            0 => unsafe {
                std::arch::asm!(
                    "movss xmm0, [{val}]",
                    val = in(reg) *$val,
                );
            }
            1 => unsafe {
                std::arch::asm!(
                    "movss xmm1, [{val}]",
                    val = in(reg) *$val,
                );
            }
            2 => unsafe {
                std::arch::asm!(
                    "movss xmm2, [{val}]",
                    val = in(reg) *$val,
                );
            }
            3 => unsafe {
                std::arch::asm!(
                    "movss xmm3, [{val}]",
                    val = in(reg) *$val,
                );
            }
            4 => unsafe {
                std::arch::asm!(
                    "movss xmm4, [{val}]",
                    val = in(reg) *$val,
                );
            }
            5 => unsafe {
                std::arch::asm!(
                    "movss xmm5, [{val}]",
                    val = in(reg) *$val,
                );
            }
            6 => unsafe {
                std::arch::asm!(
                    "movss xmm6, [{val}]",
                    val = in(reg) *$val,
                );
            }
            7 => unsafe {
                std::arch::asm!(
                    "movss xmm7, [{val}]",
                    val = in(reg) *$val,
                );
            }
            _ => unsafe {
                std::arch::asm!(
                    "push {val}",
                    val = in(reg) *$val,
                );
            }
        }
    };
    ($val:ident, $reg_index:ident, f64) => {
        match $reg_index {
            0 => unsafe {
                std::arch::asm!(
                    "movsd xmm0, [{val}]",
                    val = in(reg) *$val,
                );
            }
            1 => unsafe {
                std::arch::asm!(
                    "movsd xmm1, [{val}]",
                    val = in(reg) *$val,
                );
            }
            2 => unsafe {
                std::arch::asm!(
                    "movsd xmm2, [{val}]",
                    val = in(reg) *$val,
                );
            }
            3 => unsafe {
                std::arch::asm!(
                    "movsd xmm3, [{val}]",
                    val = in(reg) *$val,
                );
            }
            4 => unsafe {
                std::arch::asm!(
                    "movsd xmm4, [{val}]",
                    val = in(reg) *$val,
                );
            }
            5 => unsafe {
                std::arch::asm!(
                    "movsd xmm5, [{val}]",
                    val = in(reg) *$val,
                );
            }
            6 => unsafe {
                std::arch::asm!(
                    "movsd xmm6, [{val}]",
                    val = in(reg) *$val,
                );
            }
            7 => unsafe {
                std::arch::asm!(
                    "movsd xmm7, [{val}]",
                    val = in(reg) *$val,
                );
            }
            _ => unsafe {
                std::arch::asm!(
                    "push {val}",
                    val = in(reg) *$val,
                );
            }
        }
    };
}

#[cfg(unix)]
#[cfg(target_arch = "x86_64")]
fn get_stack_byte_size(call_args: &[StackValue]) -> usize {
    const INT_REGISTER_COUNT: usize = 4; // 4 because context is always the first parameter so we lose a register
    let mut int_arg_index = 0;
    const FLOAT_REGISTER_COUNT: usize = 8;
    let mut float_arg_index = 0;
    let mut stack_size = 0;

    for arg in call_args {
        match arg {
            StackValue::Blank => break,
            StackValue::Int8(_) | StackValue::Int16(_) |
            StackValue::Int32(_) | StackValue::Int64(_) |
            StackValue::Reference(_) => {
                if int_arg_index > INT_REGISTER_COUNT {
                    stack_size += std::mem::size_of::<usize>();
                }
                int_arg_index += 1;
            }
            StackValue::Float32(_) | StackValue::Float64(_) => {
                if float_arg_index > FLOAT_REGISTER_COUNT {
                    stack_size += std::mem::size_of::<usize>();
                }
                float_arg_index += 1;
            }
        }
    }
    stack_size
}