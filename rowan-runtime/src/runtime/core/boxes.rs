use paste::paste;
use rowan_shared::TypeTag;
use crate::runtime::core::{VMClass, VMMember, VMMethod};
use crate::runtime::object::Object;
use crate::runtime::{Reference, Runtime};
use crate::context::BytecodeContext;
use std::{concat, stringify};

macro_rules! generate_box {
    ($name:ident, $typ:ty) => {
        paste! {
            #[repr(C)]
            pub struct $name {
                object: Object,
                value: $typ,
            }

            pub fn [< generate_ $typ _box >]() -> VMClass {
                let static_methods = vec![
                    VMMethod::new(
                        concat!("core::", stringify!($name), "::create"),
                         [< $typ _new >] as *const (),
                        vec![TypeTag::Object, TypeTag::$name]
                    )
                ];

                let members = vec![
                    VMMember::new(concat!("core::", stringify!($name), "::value"), TypeTag::$name),
                ];

                VMClass::new(concat!("core::", stringify!($name)), "core::Object",vec![], members, static_methods, Vec::new())
            }

            fn [< $typ _new >](_: &mut BytecodeContext, value: $typ) -> *mut $name {
                let int_box = Runtime::new_object(concat!("core::", stringify!($name))) as *mut $name;
                let int_box = unsafe { int_box.as_mut().unwrap() };
                int_box.value = value;
                int_box
            }
        }
    };
}

generate_box!(U8, u8);
generate_box!(U16, u16);
generate_box!(U32, u32);
generate_box!(U64, u64);
generate_box!(I8, i8);
generate_box!(I16, i16);
generate_box!(I32, i32);
generate_box!(I64, i64);
generate_box!(F32, f32);
generate_box!(F64, f64);
