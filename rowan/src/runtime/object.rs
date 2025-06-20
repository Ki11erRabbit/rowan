use super::{Context, Reference, Symbol};



#[repr(C)]
pub struct Object {
    pub class: Symbol,
    pub parent_objects: Box<[Reference]>,
    pub custom_drop: Option<fn(&mut Object)>,
    //data: [u8]
}


impl Object {
    pub fn new(class: Symbol, parents: Box<[Reference]>, data_size: usize) -> *mut Object {
        use std::alloc::*;

        let layout = Layout::new::<Object>();
        println!("layout size: {}", layout.size());
        let data_layout = Layout::array::<u8>(data_size).expect("Wrong layout or too big");

        let (whole_layout, _) = layout.extend(data_layout).expect("Wrong layout or too big");
        println!("size: {}", whole_layout.size());
        let pointer = unsafe { alloc(whole_layout.pad_to_align()) };

        if pointer.is_null() {
            eprintln!("Out of memory in object allocate");
            handle_alloc_error(whole_layout);
        }        
        let pointer = pointer as *mut Object;
        unsafe {
            std::ptr::write(pointer, Object {
                class,
                parent_objects: parents,
                custom_drop: None,
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
        let mut pointer = self as *mut Self as *mut u8;
        let mut pointer_start = pointer as usize;
        println!("\npointer: {:p}", pointer);
        unsafe {
            pointer = pointer.add(size_of::<Object>());
            println!("pointer: {:p}", pointer);
            pointer = pointer.add(offset);
            println!("pointer: {:p}", pointer);
            std::ptr::write(pointer as *mut T, value);
        }
        let pointer_end = pointer as usize + size_of::<T>();
        println!("pointer size: {}", pointer_end - pointer_start);
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

        println!("pointer offset {pointer_offset}");

        unsafe {
            self.set(pointer_offset, value);
            Some(())
        }
    }

    pub fn get_class_and_parents(&self) -> (Symbol, &Box<[Reference]>) {
        let class_symbol = self.class;

        (class_symbol, &self.parent_objects)
    }

    pub fn add_custom_drop(&mut self, func: fn(&mut Object)) {
        self.custom_drop = Some(func);
    }
    
    pub fn get_internal<T: Sized + Default>(context: &mut Context, this: Reference, class_symbol: u64, parent_symbol: u64, offset: u64) -> T {
        let Some(object) = context.get_object(this) else {
            let exception = Context::new_object("NullPointerException");
            context.set_exception(exception);
            return T::default();
        };
        let object = unsafe { object.as_ref().unwrap() };

        if object.class == class_symbol as Symbol {
            return object.get_safe(offset as usize).unwrap();
        }

        for parent in object.parent_objects.iter() {
            let Some(parent) = context.get_object(*parent) else {
                let exception = Context::new_object("NullPointerException");
                context.set_exception(exception);
                return T::default();
            };
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
        let Some(object) = context.get_object(this) else {
            let exception = Context::new_object("NullPointerException");
            context.set_exception(exception);
            return Some(T::default());
        };
        let object = unsafe { object.as_ref().unwrap() };

        if object.class == class_symbol as Symbol {
            return object.get_safe(offset as usize).unwrap();
        }

        for parent in object.parent_objects.iter() {
            let Some(parent) = context.get_object(*parent) else {
                let exception = Context::new_object("NullPointerException");
                context.set_exception(exception);
                return None;
            };
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
        let Some(object) = context.get_object(this) else {
            let exception = Context::new_object("NullPointerException");
            context.set_exception(exception);
            return;
        };
        let object = unsafe { object.as_mut().unwrap() };

        if object.class == class_symbol as Symbol {
            object.set_safe(offset as usize, value).expect("TODO: throw exception saying invalid offset");
            return;
        }

        for parent in object.parent_objects.iter() {
            let Some(parent) = context.get_object(*parent) else {
                let exception = Context::new_object("NullPointerException");
                context.set_exception(exception);
                return;
            };
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
        let Some(object) = context.get_object(this) else {
            let exception = Context::new_object("NullPointerException");
            context.set_exception(exception);
            return None;
        };
        let object = unsafe { object.as_mut().unwrap() };

        if object.class == class_symbol as Symbol {
            return object.get_safe(offset as usize).unwrap();
        }

        for parent in object.parent_objects.iter() {
            let Some(parent) = context.get_object(*parent) else {
                let exception = Context::new_object("NullPointerException");
                context.set_exception(exception);
                return None;
            };
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
}
