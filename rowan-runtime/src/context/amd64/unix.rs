use crate::context::{BytecodeContext, Value};
use crate::runtime::class::TypeTag;


pub fn call_function_pointer(
    context: *mut BytecodeContext,
    call_args: *const Value,
    call_args_len: usize,
    fn_ptr: *const (),
    return_type: u8,
    padding_byte: u8,
) -> Value {
    //println!("values: {context:p}, {call_args:p}, {call_args_len}, {fn_ptr:p}, {return_type}");
    unsafe {
        std::arch::asm!(
        "",
        in("r11") context,
        in("r15") call_args,
        in("r14") call_args_len,
        in("r13") fn_ptr,
        in("r12b") return_type,
        in("al") padding_byte,
        );
    }

    /*let stack_byte_size = get_stack_byte_padding_size(unsafe {
        std::slice::from_raw_parts(call_args, call_args_len)
    });*/

    /*
    Here is what the assembly looked like before it was converted to use numeric labels
    unsafe {
        std::arch::asm!(
                "jmp dispatch",
            "handlers:",
                ".quad integer",
                ".quad float",
            "load_int_handlers:",
                ".quad first_int",
                ".quad second_int",
                ".quad third_int",
                ".quad fourth_int",
                ".quad fifth_int",
            "load_float_handlers:",
                ".quad first_float",
                ".quad second_float",
                ".quad third_float",
                ".quad fourth_float",
                ".quad fifth_float",
                ".quad sixth_float",
                ".quad seventh_float",
                ".quad eighth_float",
            "dispatch:",
                "push rsp", // backing up rsp
                "push r13", // storing fn_ptr
                "push r12", // storing return_type
                "test rax, rax",
                "mov rax, rsp", // putting rsp into rax so that we can access it later
                "jne body_label",
                "sub rsp, 8", // Extending the stack if we have an odd number of arguments on the stack
            "body_label:",
                "mov rdi, r11", // putting context into first call register
                "xor r11, r11", // Clear out r11 to be used as index offset
                "xor r12, r12", // Clear out r12 to be used as float index
                "xor r13, r13", // Clear out r13 to be used for int index
            "start_of_for_loop:",
                "cmp r11, r14", // checking if index is less than the length
                "je call_label",
                "mov r10, [r15+r11*16]", // load value tag into r10
                "cmp r10, 4",
                "jbe integer", // jump if we are less than or equal to the reference tag
                "jmp float",   // otherwise jump to the float handler
            "body_of_for_loop:",
                "integer:",
                    "mov r10, [r15+r11*16+8]", // fetch data and put it in r10
                    "cmp r13, 5", // Checking if int index is less than 5 (we have already used rdi)
                    "jl int_reg",
                    "push r10", // putting arguments on the stack, although, right now they are in the wrong order
                    "jmp end_of_for_loop",
                "int_reg:",
                    "jmp qword ptr [load_int_handlers+r13*8]",
                "float:",
                    "mov r10, [r15+r11*16+8]", // fetch data and put it in r10
                    "cmp r12, 8", // Checking if float index is less than 8
                    "jl float_reg",
                    "inc r12",
                    "push r10", // putting arguments on the stack, although, right now they are in the wrong order
                    "jmp end_of_for_loop",
                "float_reg:",
                    "jmp qword ptr [load_float_handlers+r13*8]",
            "end_of_for_loop:",
                "inc r14",
                "jmp start_of_for_loop",
            "load_int_registers:",
                "first_int:",
                    "inc r13",
                    "mov rsi, r10",
                    "jmp end_of_for_loop",
                "second_int:",
                    "inc r13",
                    "mov rdx, r10",
                    "jmp end_of_for_loop",
                "third_int:",
                    "inc r13",
                    "mov rcx, r10",
                    "jmp end_of_for_loop",
                "fourth_int:",
                    "inc r13",
                    "mov r8, r10",
                    "jmp end_of_for_loop",
                "fifth_int:",
                    "inc r13",
                    "mov r9, r10",
                    "jmp end_of_for_loop",
            "load_float_registers:",
                "first_float:",
                    "inc r12",
                    "movq xmm0, r10",
                    "jmp end_of_for_loop",
                "second_float:",
                    "inc r12",
                    "movq xmm1, r10",
                    "jmp end_of_for_loop",
                "third_float:",
                    "inc r12",
                    "movq xmm2, r10",
                    "jmp end_of_for_loop",
                "fourth_float:",
                    "inc r12",
                    "movq xmm3, r10",
                    "jmp end_of_for_loop",
                "fifth_float:",
                    "inc r12",
                    "movq xmm4, r10",
                    "jmp end_of_for_loop",
                "sixth_float:",
                    "inc r12",
                    "movq xmm5, r10",
                    "jmp end_of_for_loop",
                "seventh_float:",
                    "inc r12",
                    "movq xmm6, r10",
                    "jmp end_of_for_loop",
                "eighth_float:",
                    "inc r12",
                    "movq xmm7, r10",
                    "jmp end_of_for_loop",
            "call_label:",
                "mov r12, [rax]", // getting return type
                "mov r13, [rax+8]", // getting fn_ptr
                "mov r15, rax",   // putting older  stack pointer in r15 for quick reloading
                "call r13", // calling function pointer
                "mov rsp, [r15+16]", // restoring original stack pointer and handing control back to rust.
            options(nostack)
        );
    }


     */

    unsafe {
        std::arch::asm!(
            "jmp 2f",
        "40:",
            ".quad 41f",
            ".quad 42f",
        "50:",
            ".quad 51f",
            ".quad 52f",
            ".quad 53f",
            ".quad 54f",
            ".quad 55f",
        "60:",
            ".quad 61f",
            ".quad 62f",
            ".quad 63f",
            ".quad 64f",
            ".quad 65f",
            ".quad 66f",
            ".quad 67f",
            ".quad 68f",
        "2:",
            "push rsp", // backing up rsp
            "push r13", // storing fn_ptr
            "push r12", // storing return_type
            "test rax, rax",
            "mov rax, rsp", // putting rsp into rax so that we can access it later
            "jne 3f",
            "sub rsp, 8", // Extending the stack if we have an odd number of arguments on the stack
        "3:",
            "mov rdi, r11", // putting context into first call register
            "xor r11, r11", // Clear out r11 to be used as index offset
            "xor r12, r12", // Clear out r12 to be used as float index
            "xor r13, r13", // Clear out r13 to be used for int index
        "4:",
            "cmp r11, r14", // checking if index is less than the length
            "je 6f",
            "push rax", // Backing up rax
            "mov rax, 16",
            "push rdx", // Backing up rdx since mul will overwrite it
            "mul r11",
            "pop rdx", // Restoring rdx after mul
            "mov r10, [r15+rax]", // load value tag into r10
            "pop rax",
            "cmp r10, 4",
            "jbe 41f", // jump if we are less than or equal to the reference tag
            "jmp 42f",   // otherwise jump to the float handler
        //"body_of_for_loop:",
            "41:",
                "push rax", // Back up rax
                "mov rax, 16",
                "push rdx", // Backing up rdx since mul will overwrite it
                "mul r11",
                "pop rdx", // Restoring rdx after mul
                "mov r10, [r15+rax+8]", // fetch data and put it in r10
                "pop rax",
                "cmp r13, 5", // Checking if int index is less than 5 (we have already used rdi)
                "jl 31f",
                "push r10", // putting arguments on the stack, although, right now they are in the wrong order
                "jmp 5f",
            "31:",
                "push rax",
                "lea rax, [rip+50b]",
                "jmp qword ptr [rax+r13*8]",
            "42:",
                "push rax", // Back up rax
                "mov rax, 16",
                "push rdx", // Backing up rdx since mul will overwrite it
                "mul r11",
                "pop rdx", // Restoring rdx after mul
                "mov r10, [r15+rax+8]", // fetch data and put it in r10
                "pop rax",
                "cmp r12, 8", // Checking if float index is greater than 8
                "jl 32f",
                "inc r12",
                "push r10", // putting arguments on the stack, although, right now they are in the wrong order
                "jmp 5f",
            "32:",
                "push rax",
                "lea rax, [rip+50b]",
                "jmp qword ptr [rax+r13*8]",
        "5:",
            "inc r11",
            "jmp 4b",
        //"load_int_registers:",
            "51:",
                "pop rax",
                "inc r13",
                "mov rsi, r10",
                "jmp 5b",
            "52:",
                "pop rax",
                "inc r13",
                "mov rdx, r10",
                "jmp 5b",
            "53:",
                "pop rax",
                "inc r13",
                "mov rcx, r10",
                "jmp 5b",
            "54:",
                "pop rax",
                "inc r13",
                "mov r8, r10",
                "jmp 5b",
            "55:",
                "pop rax",
                "inc r13",
                "mov r9, r10",
                "jmp 5b",
        //"load_float_registers:",
            "61:",
                "pop rax",
                "inc r12",
                "movq xmm0, r10",
                "jmp 5b",
            "62:",
                "pop rax",
                "inc r12",
                "movq xmm1, r10",
                "jmp 5b",
            "63:",
                "pop rax",
                "inc r12",
                "movq xmm2, r10",
                "jmp 5b",
            "64:",
                "pop rax",
                "inc r12",
                "movq xmm3, r10",
                "jmp 5b",
            "65:",
                "pop rax",
                "inc r12",
                "movq xmm4, r10",
                "jmp 5b",
            "66:",
                "pop rax",
                "inc r12",
                "movq xmm5, r10",
                "jmp 5b",
            "67:",
                "pop rax",
                "inc r12",
                "movq xmm6, r10",
                "jmp 5b",
            "68:",
                "pop rax",
                "inc r12",
                "movq xmm7, r10",
                "jmp 5b",
        "6:",
            "mov r12, [rax]", // getting return type
            "mov r13, [rax+8]", // getting fn_ptr
            "mov r15, rax",   // putting older  stack pointer in r15 for quick reloading
            "call r13", // calling function pointer
            "mov rsp, [r15+16]", // restoring original stack pointer and handing control back to rust.
        options(nostack)
        );
    }
    let mut int_return: u64 = 0;
    let mut float_return: f64 = 0.0;
    let mut return_type: u8 = 0;
    unsafe {
        std::arch::asm!(
        "",
        // Capture return values in explicit registers
        out("rax") int_return,
        out("xmm0") float_return,
        out("r12b") return_type,
        );
    }

    let return_type = TypeTag::from_tag(return_type);
    match return_type {
        TypeTag::U8 | TypeTag::I8 => Value::from(int_return as u8),
        TypeTag::U16 | TypeTag::I16 => Value::from(int_return as u16),
        TypeTag::U32 | TypeTag::I32 => Value::from(int_return as u32),
        TypeTag::U64 | TypeTag::I64 => Value::from(int_return),
        TypeTag::F32 => Value::from(float_return as f32),
        TypeTag::F64 => Value::from(float_return),
        TypeTag::Void => Value::blank(),
        _ => unreachable!("invalid return type"),
    }
}


pub fn need_padding(call_args: &[Value]) -> bool {
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
            Value { tag: 4, ..} => {
                if int_arg_index > INT_REGISTER_COUNT {
                    stack_size += std::mem::size_of::<usize>();
                }
                int_arg_index += 1;
            }
            Value { tag: 5, ..} | Value { tag: 6, ..} => {
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
    //println!("padding: {output}");
    output != 0
}