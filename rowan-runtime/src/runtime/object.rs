use std::collections::HashSet;
use crate::context::BytecodeContext;
use crate::runtime::core::Array;
use crate::runtime::garbage_collection::GarbageCollection;
use super::{Runtime, Reference, Symbol};



#[repr(C)]
pub struct Object {
    pub class: Symbol,
    pub parent_object: Reference,
    pub custom_drop: Option<extern "C" fn(&mut Object)>,
    //data: [u8]
}


impl Object {
    pub fn new(
        class: Symbol,
        parent: Reference,
        data_size: usize,
        drop: Option<extern "C" fn(&mut Object)>,
    ) -> *mut Object {
        use std::alloc::*;

        let layout = Layout::new::<Object>();
        //println!("layout size: {}", layout.size());
        let data_layout = Layout::array::<u8>(data_size).expect("Wrong layout or too big");

        let (whole_layout, size) = layout.extend(data_layout).expect("Wrong layout or too big");
        //println!("size: {}", whole_layout.size());
        //println!("padded size: {}", whole_layout.pad_to_align().size());
        let pointer = unsafe { alloc(whole_layout.pad_to_align()) };

        if pointer.is_null() {
            eprintln!("Out of memory in object allocate");
            handle_alloc_error(whole_layout);
        }        
        let pointer = pointer as *mut Object;
        unsafe {
            {
                let pointer = pointer as *mut u8;
                for i in 0..size {
                    pointer.add(i).write(0);
                }
            }

            std::ptr::write(pointer, Object {
                class,
                parent_object: parent,
                custom_drop: drop,
            });

        }
        GarbageCollection::update_heap_size((size_of::<Object>() + data_size) as i64);
        pointer
    }

    pub unsafe fn free(ptr: *mut Self, data_size: usize) {
        use std::alloc::*;
        unsafe {
            let self_ptr = ptr.as_mut().unwrap();
            if let Some(func) = self_ptr.custom_drop {
                func(self_ptr);
            }
        }

        let layout = Layout::new::<Object>();
        let data_layout = Layout::array::<u8>(data_size).expect("Wrong layout or too big");

        let (whole_layout, _) = layout.extend(data_layout).expect("Wrong layout or too big");

        GarbageCollection::update_heap_size(-((size_of::<Object>() + data_size) as i64));
        unsafe {
            dealloc(ptr as *mut u8, whole_layout);
        }
    }

    pub unsafe fn get<T: Sized>(&self, offset: usize) -> T {
        let mut pointer = self as *const Self as *const u8;
        unsafe {
            pointer = pointer.add(size_of::<Object>());
            pointer = pointer.add(offset);
            (pointer as *const T).read()
        }
    }
    
    pub unsafe fn set<T: Sized>(&mut self, offset: usize, value: T) {
        let mut pointer = self as *mut Self;
        pointer = unsafe { pointer.add(1) };
        let pointer = pointer.cast::<u8>();
        unsafe {

            std::ptr::write(pointer.add(offset) as *mut T, value);
            /*let mut old_pointer = pointer;
            println!("writing pointer: {:p}", pointer);
            for _ in 0..(offset / 8 + 8) {
                for i in 0..8 {
                    print!("\t{:x}", *old_pointer.add(i));
                    old_pointer = old_pointer.add(8);
                }
                println!("");
            }*/

        }
    }
    
    pub fn get_safe<T: Sized>(&self, mut offset: usize) -> Option<T> {
        let class = Runtime::get_class(self.class);
        let class = unsafe { class.as_ref()? };
        let mut pointer_offset = 0;
        for field in class.members.iter() {
            println!("field type: {:?}", field);
            let offset_part = field.get_size_and_padding();
            println!("offset_part: {}", offset_part);
            pointer_offset += field.get_size_and_padding();
            if offset == 0 {
                break;
            }
            offset -= 1;
        }

        if offset != 0 {
            return None;
        }

        unsafe {
            Some(self.get(pointer_offset))
        }
    }

