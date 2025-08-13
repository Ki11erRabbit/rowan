pub struct StringTable {
    table: Vec<(usize, *const u8)>
}

impl StringTable {
    pub fn new() -> Self {
        StringTable {
            table: Vec::new(),
        }
    }

    pub fn add_string(&mut self, string: &str) -> usize {
        use std::alloc::*;
        let out = self.table.len();
        let layout = Layout::array::<u8>(string.len() + 1) // Adding one to null terminate it
            .expect("string layout is wrong or too big");
        let pointer = unsafe { alloc(layout) };
        if pointer.is_null() {
            eprintln!("Out of memory");
            handle_alloc_error(layout);
        }
        unsafe {
            std::ptr::copy_nonoverlapping(string.as_ptr(), pointer, string.len());
            pointer.add(string.len()).write(0);
        }

        self.table.push((string.len(), pointer as *const u8)); // Not adding one to prevent null terminator from being in the Rust &str
        
        out
    }

    pub fn add_static_string(&mut self, string: &'static str) -> usize {
        let size = string.len();
        let out = self.table.len();
        self.table.push((size, string.as_ptr()));

        out
    }

    pub fn get_string(&self, index: usize) -> &'static str {
        let (size, ptr) = self.table[index];
        let s = unsafe {
            let slice = std::slice::from_raw_parts(ptr, size);

            std::str::from_utf8_unchecked(slice)
        };
        s
    }
}

impl std::ops::Index<usize> for StringTable {
    type Output = str;
    fn index(&self, index: usize) -> &'static Self::Output {
        let (size, ptr) = self.table[index];
        let s = unsafe {
            let slice = std::slice::from_raw_parts(ptr, size);

            std::str::from_utf8_unchecked(slice)
        };
        s
    }
}

unsafe impl Send for StringTable {}
unsafe impl Sync for StringTable {}
