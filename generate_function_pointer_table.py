import sys

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
                elif ty == u16:
                    output += f", Value::U16(x{i})"
                elif ty == u32:
                    output += f", Value::U32(x{i})"
                elif ty == u64:
                    output += f", Value::U64(x{i})"
                elif ty == i8:
                    output += f", Value::I8(x{i})"
                elif ty == i16:
                    output += f", Value::I16(x{i})"
                elif ty == i32:
                    output += f", Value::I32(x{i})"
                elif ty == i64:
                    output += f", Value::I64(x{i})"
                elif ty == f32:
                    output += f", Value::F32(x{i})"
                elif ty == f64:
                    output += f", Value::F64(x{i})"
            output += "], "
            if return_type == void: 
                output += "Type::Void) => {\n"
            elif return_type == u8:
                output += "Type::U8) => {\n"
            elif return_type == u16:
                output += "Type::U16) => {\n"
            elif return_type == u32:
                output += "Type::U32) => {\n"
            elif return_type == u64:
                output += "Type::64) => {\n"
            elif return_type == i8:
                output += "Type::I8) => {\n"
            elif return_type == i16:
                output += "Type::I16) => {\n"
            elif return_type == i32:
                output += "Type::I32) => {\n"
            elif return_type == i64:
                output += "Type::I64) => {\n"
            elif return_type == f32:
                output += "Type::F32) => {\n"
            elif return_type == f64:
                output += "Type::F64) => {\n"

            
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
        elif ty == u16:
            output += ', u16'
            call += f', x{i}'
        elif ty == u32:
            output += ', u32'
            call += f', x{i}'
        elif ty == u64:
            output += ', u64'
            call += f', x{i}'
        elif ty == i8:
            output += ', i8'
            call += f', x{i}'
        elif ty == i16:
            output += ', i16'
            call += f', x{i}'
        elif ty == i32:
            output += ', i32'
            call += f', x{i}'
        elif ty == i64:
            output += ', i64'
            call += f', x{i}'
        elif ty == f32:
            output += ', f32'
            call += f', x{i}'
        elif ty == f64:
            output += ', f64'
            call += f', x{i}'
    output += ') -> '
    call += ');\n'
    return_value = '            Value::'
    if return_type == void:
        output += "()"
        return_value += 'Unit(a)\n'
    elif return_type == u8:
        output += "u8"
        return_value += 'U8(a)\n'
    elif return_type == u16:
        output += "u16"
        return_value += 'U16(a)\n'
    elif return_type == u32:
        output += "u32"
        return_value += 'U32(a)\n'
    elif return_type == u64:
        output += "u64"
        return_value += 'U164(a)\n'
    elif return_type == i8:
        output += "i8"
        return_value += 'I8(a)\n'
    elif return_type == i16:
        output += "i16"
        return_value += 'I16(a)\n'
    elif return_type == i32:
        output += "i32"
        return_value += 'I32(a)\n'
    elif return_type == i64:
        output += "i64"
        return_value += 'I64(a)\n'
    elif return_type == f32:
        output += "f32"
        return_value += 'F32(a)\n'
    elif return_type == f64:
        output += "f64"
        return_value += 'F64(a)\n'
    
    output += '>(function_pointer) };\n'
    
    return output + call + return_value

def increment_types(types):
    global void, u8, u16, u32, u64, i8, i16, i32, i64, f32, f64
    index = 0
    carry = 1
    while index < len(types) and carry == 1:
        types[index] += carry
        if types[index] == f64 + 1:
            types[index] = u8
            carry = 1
        else:
            carry = 0
        index += 1
    if carry == 1:
        types.append(u8)

if __name__ == '__main__':
    with open(sys.argv[1], 'w') as f:
        generate_function(f)
