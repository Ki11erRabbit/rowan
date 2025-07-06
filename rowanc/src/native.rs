use rowan_shared::TypeTag;

pub struct NativeAttributes {
    pub native_member_sizes: Vec<String>,
    pub native_functions: Vec<(String, Vec<TypeTag>, TypeTag)>,
}

impl NativeAttributes {
    pub fn new(native_member_sizes: Vec<String>, native_functions: Vec<(String, Vec<TypeTag>, TypeTag)>) -> Self {
        Self {
            native_functions,
            native_member_sizes,
        }
    }
    
    pub fn as_c_header(&self) -> String {
        let mut output = String::from("#include <stdint.h>\n");
        
        for member_size in self.native_member_sizes.iter() {
            let name = member_size.replace("::", "__")
                .replace("-", "_dash_");
            output.push_str(&format!("size_t {name}();\n"));
        }
        
        for (name, args, return_type) in self.native_functions.iter() {
            let name = name.replace("::", "__")
                .replace("-", "_dash_");
            
            match return_type {
                TypeTag::I8 => output.push_str("int_8"),
                TypeTag::U8 => output.push_str("uint_8"),
                TypeTag::I16 => output.push_str("int_16"),
                TypeTag::U16 => output.push_str("uint_16"),
                TypeTag::I32 => output.push_str("int_32"),
                TypeTag::U32 => output.push_str("uint_32"),
                TypeTag::I64 => output.push_str("int_64"),
                TypeTag::U64 => output.push_str("uint_64"),
                TypeTag::F32 => output.push_str("float"),
                TypeTag::F64 => output.push_str("double"),
                TypeTag::Object | TypeTag::Str => output.push_str("uint_64"),
                TypeTag::Void => output.push_str("void"),
                _ => unreachable!("return type can't be native")
            }

            output.push_str(&format!(" {name}("));

            for (i, arg) in args.iter().enumerate() {
                match arg {
                    TypeTag::I8 => output.push_str("int_8"),
                    TypeTag::U8 => output.push_str("uint_8"),
                    TypeTag::I16 => output.push_str("int_16"),
                    TypeTag::U16 => output.push_str("uint_16"),
                    TypeTag::I32 => output.push_str("int_32"),
                    TypeTag::U32 => output.push_str("uint_32"),
                    TypeTag::I64 => output.push_str("int_64"),
                    TypeTag::U64 => output.push_str("uint_64"),
                    TypeTag::F32 => output.push_str("float"),
                    TypeTag::F64 => output.push_str("double"),
                    TypeTag::Object | TypeTag::Str => output.push_str("uint_64"),
                    TypeTag::Void => output.push_str("void"),
                    _ => unreachable!("return type can't be native")
                }
                if i < args.len() - 1 {
                    output.push_str(", ");
                }
            }

            output.push_str(");\n");
        }
        
        
        
        output
    }
}