use super::{Reference, Symbol};



#[repr(C)]
pub struct Object {
    pub class: Symbol,
    pub parent_objects: Box<[Reference]>,
    pub children: Vec<Reference>,
    pub custom_drop: Option<fn(&mut Object)>,
    //data: [u8]
}


impl Object {
    pub fn new(class: Symbol, parents: Box<[Reference]>, data_size: usize) -> *mut Object {
        use std::alloc::*;

        let layout = Layout::new::<Object>();
        let data_layout = Layout::array::<u8>(data_size).expect("Wrong layout or too big");

        let (whole_layout, _) = layout.extend(data_layout).expect("Wrong layout or too big");

        let pointer = unsafe { alloc(whole_layout) };

        if pointer.is_null() {
            eprintln!("Out of memory in object allocate");
            handle_alloc_error(whole_layout);
        }
        let pointer = pointer as *mut Object;
        unsafe {
            std::ptr::write(pointer, Object {
                class,
                parent_objects: parents,
                children: Vec::new(),
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
            drop(ptr.read().children);
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
        let mut pointer = self as *const Self;
        unsafe {
            pointer = pointer.add(size_of::<Object>());
            pointer = pointer.add(offset);
            (pointer as *const T).read()
        }
    }
    
    pub unsafe fn set<T: Sized>(&mut self, offset: usize, value: T) {
        let mut pointer = self as *mut Self;
        unsafe {
            pointer = pointer.add(size_of::<Object>());
            pointer = pointer.add(offset);
            std::ptr::write(pointer as *mut T, value);
        }
    }

    pub fn get_class_and_parents(&self) -> (Symbol, &Box<[Reference]>) {
        let class_symbol = self.class;

        (class_symbol, &self.parent_objects)
    }

    pub fn add_custom_drop(&mut self, func: fn(&mut Object)) {
        self.custom_drop = Some(func);
    }
}
