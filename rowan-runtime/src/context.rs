mod interpreter;

use cranelift_codegen::gimli::write::Writer;
pub use interpreter::BytecodeContext;
use crate::context::interpreter::StackValue;
use crate::runtime::class::TypeTag;
use crate::runtime::Reference;

#[cfg(target_arch = "x86_64")]
macro_rules! place_value_in_int_reg {
    ($val:ident, $reg_index:ident) => {
        match $reg_index {
            0 => unreachable!("We entered a state that should not have happened"),
            1 => unsafe {
                std::arch::asm!(
                    "nop",
                    in("rsi") $val,
                );
            }
            2 => unsafe {
                std::arch::asm!(
                    "nop",
                    in("rdx") $val,
                );
            }
            3 => unsafe {
                std::arch::asm!(
                    "nop",
                    in("rcx") $val,
                );
            }
            4 => unsafe {
                std::arch::asm!(
                    "nop",
                    in("r8") $val,
                );
            }
            5 => unsafe {
                std::arch::asm!(
                    "nop",
                    in("r9") $val,
                );
            }
            _ => unsafe {
                std::arch::asm!(
                    "push {val}",
                    val = in(reg) $val,
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
                    in("sil") $val,
                );
            }
            2 => unsafe {
                std::arch::asm!(
                    "mov dl, 0",
                    in("dl") $val,
                );
            }
            3 => unsafe {
                std::arch::asm!(
                    "mov cl, 0",
                    in("cl") $val,
                );
            }
            4 => unsafe {
                std::arch::asm!(
                    "mov r8b, 0",
                    in("r8b") $val,
                );
            }
            5 => unsafe {
                std::arch::asm!(
                    "mov r9b, 0",
                    in("r9b") $val,
                );
            }
            _ => unsafe {
                std::arch::asm!(
                    "push rax",
                    in("al") $val,
                );
            }
        }
    };
}

#[cfg(target_arch = "x86_64")]
macro_rules! place_value_in_float_reg {
    ($val:ident, $reg_index:ident, f32) => {
        match $reg_index {
            0 => unsafe {
                std::arch::asm!(
                    "movss xmm0, [{val}]",
                    val = in(reg) $val,
                );
            }
            1 => unsafe {
                std::arch::asm!(
                    "movss xmm1, [{val}]",
                    val = in(reg) $val,
                );
            }
            2 => unsafe {
                std::arch::asm!(
                    "movss xmm2, [{val}]",
                    val = in(reg) $val,
                );
            }
            3 => unsafe {
                std::arch::asm!(
                    "movss xmm3, [{val}]",
                    val = in(reg) $val,
                );
            }
            4 => unsafe {
                std::arch::asm!(
                    "movss xmm4, [{val}]",
                    val = in(reg) $val,
                );
            }
            5 => unsafe {
                std::arch::asm!(
                    "movss xmm5, [{val}]",
                    val = in(reg) $val,
                );
            }
            6 => unsafe {
                std::arch::asm!(
                    "movss xmm6, [{val}]",
                    val = in(reg) $val,
                );
            }
            7 => unsafe {
                std::arch::asm!(
                    "movss xmm7, [{val}]",
                    val = in(reg) $val,
                );
            }
            _ => unsafe {
                std::arch::asm!(
                    "push {val}",
                    val = in(reg) $val,
                );
            }
        }
    };
    ($val:ident, $reg_index:ident, f64) => {
        match $reg_index {
            0 => unsafe {
                std::arch::asm!(
                    "movsd xmm0, [{val}]",
                    val = in(reg) $val,
                );
            }
            1 => unsafe {
                std::arch::asm!(
                    "movsd xmm1, [{val}]",
                    val = in(reg) $val,
                );
            }
            2 => unsafe {
                std::arch::asm!(
                    "movsd xmm2, [{val}]",
                    val = in(reg) $val,
                );
            }
            3 => unsafe {
                std::arch::asm!(
                    "movsd xmm3, [{val}]",
                    val = in(reg) $val,
                );
            }
            4 => unsafe {
                std::arch::asm!(
                    "movsd xmm4, [{val}]",
                    val = in(reg) $val,
                );
            }
            5 => unsafe {
                std::arch::asm!(
                    "movsd xmm5, [{val}]",
                    val = in(reg) $val,
                );
            }
            6 => unsafe {
                std::arch::asm!(
                    "movsd xmm6, [{val}]",
                    val = in(reg) $val,
                );
            }
            7 => unsafe {
                std::arch::asm!(
                    "movsd xmm7, [{val}]",
                    val = in(reg) $val,
                );
            }
            _ => unsafe {
                std::arch::asm!(
                    "push {val}",
                    val = in(reg) $val,
                );
            }
        }
    };
}