    pub fn set_safe<T: Sized>(&mut self, mut offset: usize, value: T) -> Option<()> {
        let class = Runtime::get_class(self.class);
        let class = unsafe { class.as_ref().unwrap() };
        let mut pointer_offset = 0;
        for field in class.members.iter() {
            pointer_offset += field.get_size();
            if offset == 0 {
                break;
            }
            offset -= 1;
        }
        
        if offset != 0 {
            return None;
        }

        unsafe {
            self.set(pointer_offset, value);
            Some(())
        }
    }

    pub fn get_class_and_parent(&self) -> (Symbol, Reference) {
        let class_symbol = self.class;

        (class_symbol, self.parent_object)
    }

    pub fn add_custom_drop(&mut self, func: extern "C" fn(&mut Object)) {
        self.custom_drop = Some(func);
    }
    
    pub fn get_internal<T: Sized + Default>(context: &mut BytecodeContext, this: Reference, class_symbol: u64, parent_symbol: u64, offset: u64) -> T {
        let object = this;
        let object = unsafe { object.as_ref().unwrap() };

        if object.class == class_symbol as Symbol {
            return object.get_safe(offset as usize).unwrap();
        }

        let parent = unsafe { object.parent_object.as_ref().unwrap() };

        if parent.class == parent_symbol as Symbol {
            if parent.class == class_symbol as Symbol {
                return parent.get_safe(offset as usize).unwrap();
            }
            let Some(value) = Self::get_internal_helper(context, parent.parent_object, class_symbol, offset) else {
                todo!("Throw exception saying invalid offset")
            };
            return value;
        }

        todo!("Throw exception saying invalid offset")
    }
    fn get_internal_helper<T: Sized + Default>(context: &mut BytecodeContext, this: Reference, class_symbol: u64, offset: u64) -> Option<T> {
        let object = this;
        let object = unsafe { object.as_ref().unwrap() };

        if object.class == class_symbol as Symbol {
            return object.get_safe(offset as usize).unwrap();
        }

        let parent = unsafe { object.parent_object.as_ref().unwrap() };


        if parent.class == class_symbol as Symbol {
            return parent.get_safe(offset as usize).unwrap();
        }

        Self::get_internal_helper(context, parent.parent_object, class_symbol, offset)
    }

    pub extern "C" fn get_8(context: &mut BytecodeContext, this: Reference, class_symbol: u64, parent_symbol: u64, offset: u64) -> u8 {
        Self::get_internal(context, this, class_symbol, parent_symbol, offset)
    }

    pub extern "C" fn get_16(context: &mut BytecodeContext, this: Reference, class_symbol: u64, parent_symbol: u64, offset: u64) -> u16 {
        Self::get_internal(context, this, class_symbol, parent_symbol, offset)
    }

    pub extern "C" fn get_32(context: &mut BytecodeContext, this: Reference, class_symbol: u64, parent_symbol: u64, offset: u64) -> u32 {
        Self::get_internal(context, this, class_symbol, parent_symbol, offset)
    }

    pub extern "C" fn get_64(context: &mut BytecodeContext, this: Reference, class_symbol: u64, parent_symbol: u64, offset: u64) -> u64 {
        Self::get_internal(context, this, class_symbol, parent_symbol, offset)
    }

    pub extern "C" fn get_object(context: &mut BytecodeContext, this: Reference, class_symbol: u64, parent_symbol: u64, offset: u64) -> u64 {
        Self::get_internal(context, this, class_symbol, parent_symbol, offset)
    }

    pub extern "C" fn get_f32(context: &mut BytecodeContext, this: Reference, class_symbol: u64, parent_symbol: u64, offset: u64) -> f32 {
        Self::get_internal(context, this, class_symbol, parent_symbol, offset)
    }

    pub extern "C" fn get_f64(context: &mut BytecodeContext, this: Reference, class_symbol: u64, parent_symbol: u64, offset: u64) -> f64 {
        Self::get_internal(context, this, class_symbol, parent_symbol, offset)
    }

