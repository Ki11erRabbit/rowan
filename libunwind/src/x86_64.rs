


#[derive(Debug, Copy, Clone)]
pub enum Register {
    RAX,
    RDX,
    RCX,
    RBX,
    RSI,
    RSP,
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15,
    RIP,
}

impl From<std::os::raw::c_int> for Register {
    fn from(c_int: std::os::raw::c_int) -> Register {
        match c_int {
            0 => Register::RAX,
            1 => Register::RDX,
            2 => Register::RCX,
            3 => Register::RBX,
            4 => Register::RSI,
            5 => Register::RSP,
            6 => Register::R8,
            7 => Register::R9,
            8 => Register::R10,
            9 => Register::R11,
            10 => Register::R12,
            11 => Register::R13,
            12 => Register::R14,
            13 => Register::R15,
            14 => Register::RIP,
            _ => panic!("invalid register")
        }
    }
}

impl Into<i32> for Register {
    fn into(self) -> i32 {
        match self {
            Register::RAX => 0,
            Register::RDX => 1,
            Register::RCX => 2,
            Register::RBX => 3,
            Register::RSI => 4,
            Register::RSP => 7,
            Register::R8 => 6,
            Register::R9 => 7,
            Register::R10 => 8,
            Register::R11 => 9,
            Register::R12 => 10,
            Register::R13 => 11,
            Register::R14 => 12,
            Register::R15 => 13,
            Register::RIP => 14,
        }
    }
}
