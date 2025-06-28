use std::ffi::CStr;
use libunwind_sys as unwind;
use crate::machine::Register;

/*
pub struct DynamicInfo {
    info: unwind::unw_dyn_info_t,
}

impl DynamicInfo {

    pub fn new_builder() -> DynamicInfoBuilder {
        DynamicInfoBuilder::new()
    }

    pub fn register(&mut self) {
        unsafe {
            unwind::unw_dyn_register(&mut self.info)
        }
    }
}

/*impl Drop for DynamicInfo {
    fn drop(&mut self) {
        unsafe {
            unwind::unw_dyn_cancel(&mut self.info)
        }
    }
}*/

pub struct DynamicInfoBuilder {
    info: unwind::unw_dyn_info_t,
}

impl DynamicInfoBuilder {
    pub fn new() -> DynamicInfoBuilder {
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

        Self {
            info
        }
    }

    pub fn build(self) -> DynamicInfo {
        DynamicInfo { info: self.info }
    }

    pub fn ip(mut self, start_ip: *const (), end_ip: *const ()) -> DynamicInfoBuilder {
        self.info.start_ip = start_ip as usize as u64;
        self.info.end_ip = end_ip as usize as u64;
        self
    }

    /*pub fn format(mut self, format: DynInfoFormat) -> DynamicInfoBuilder {
        let format = match format {
            DynInfoFormat::Dynamic => 0,
            DynInfoFormat::Table => 1,
            DynInfoFormat::RemoteTable => 2,
            DynInfoFormat::ArmExidx => 3,
            DynInfoFormat::IpOffset => 4,
        };
        self.info.format = format;
        self
    }*/

    pub fn proc_info(mut self, info: DynProcInfo) -> DynamicInfoBuilder {
        self.info.pi = info.to_proc_info();
        self.info.format = 0; // Info Dynamic
        self
    }
}

pub enum DynInfoFormat {
    Dynamic,
    Table,
    RemoteTable,
    /// ARM specific unwind info
    ArmExidx,
    IpOffset,
}

pub struct DynProcInfo<'a> {
    name: &'a CStr,
    handler: *const (),
    regions: Vec<RegionInfo>,
}

impl<'a> DynProcInfo<'a> {
    pub fn new(name: &'a CStr, handler: *const (), regions: Vec<RegionInfo>) -> DynProcInfo<'a> {
        Self {
            name,
            handler,
            regions,
        }
    }

    fn to_proc_info(self) -> unwind::unw_dyn_proc_info_t {
        let mut output = unwind::unw_dyn_proc_info_t {
            name_ptr: unsafe { std::mem::zeroed() },
            handler: self.handler as u64,
            flags: 0,
            pad0: 0,
            regions: unsafe { std::mem::zeroed() },
        };

        output.name_ptr = self.name.as_ptr() as u64;
        output.handler = self.handler as u64;
        let mut regions_iter = self.regions.into_iter();
        let mut prev = regions_iter.next().unwrap().to_region();
        output.regions = prev;
        for region in regions_iter {
            let current = region.to_region();
            prev.next = current;
            prev = current;
        }
        output
    }
}

pub struct RegionInfo {
    instructions_count: i32,
    ops: Vec<DynamicOperation>
}

impl RegionInfo {
    pub fn new(instructions_count: i32, ops: Vec<DynamicOperation>) -> RegionInfo {
        Self {
            instructions_count,
            ops
        }
    }

    fn to_region(self) -> *mut unwind::unw_dyn_region_info_t {
        use std::alloc::*;
        let layout = Layout::new::<unwind::unw_dyn_region_info_t>();
        let op_layout = Layout::array::<unwind::unw_dyn_op_t>(self.ops.len()).unwrap();
        let (layout, _) = layout.extend(op_layout).unwrap();
        let pointer = unsafe {
            alloc(layout)
        };
        if pointer.is_null() {
            handle_alloc_error(layout);
        }
        let pointer = pointer as *mut unwind::unw_dyn_region_info;
        let region = unsafe { pointer.as_mut().unwrap() };
        region.next = std::ptr::null_mut();
        region.insn_count = self.instructions_count;
        region.op_count = self.ops.len() as u32;
        unsafe {
            let pointer = pointer.add(std::mem::size_of::<unwind::unw_dyn_region_info>());
            for i in 0..self.ops.len() {
                pointer.add(i).write(self.ops[i].to_operation())
            }
        }
        pointer
    }
}

#[derive(Debug, Copy, Clone)]
pub struct DynamicOperation {
    tag: OperationTag,
    qp: bool,
    register: Register,
    when: i32,
    value: usize,
}

impl DynamicOperation {
    pub fn new(tag: OperationTag, qp: bool, register: Register, when: i32, value: usize) -> DynamicOperation {
        Self {
            tag,
            qp,
            register,
            when,
            value,
        }
    }

    fn to_operation(self) -> unwind::unw_dyn_op_t {
        let tag = self.tag.to_u8();
        let qp = self.qp as u8;
        let register = self.register.into();
        let when = self.when;
        let value = self.value;

        unwind::unw_dyn_op_t {
            tag,
            qp,
            reg: register,
            when,
            value
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum OperationTag {
    Stop,
    SaveRegisters,
    SpillFloatingPoint,
    SpillStackPointerRelative,
    Add,
    PopFrames,
    LabelState,
    CopyState,
    Alias,
}

impl OperationTag {
    pub fn to_u8(self) -> u8 {
        match self {
            OperationTag::Stop => 0,
            OperationTag::SaveRegisters => 1,
            OperationTag::SpillFloatingPoint => 2,
            OperationTag::SpillStackPointerRelative => 3,
            OperationTag::Add => 4,
            OperationTag::PopFrames => 5,
            OperationTag::LabelState => 6,
            OperationTag::CopyState => 7,
            OperationTag::Alias => 8,
        }
    }
}
*/
