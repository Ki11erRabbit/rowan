

void = 0
u8 = 1
u16 = 2
u32 = 3
u64 = 4
i8 = 5
i16 = 6
i32 = 7
i64 = 8
f32 = 9
f64 = 10




def generate_function(f):
    global void, u8, u16, u32, u64, i8, i16, i32, i64, f32, f64
    f.write("pub fn cast_and_call(function_pointer: *const (), args: &[Value], return_type: Type) -> Value {\n")
    f.write("    match (args, return_type) {\n")
    types = []
    return_type = void
    while len(types) < 256:
        
        while return_type < f64 + 1:

            output = "        ([Value::U64(x)"

            for i, ty in enumerate(types):
                if ty == u8:
                    output += f", Value::U8(x{i})"
            output += "], "
            if return_type == void: 
                output += "Type:: Void) => {\n"
            
            output += generate_cast(types, return_type)
            output += "\n        }\n"
            f.write(output)
            return_type += 1
             
        increment_types(types)
        return_type = void

    f.write("}")

def generate_cast(types, return_type) -> str:
    global void, u8, u16, u32, u64, i8, i16, i32, i64, f32, f64
    call = '            let out = a(x'
    output = '            let a = unsafe { std::mem::transmute::<*const (), extern "C" fn(u64'
    for i, ty in enumerate(types):
        if ty == u8:
            output += ', u8'
            call += f', x{i}'
    output += ') -> '
    call += ');\n'
    return_value = '            Value::'
    if return_type == void:
        output += "()"
        return_value += 'Unit(a)\n'
    
    output += '> };\n'
    
    return output + call + return_value

def increment_types(types):
    global void, u8, u16, u32, u64, i8, i16, i32, i64, f32, f64
    index = 0
    carry = 1
    while index < len(types) and carry == 1:
        types[index] += carry
        if types[index] == f64:
            types[index] = u8
            carry = 1
        else:
            carry = 0

        
