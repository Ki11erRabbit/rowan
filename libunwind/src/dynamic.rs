

use libunwind_sys as unwind;

pub struct DynamicInfo {
    info: unwind::unw_dyn_info_t,
}

impl DynamicInfo {
    pub fn register() -> DynamicInfo {
        let mut info = unwind::unw_dyn_info_t {
            end_ip: unsafe { std::mem::zeroed() },
            format: unsafe { std::mem::zeroed() },
            gp: unsafe { std::mem::zeroed() },
            load_offset: unsafe { std::mem::zeroed() },
            next: unsafe { std::mem::zeroed() },
            pad: unsafe { std::mem::zeroed() },
            prev: unsafe { std::mem::zeroed() },
            start_ip: unsafe { std::mem::zeroed() },
            u: unsafe { std::mem::zeroed() },
        };

        unsafe {
            unwind::unw_dyn_register(&mut info)
        }

        DynamicInfo {
            info
        }
    }
}

impl Drop for DynamicInfo {
    fn drop(&mut self) {
        unsafe {
            unwind::unw_dyn_cancel(&mut self.info)
        }
    }
}