    pub fn set_internal<T: Sized + Default + Copy>(context: &mut BytecodeContext, this: Reference, class_symbol: u64, parent_symbol: u64, offset: u64, value: T) {
        let object = this;
        let object = unsafe { object.as_mut().unwrap() };

        if object.class == class_symbol as Symbol {
            object.set_safe(offset as usize, value).expect("TODO: throw exception saying invalid offset");
            return;
        }

        let parent = unsafe { object.parent_object.as_mut().unwrap() };

        if parent.class == parent_symbol as Symbol {
            if parent.class == class_symbol as Symbol {
                return parent.set_safe(offset as usize, value).unwrap();
            }
            let Some(_) = Self::set_internal_helper(context, parent.parent_object, class_symbol, offset, value) else {
                panic!("Throw exception saying invalid offset");
            };
            return;
        }

        todo!("Throw exception saying invalid offset")
    }
    fn set_internal_helper<T: Sized + Default + Copy>(context: &mut BytecodeContext, this: Reference, class_symbol: u64, offset: u64, value: T) -> Option<()> {
        let object = this;
        let object = unsafe { object.as_mut().unwrap() };

        if object.class == class_symbol as Symbol {
            return object.get_safe(offset as usize).unwrap();
        }

        let parent = unsafe { object.parent_object.as_mut().unwrap() };

        if parent.class == class_symbol as Symbol {
            return parent.get_safe(offset as usize).unwrap();
        }

        Self::set_internal_helper(context, parent.parent_object, class_symbol, offset, value)
    }

    pub extern "C" fn set_8(context: &mut BytecodeContext, this: Reference, class_symbol: u64, parent_symbol: u64, offset: u64, value: u8) {
        Self::set_internal(context, this, class_symbol, parent_symbol, offset, value);
    }

    pub extern "C" fn set_16(context: &mut BytecodeContext, this: Reference, class_symbol: u64, parent_symbol: u64, offset: u64, value: u16) {
        Self::set_internal(context, this, class_symbol, parent_symbol, offset, value);
    }

    pub extern "C" fn set_32(context: &mut BytecodeContext, this: Reference, class_symbol: u64, parent_symbol: u64, offset: u64, value: u32) {
        Self::set_internal(context, this, class_symbol, parent_symbol, offset, value);
    }

    pub extern "C" fn set_64(context: &mut BytecodeContext, this: Reference, class_symbol: u64, parent_symbol: u64, offset: u64, value: u64) {
        Self::set_internal(context, this, class_symbol, parent_symbol, offset, value);
    }

    pub extern "C" fn set_object(context: &mut BytecodeContext, this: Reference, class_symbol: u64, parent_symbol: u64, offset: u64, value: u64) {
        Self::set_internal(context, this, class_symbol, parent_symbol, offset, value);
    }

    pub extern "C" fn set_f32(context: &mut BytecodeContext, this: Reference, class_symbol: u64, parent_symbol: u64, offset: u64, value: f32) {
        Self::set_internal(context, this, class_symbol, parent_symbol, offset, value);
    }

    pub extern "C" fn set_f64(context: &mut BytecodeContext, this: Reference, class_symbol: u64, parent_symbol: u64, offset: u64, value: f64) {
        Self::set_internal(context, this, class_symbol, parent_symbol, offset, value);
    }

    pub fn garbage_collect(this: Reference, live_objects: &mut HashSet<Reference>) {
        let object_ptr = this;
        let object = unsafe { object_ptr.as_ref() };
        let Some(object) = object else {
            return
        };

        live_objects.insert(object.parent_object);
        let class = Runtime::get_class(object.class);
        let class = unsafe { class.as_ref().unwrap() };
        let live_objects_indices = class.get_object_member_indices();
        for index in live_objects_indices {
            let result = object.get_safe(index).unwrap();
            Self::garbage_collect(result, live_objects);
        }

        let array_object = Runtime::get_class_symbol("core::Arrayobject");
        if array_object == object.class {
            let object_ptr = object_ptr as *mut Array;
            let object = unsafe { object_ptr.as_ref().unwrap() };
            let length = object.length;
            let pointer = object.buffer as *const Reference;
            for i in 0..length {
                unsafe {
                    let reference = pointer.add(i as usize).read();
                    Self::garbage_collect(reference, live_objects);
                }
            }
        } // TODO: do this for backtrace as well since it holds string objects

    }
}
