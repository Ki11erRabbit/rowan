use std::collections::HashSet;
use crate::runtime::core::Array;
use super::{Context, Reference, Symbol};



#[repr(C)]
pub struct Object {
    pub class: Symbol,
    pub parent_objects: Box<[Reference]>,
    pub custom_drop: Option<extern "C" fn(&mut Object)>,
    //data: [u8]
}


impl Object {
    pub fn new(
        class: Symbol,
        parents: Box<[Reference]>,
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
                parent_objects: parents,
                custom_drop: drop,
            });

        }
        pointer
    }

    pub unsafe fn free(ptr: *mut Self, data_size: usize) {
        use std::alloc::*;
        unsafe {
            // Dropping boxed and vec members from pointer
            drop(ptr.read().parent_objects);
            let self_ptr = ptr.as_mut().unwrap();
            if let Some(func) = self_ptr.custom_drop {
                func(self_ptr);
            }
        }

        let layout = Layout::new::<Object>();
        let data_layout = Layout::array::<u8>(data_size).expect("Wrong layout or too big");

        let (whole_layout, _) = layout.extend(data_layout).expect("Wrong layout or too big");

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
        pointer = pointer.add(1);
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
        let class = Context::get_class(self.class);
        let class = unsafe { class.as_ref()? };
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
            Some(self.get(pointer_offset))
        }
    }

    pub fn set_safe<T: Sized>(&mut self, mut offset: usize, value: T) -> Option<()> {
        let class = Context::get_class(self.class);
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

    pub fn get_class_and_parents(&self) -> (Symbol, &Box<[Reference]>) {
        let class_symbol = self.class;

        (class_symbol, &self.parent_objects)
    }

    pub fn add_custom_drop(&mut self, func: extern "C" fn(&mut Object)) {
        self.custom_drop = Some(func);
    }
    
    pub fn get_internal<T: Sized + Default>(context: &mut Context, this: Reference, class_symbol: u64, parent_symbol: u64, offset: u64) -> T {
        let object = this;
        let object = unsafe { object.as_ref().unwrap() };

        if object.class == class_symbol as Symbol {
            return object.get_safe(offset as usize).unwrap();
        }

        for parent in object.parent_objects.iter() {
            let parent = unsafe { parent.as_ref().unwrap() };

            if parent.class == parent_symbol as Symbol {
                if parent.class == class_symbol as Symbol {
                    return parent.get_safe(offset as usize).unwrap();
                }
                for parent in parent.parent_objects.iter() {
                    let Some(value) = Self::get_internal_helper(context, *parent, class_symbol, offset) else {
                        continue;
                    };
                    return value;
                }
            }
        }

        todo!("Throw exception saying invalid offset")
    }
    fn get_internal_helper<T: Sized + Default>(context: &mut Context, this: Reference, class_symbol: u64, offset: u64) -> Option<T> {
        let object = this;
        let object = unsafe { object.as_ref().unwrap() };

        if object.class == class_symbol as Symbol {
            return object.get_safe(offset as usize).unwrap();
        }

        for parent in object.parent_objects.iter() {
            let parent = unsafe { parent.as_ref().unwrap() };


            if parent.class == class_symbol as Symbol {
                return parent.get_safe(offset as usize).unwrap();
            }
            for parent in parent.parent_objects.iter() {
                let Some(value) = Self::get_internal_helper(context, *parent, class_symbol, offset) else {
                    continue;
                };
                return Some(value);
            }
        }

        None
    }

    pub extern "C" fn get_8(context: &mut Context, this: Reference, class_symbol: u64, parent_symbol: u64, offset: u64) -> i8 {
        Self::get_internal(context, this, class_symbol, parent_symbol, offset)
    }

    pub extern "C" fn get_16(context: &mut Context, this: Reference, class_symbol: u64, parent_symbol: u64, offset: u64) -> i16 {
        Self::get_internal(context, this, class_symbol, parent_symbol, offset)
    }

    pub extern "C" fn get_32(context: &mut Context, this: Reference, class_symbol: u64, parent_symbol: u64, offset: u64) -> i32 {
        Self::get_internal(context, this, class_symbol, parent_symbol, offset)
    }

    pub extern "C" fn get_64(context: &mut Context, this: Reference, class_symbol: u64, parent_symbol: u64, offset: u64) -> i64 {
        Self::get_internal(context, this, class_symbol, parent_symbol, offset)
    }

    pub extern "C" fn get_object(context: &mut Context, this: Reference, class_symbol: u64, parent_symbol: u64, offset: u64) -> i64 {
        Self::get_internal(context, this, class_symbol, parent_symbol, offset)
    }

    pub extern "C" fn get_f32(context: &mut Context, this: Reference, class_symbol: u64, parent_symbol: u64, offset: u64) -> f32 {
        Self::get_internal(context, this, class_symbol, parent_symbol, offset)
    }

    pub extern "C" fn get_f64(context: &mut Context, this: Reference, class_symbol: u64, parent_symbol: u64, offset: u64) -> f64 {
        Self::get_internal(context, this, class_symbol, parent_symbol, offset)
    }

    pub fn set_internal<T: Sized + Default + Copy>(context: &mut Context, this: Reference, class_symbol: u64, parent_symbol: u64, offset: u64, value: T) {
        let object = this;
        let object = unsafe { object.as_mut().unwrap() };

        if object.class == class_symbol as Symbol {
            object.set_safe(offset as usize, value).expect("TODO: throw exception saying invalid offset");
            return;
        }

        for parent in object.parent_objects.iter() {
            let parent = unsafe { parent.as_mut().unwrap() };

            if parent.class == parent_symbol as Symbol {
                if parent.class == class_symbol as Symbol {
                    return parent.set_safe(offset as usize, value).unwrap();
                }
                for parent in parent.parent_objects.iter() {
                    let Some(_) = Self::set_internal_helper(context, *parent, class_symbol, offset, value) else {
                        continue;
                    };
                    return;
                }
            }
        }

        todo!("Throw exception saying invalid offset")
    }
    fn set_internal_helper<T: Sized + Default + Copy>(context: &mut Context, this: Reference, class_symbol: u64, offset: u64, value: T) -> Option<()> {
        let object = this;
        let object = unsafe { object.as_mut().unwrap() };

        if object.class == class_symbol as Symbol {
            return object.get_safe(offset as usize).unwrap();
        }

        for parent in object.parent_objects.iter() {
            let parent = unsafe { parent.as_mut().unwrap() };

            if parent.class == class_symbol as Symbol {
                return parent.get_safe(offset as usize).unwrap();
            }
            for parent in parent.parent_objects.iter() {
                let Some(_) = Self::set_internal_helper(context, *parent, class_symbol, offset, value) else {
                    continue;
                };
                return Some(());
            }
        }

        None
    }

    pub extern "C" fn set_8(context: &mut Context, this: Reference, class_symbol: u64, parent_symbol: u64, offset: u64, value: i8) {
        Self::set_internal(context, this, class_symbol, parent_symbol, offset, value);
    }

    pub extern "C" fn set_16(context: &mut Context, this: Reference, class_symbol: u64, parent_symbol: u64, offset: u64, value: i16) {
        Self::set_internal(context, this, class_symbol, parent_symbol, offset, value);
    }

    pub extern "C" fn set_32(context: &mut Context, this: Reference, class_symbol: u64, parent_symbol: u64, offset: u64, value: i32) {
        Self::set_internal(context, this, class_symbol, parent_symbol, offset, value);
    }

    pub extern "C" fn set_64(context: &mut Context, this: Reference, class_symbol: u64, parent_symbol: u64, offset: u64, value: i64) {
        Self::set_internal(context, this, class_symbol, parent_symbol, offset, value);
    }

    pub extern "C" fn set_object(context: &mut Context, this: Reference, class_symbol: u64, parent_symbol: u64, offset: u64, value: i64) {
        Self::set_internal(context, this, class_symbol, parent_symbol, offset, value);
    }

    pub extern "C" fn set_f32(context: &mut Context, this: Reference, class_symbol: u64, parent_symbol: u64, offset: u64, value: f32) {
        Self::set_internal(context, this, class_symbol, parent_symbol, offset, value);
    }

    pub extern "C" fn set_f64(context: &mut Context, this: Reference, class_symbol: u64, parent_symbol: u64, offset: u64, value: f64) {
        Self::set_internal(context, this, class_symbol, parent_symbol, offset, value);
    }

    pub fn garbage_collect(this: Reference, live_objects: &mut HashSet<Reference>) {
        let object_ptr = this;
        let object = unsafe { object_ptr.as_ref().unwrap() };

        for parent in object.parent_objects.iter() {
            live_objects.insert(*parent);
            Self::garbage_collect(*parent, live_objects);
        }
        let class = Context::get_class(object.class);
        let class = unsafe { class.as_ref().unwrap() };
        let live_objects_indices = class.get_object_member_indices();
        for index in live_objects_indices {
            Self::garbage_collect(object.get_safe(index as usize).unwrap(), live_objects);
        }

        let array_object = Context::get_class_symbol("Arrayobject");
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