pub struct Value {
    tag: u64,
    value: ValueUnion,
}

impl Value {
    pub fn new(tag: u64, value: ValueUnion) -> Self {
        Self { tag, value }
    }

    pub fn from_stack_value(value: StackValue) -> Self {
        match value {
            StackValue::Int8(v) => {
                Value::new(0, ValueUnion { c: v })
            }
            StackValue::Int16(v) => {
                Value::new(1, ValueUnion { s: v })
            }
            StackValue::Int32(v) => {
                Value::new(2, ValueUnion { i: v })
            }
            StackValue::Int64(v) => {
                Value::new(3, ValueUnion { l: v })
            }
            StackValue::Float32(v) => {
                Value::new(4, ValueUnion{ f: v })
            }
            StackValue::Float64(v) => {
                Value::new(5, ValueUnion { d: v })
            }
            StackValue::Reference(v) => {
                Value::new(6, ValueUnion { r: v })
            }
            StackValue::Blank => {
                Value::new(7, ValueUnion { s: 0 })
            }
        }
    }
}

pub union ValueUnion {
    c: u8,
    s: u16,
    i: u32,
    l: u64,
    f: f32,
    d: f64,
    r: Reference,
}

#[cfg(unix)]
#[cfg(target_arch = "x86_64")]
pub extern "C" fn call_function_pointer(
    context: *mut BytecodeContext,
    call_args: *const Value,
    call_args_len: usize,
    fn_ptr: *const (),
    return_type: u8,
) -> StackValue {
    //println!("values: {context:p}, {call_args:p}, {call_args_len}, {fn_ptr:p}, {return_type}");
    unsafe {
        std::arch::asm!(
            "",
            in("r11") context,
            in("r15") call_args,
            in("r14") call_args_len,
            in("r13") fn_ptr,
            in("r12b") return_type,
        )
    }

    let stack_byte_size = get_stack_byte_padding_size(unsafe {
        std::slice::from_raw_parts(call_args, call_args_len)
    });

    unsafe {
        std::arch::asm!(
                "jmp dispatch",
            "handlers:",
                ".quad u8",
                ".quad u16",
                ".quad u32",
                ".quad u64",
                ".quad f32",
                ".quad f64",
                ".quad ref",
            "load_int_handler:",
                ".quad first_int",
                ".quad second_int",
                ".quad third_int",
                ".quad fourth_int",
                ".quad fifth_int",
            "dispatch:",
                "push rsp", // backing up rsp
                "push r13", // storing fn_ptr
                "push r12", // storing return_type
                "test rax, rax",
                "jne body_label",
                "sub rsp, 8", // Extending the stack if we have an odd number of arguments on the stack
            "body_label:",
                "mov rdi, r11", // putting context into first call register
                "xor r11, r11", // Clear out r11 to be used as byte offset
                "xor r12, r12", // Clear out r12 to be used as float index
                "xor r13, r13", // Clear out r13 to be used for int index
            "start_of_for_loop:",
                "cmp r11, r14",
                "je call_label",
                "mov r10, [r15+r11]", // load value tag into r10
                "jmp qword ptr [handlers+r10*8]", // use jump table to handle each arg type
            "body_of_for_loop:",
                "u8:",
                    "mov r10, [r15+r11+8]", // fetch data and put it in r10
                    "cmp r13, 5", // Checking if int index is less than 5 (we have already used rdi)
                    "jl u8_reg",
                    "push r10",
                    "inc r14",
                    "jmp start_of_for_loop",
                "u8_reg:",
                    ""

            "end_of_for_loop:",
                "add rsp, 16",
                "jmp start_of_for_loop",
            "load_int_registers:",
                ""
            "call_label:",
        )
    }

    let mut integer_index = 1; // context takes the first slot
    let mut float_index = 0;
    let mut saved_rsp: *const () = std::ptr::null();
    unsafe {
        std::arch::asm!(
            "mov {}, rsp",
            "mov rdi, {}",
            out(reg) saved_rsp,
            out(reg) context,
        );
    }
    let mut i = 0;
    loop {
        let arg = unsafe {
            call_args.add(i).read()
        };
        match arg {
            Value { tag: 7, ..} => break,
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
        i += 1;
        if i >= call_args_len {
            break;
        }
    }
    let mut int_return: u64 = 0;
    let mut float_return: f64 = 0.0;
    unsafe {
        std::arch::asm!(
            "call {ptr}",
            "mov rsp, {saved_rsp}",
            ptr = in(reg) saved_rsp,
            saved_rsp = out(reg) saved_rsp,
            // Capture return values in explicit registers
            out("rax") int_return,
            out("xmm0") float_return,

            // Clobber other caller-saved registers
            out("rcx") _,
            out("rdx") _,
            out("rsi") _,
            out("rdi") _,
            out("r8") _,
            out("r9") _,
            out("r10") _,
            out("r11") _,
            out("xmm1") _,
            out("xmm2") _,
            out("xmm3") _,
            out("xmm4") _,
            out("xmm5") _,
            out("xmm6") _,
            out("xmm7") _,
        );
    }

    let return_type = TypeTag::from_tag(return_type);
    match return_type {
        TypeTag::U8 | TypeTag::I8 => StackValue::Int8(int_return as u8),
        TypeTag::U16 | TypeTag::I16 => StackValue::Int16(int_return as u16),
        TypeTag::U32 | TypeTag::I32 => StackValue::Int32(int_return as u32),
        TypeTag::U64 | TypeTag::I64 => StackValue::Int64(int_return),
        TypeTag::F32 => StackValue::Float32(float_return as f32),
        TypeTag::F64 => StackValue::Float64(float_return),
        TypeTag::Void => StackValue::Blank,
        _ => unreachable!("invalid return type"),
    }
}

#[cfg(unix)]
#[cfg(target_arch = "x86_64")]
fn get_stack_byte_padding_size(call_args: &[Value]) -> usize {
    const INT_REGISTER_COUNT: usize = 5; // 5 because context is always the first parameter so we lose a register
    let mut int_arg_index = 0;
    const FLOAT_REGISTER_COUNT: usize = 8;
    let mut float_arg_index = 0;
    let mut stack_size = 0;

    for arg in call_args {
        match arg {
            Value { tag: 7, ..}  => break,
            Value { tag: 0, ..} | Value { tag: 1, ..} |
            Value { tag: 2, ..} | Value { tag: 3, ..} |
            Value { tag: 6, ..} => {
                if int_arg_index > INT_REGISTER_COUNT {
                    stack_size += std::mem::size_of::<usize>();
                }
                int_arg_index += 1;
            }
            Value { tag: 4, ..} | Value { tag: 5, ..} => {
                if float_arg_index > FLOAT_REGISTER_COUNT {
                    stack_size += std::mem::size_of::<usize>();
                }
                float_arg_index += 1;
            }
            _ => unreachable!("invalid arg type"),
        }
    }
    let mut output = 0;
    while stack_size % 16 != 0 {
        stack_size += std::mem::size_of::<usize>();
        output += std::mem::size_of::<usize>();
    }
    println!("padding: {output}");
    output
}