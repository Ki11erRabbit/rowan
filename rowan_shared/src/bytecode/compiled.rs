//! This file contains the bytecode for the compiled class file.
//! This bytecode needs to be linked by the runtime to be executable.
use crate::TypeTag;

/// BlockIdOffset is an offset to a block of code
pub type BlockIdOffset = i64;
pub type StringIndex = u64;

#[derive(Debug)]
pub enum Bytecode {
    /// No operation
    Nop,
    /// Breakpoint
    Breakpoint,
    /// Load constants
    LoadU8(u8),
    /// Load constants
    LoadU16(u16),
    /// Load constants
    LoadU32(u32),
    /// Load constants
    LoadU64(u64),
    /// Load constants
    LoadI8(i8),
    /// Load constants
    LoadI16(i16),
    /// Load constants
    LoadI32(i32),
    /// Load constants
    LoadI64(i64),
    /// Load constants
    LoadF32(f32),
    /// Load constants
    LoadF64(f64),
    /// Load symbol
    LoadSymbol(StringIndex),
    // Stack operations
    /// Pop the top value off the stack
    Pop,
    /// Duplicate the top value on the stack
    Dup,
    /// Swap the top two values on the stack
    Swap,
    /// Store local variable
    StoreLocal(u8),
    /// Load local variable
    LoadLocal(u8),
    /// Store argument
    /// This is how we specify the call arguments for functions
    StoreArgument(u8),
    // Arithmetic operations
    /// Wrapping Addition
    AddInt,
    /// Wrapping Subtraction
    SubInt,
    /// Wrapping Multiplication
    MulInt,
    /// Wrapping Division
    DivSigned,
    /// Wrapping Division
    DivUnsigned,
    /// Wrapping Modulus
    ModSigned,
    /// Wrapping Modulus
    ModUnsigned,
    /// Wrapping Addition
    AddFloat,
    /// Wrapping Subtraction
    SubFloat,
    /// Wrapping Multiplication
    MulFloat,
    /// Wrapping Division
    DivFloat,
    /// Wrapping Modulus
    ModFloat,
    /// Saturating Addition
    SatAddIntUnsigned,
    /// Saturating Subtraction
    SatSubIntUnsigned,
    // Bitwise operations
    /// Bitwise AND
    And,
    /// Bitwise OR
    Or,
    /// Bitwise XOR
    Xor,
    /// Bitwise NOT
    Not,
    /// Shift Left
    Shl,
    /// Arithmetic Shift Right
    AShr,
    /// Logical Shift Right
    LShr,
    /// Negate
    Neg,
    // Comparison operations
    /// Equal comparison
    EqualSigned,
    /// Not equal comparison
    NotEqualSigned,
    /// Equal comparison
    EqualUnsigned,
    /// Not equal comparison
    NotEqualUnsigned,
    /// Greater than comparison
    GreaterSigned,
    /// Less than comparison
    LessSigned,
    /// Greater than or equal comparison
    GreaterOrEqualSigned,
    /// Less than or equal comparison
    LessOrEqualSigned,
    /// Greater than comparison
    GreaterUnsigned,
    /// Less than comparison
    LessUnsigned,
    /// Greater than or equal comparison
    GreaterOrEqualUnsigned,
    /// Less than or equal comparison
    LessOrEqualUnsigned,
    /// Equal comparison
    EqualFloat,
    /// Not equal comparison
    NotEqualFloat,
    /// Greater than comparison
    GreaterFloat,
    /// Less than comparison
    LessFloat,
    /// Greater than or equal comparison
    GreaterOrEqualFloat,
    /// Less than or equal comparison
    LessOrEqualFloat,
    // Type conversions
    /// Convert the top value on the stack to the specified type
    Convert(TypeTag),
    /// Convert the top value on the stack to the specified type via its binary representation
    BinaryConvert(TypeTag),
    /// Create an array of the specified type
    CreateArray(TypeTag),
    /// Get an element from an array of the specified type
    /// The top value on the stack is the index and the second value is the array
    ArrayGet(TypeTag),
    /// Set an element in an array of the specified type
    /// The top value on the stack is the value, the second value is the index and the third value is the array
    ArraySet(TypeTag),
    /// Create a new object of the specified class
    NewObject(StringIndex),
    /// Get a field from an object of the specified class
    /// The first StringIndex is the class name we are accessing, the second StringIndex is another classname
    /// that allows for selecting the particular parent to access the field.
    GetField(StringIndex, StringIndex, u64, TypeTag),
    /// Set a field in an object of the specified class
    /// The first StringIndex is the class name we are accessing, the second StringIndex is another classname
    /// that allows for selecting the particular parent to access the field.
    SetField(StringIndex, StringIndex, u64, TypeTag),
    /// Check if an object is of a specified class
    IsA(StringIndex),
    /// Invoke a virtual method on an object of the specified class
    /// The first is the class name that the method belongs to
    /// The second is the method name
    InvokeVirt(StringIndex, StringIndex),
    /// Invoke a virtual method on an object of the specified class
    /// The first is the class name that the method belongs to
    /// The second is the method name
    /// This is for tail recursion
    InvokeVirtTail(StringIndex, StringIndex),
    /// Invoke a static method from a class
    /// The first StringIndex is the class name
    /// The second StringIndex is the Method Name
    InvokeStatic(StringIndex, StringIndex),
    /// Invoke a static method from a class
    /// The first StringIndex is the class name
    /// The second StringIndex is the Method Name
    /// This is for tail recursion
    InvokeStaticTail(StringIndex, StringIndex),
    /// Get a static method from a class and construct an object with the method `call`
    /// The first StringIndex is the class name
    /// The second StringIndex is the Method Name
    GetStaticMethod(StringIndex, StringIndex),
    /// Access a static field on a class and get its value
    /// The StringIndex is the class name
    /// The u64 is the index of the static member
    /// The TypeTag is the type of the variable
    GetStaticMember(StringIndex, u64, TypeTag),
    /// Access a static field on a class and update its value
    /// The StringIndex is the class name
    /// The u64 is the index of the static member
    /// The TypeTag is the type of the variable
    SetStaticMember(StringIndex, u64, TypeTag),
    GetStrRef(StringIndex),
    /// Return from a function
    /// This pops the top value off the stack and returns it
    Return,
    /// Return from a function
    /// This pops nothing off the stack and returns void
    ReturnVoid,
    /// Register a classname as a catchable exception
    /// StringIndex is the classname
    /// BlockIdOffset is the offset to the handler block
    RegisterException(StringIndex, BlockIdOffset),
    /// Unregister a classname as a catchable exception
    UnregisterException(StringIndex),
    /// Throw an exception, pops an object off of the stack and throws it
    Throw,
    /// Start a new block of code
    StartBlock(u64),
    /// Goto a block of code via an offset from the current block
    Goto(BlockIdOffset),
    /// If a condition is true, goto one block, otherwise goto another
    If(BlockIdOffset, BlockIdOffset),
    /// Switch to one of several blocks based on the value of the top of the stack
    /// The first block is the default case
    Switch(Vec<BlockIdOffset>, Option<BlockIdOffset>),
}

impl Bytecode {
    pub fn try_from(iter: &mut dyn Iterator<Item = &u8>) -> Result<Vec<Bytecode>, &'static str> {
        let mut result = Vec::new();
        while let Some(opcode) = iter.next() {
            match opcode {
                0 => result.push(Bytecode::Nop),
                1 => result.push(Bytecode::Breakpoint),
                2 => {
                    let value = *iter.next().ok_or("Expected u8 value")?;
                    result.push(Bytecode::LoadU8(value));
                },
                3 => {
                    let value = u16::from_le_bytes([
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?
                    ]);
                    result.push(Bytecode::LoadU16(value));
                },
                4 => {
                    let value = u32::from_le_bytes([
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?
                    ]);
                    result.push(Bytecode::LoadU32(value));
                },
                5 => {
                    let value = u64::from_le_bytes([
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                    ]);
                    result.push(Bytecode::LoadU64(value));
                }
                6 => {
                    let value = i8::from_le_bytes([*iter.next().ok_or("Expected u8 value")?]);
                    result.push(Bytecode::LoadI8(value));
                },
                7 => {
                    let value = i16::from_le_bytes([
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?
                    ]);
                    result.push(Bytecode::LoadI16(value));
                },
                8 => {
                    let value = i32::from_le_bytes([
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?
                    ]);
                    result.push(Bytecode::LoadI32(value));
                },
                9 => {
                    let value = i64::from_le_bytes([
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                    ]);
                    result.push(Bytecode::LoadI64(value));
                },
                10 => {
                    let value = f32::from_le_bytes([
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?
                    ]);
                    result.push(Bytecode::LoadF32(value));
                },
                11 => {
                    let value = f64::from_le_bytes([
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                    ]);
                    result.push(Bytecode::LoadF64(value));
                },
                12 => {
                    let index = u64::from_le_bytes([
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                    ]);
                    result.push(Bytecode::LoadSymbol(index));
                },
                13 => result.push(Bytecode::Pop),
                14 => result.push(Bytecode::Dup),
                15 => result.push(Bytecode::Swap),
                16 => {
                    let index = *iter.next().ok_or("Expected u8 value")?;
                    result.push(Bytecode::StoreLocal(index));
                },
                17 => {
                    let index = *iter.next().ok_or("Expected u8 value")?;
                    result.push(Bytecode::LoadLocal(index));
                },
                18 => {
                    let index = *iter.next().ok_or("Expected u8 value")?;
                    result.push(Bytecode::StoreArgument(index));
                },
                19 => result.push(Bytecode::AddInt),
                20 => result.push(Bytecode::SubInt),
                21 => result.push(Bytecode::MulInt),
                22 => result.push(Bytecode::DivSigned),
                23 => result.push(Bytecode::DivUnsigned),
                24 => result.push(Bytecode::ModSigned),
                25 => result.push(Bytecode::ModUnsigned),
                26 => result.push(Bytecode::AddFloat),
                27 => result.push(Bytecode::SubFloat),
                28 => result.push(Bytecode::MulFloat),
                29 => result.push(Bytecode::DivFloat),
                30 => result.push(Bytecode::ModFloat),
                31 => result.push(Bytecode::SatAddIntUnsigned),
                32 => result.push(Bytecode::SatSubIntUnsigned),
                33 => result.push(Bytecode::And),
                34 => result.push(Bytecode::Or),
                35 => result.push(Bytecode::Xor),
                36 => result.push(Bytecode::Not),
                37 => result.push(Bytecode::Shl),
                38 => result.push(Bytecode::AShr),
                39 => result.push(Bytecode::LShr),
                40 => result.push(Bytecode::Neg),
                41 => result.push(Bytecode::EqualSigned),
                42 => result.push(Bytecode::NotEqualSigned),
                43 => result.push(Bytecode::EqualUnsigned),
                44 => result.push(Bytecode::NotEqualSigned),
                45 => result.push(Bytecode::GreaterSigned),
                46 => result.push(Bytecode::LessSigned),
                47 => result.push(Bytecode::GreaterOrEqualSigned),
                48 => result.push(Bytecode::LessOrEqualSigned),
                49 => result.push(Bytecode::GreaterUnsigned),
                50 => result.push(Bytecode::LessUnsigned),
                51 => result.push(Bytecode::GreaterOrEqualUnsigned),
                52 => result.push(Bytecode::LessOrEqualUnsigned),
                53 => result.push(Bytecode::EqualFloat),
                54 => result.push(Bytecode::NotEqualFloat),
                55 => result.push(Bytecode::GreaterFloat),
                56 => result.push(Bytecode::LessFloat),
                57 => result.push(Bytecode::GreaterFloat),
                58 => result.push(Bytecode::LessOrEqualFloat),
                59 => {
                    let tag = TypeTag::from(*iter.next().ok_or("Expected u8 value")?);
                    result.push(Bytecode::Convert(tag));
                },
                60 => {
                    let tag = TypeTag::from(*iter.next().ok_or("Expected u8 value")?);
                    result.push(Bytecode::BinaryConvert(tag));
                },
                61 => {
                    let tag = TypeTag::from(*iter.next().ok_or("Expected u8 value")?);
                    result.push(Bytecode::CreateArray(tag));
                },
                62 => {
                    let tag = TypeTag::from(*iter.next().ok_or("Expected u8 value")?);
                    result.push(Bytecode::ArrayGet(tag));
                },
                63 => {
                    let tag = TypeTag::from(*iter.next().ok_or("Expected u8 value")?);
                    result.push(Bytecode::ArraySet(tag));
                },
                64 => {
                    let index = u64::from_le_bytes([
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                    ]);
                    result.push(Bytecode::NewObject(index));
                },
                65 => {
                    let class_name = u64::from_le_bytes([
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                    ]);
                    let parent_class_name = u64::from_le_bytes([
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                    ]);
                    let member_index = u64::from_le_bytes([
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                    ]);
                    let tag = TypeTag::from(*iter.next().ok_or("Expected u8 value")?);
                    result.push(Bytecode::GetField(class_name, parent_class_name, member_index, tag));
                },
                66 => {
                    let class_name = u64::from_le_bytes([
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                    ]);
                    let parent_class_name = u64::from_le_bytes([
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                    ]);
                    let member_index = u64::from_le_bytes([
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                    ]);
                    let tag = TypeTag::from(*iter.next().ok_or("Expected u8 value")?);
                    result.push(Bytecode::SetField(class_name, parent_class_name, member_index, tag));
                },
                67 => {
                    let class_name = u64::from_le_bytes([
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                    ]);
                    result.push(Bytecode::IsA(class_name));
                },
                68 => {
                    let class_name = u64::from_le_bytes([
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                    ]);
                    let method_name = u64::from_le_bytes([
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                    ]);
                    result.push(Bytecode::InvokeVirt(class_name, method_name));
                },
                69 => {
                    let class_name = u64::from_le_bytes([
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                    ]);
                    let method_name = u64::from_le_bytes([
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                    ]);
                    result.push(Bytecode::InvokeVirtTail(class_name, method_name));
                },
                70 => {
                    let class_name = u64::from_le_bytes([
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                    ]);
                    let method_name = u64::from_le_bytes([
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                    ]);
                    result.push(Bytecode::InvokeStatic(class_name, method_name));
                },
                71 => {
                    let class_name = u64::from_le_bytes([
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                    ]);
                    let method_name = u64::from_le_bytes([
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                    ]);
                    result.push(Bytecode::InvokeStaticTail(class_name, method_name));
                },
                72 => {
                    let class_name = u64::from_le_bytes([
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                    ]);
                    let method_name = u64::from_le_bytes([
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                    ]);
                    result.push(Bytecode::GetStaticMethod(class_name, method_name));
                },
                73 => {
                    let class_name = u64::from_le_bytes([
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                    ]);
                    let index = u64::from_le_bytes([
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                    ]);
                    let type_tag = TypeTag::from(*iter.next().ok_or("Expected TypeTag")?);
                    result.push(Bytecode::GetStaticMember(class_name, index, type_tag));
                }
                74 => {
                    let class_name = u64::from_le_bytes([
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                    ]);
                    let index = u64::from_le_bytes([
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                    ]);
                    let type_tag = TypeTag::from(*iter.next().ok_or("Expected TypeTag")?);
                    result.push(Bytecode::SetStaticMember(class_name, index, type_tag));
                }
                75 => {
                    let index = u64::from_le_bytes([
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                    ]);
                    result.push(Bytecode::GetStrRef(index));
                },
                76 => result.push(Bytecode::Return),
                77 => result.push(Bytecode::ReturnVoid),
                78 => {
                    let index = u64::from_le_bytes([
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                    ]);
                    let offset = i64::from_le_bytes([
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                    ]);
                    result.push(Bytecode::RegisterException(index, offset))
                }
                79 => {
                    let index = u64::from_le_bytes([
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                    ]);
                    result.push(Bytecode::UnregisterException(index))
                }
                80 => result.push(Bytecode::Throw),
                81 => {
                    let id = u64::from_le_bytes([
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                    ]);
                    result.push(Bytecode::StartBlock(id));
                },
                82 => {
                    let offset = i64::from_le_bytes([
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                    ]);
                    result.push(Bytecode::Goto(offset));
                },
                83 => {
                    let true_offset = i64::from_le_bytes([
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                    ]);
                    let false_offset = i64::from_le_bytes([
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                    ]);
                    result.push(Bytecode::If(true_offset, false_offset));
                },
                84 => {
                    let cases_len = u64::from_le_bytes([
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                    ]) as usize;
                    let mut cases = Vec::new();
                    for _ in 0..cases_len {
                        let case_offset = i64::from_le_bytes([
                            *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                            *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                            *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                            *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                        ]);
                        cases.push(case_offset);
                    }
                    let default = match iter.next() {
                        Some(1) => {
                            let default_offset = i64::from_le_bytes([
                                *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                                *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                                *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                                *iter.next().ok_or("Expected u8 value")?, *iter.next().ok_or("Expected u8 value")?,
                            ]);
                            Some(default_offset)
                        },
                        Some(0) => None,
                        _ => return Err("Invalid Switch default case"),
                    };
                    result.push(Bytecode::Switch(cases, default));
                },
                _ => return Err("Invalid opcode"),
            }
        }

        Ok(result)
    }
    
    pub fn into_binary(&self) -> Vec<u8> {
        let mut result = Vec::new();
        match self {
            Bytecode::Nop => result.push(0),
            Bytecode::Breakpoint => result.push(1),
            Bytecode::LoadU8(value) => {
                result.push(2);
                result.extend_from_slice(&value.to_le_bytes());
            },
            Bytecode::LoadU16(value) => {
                result.push(3);
                result.extend_from_slice(&value.to_le_bytes());
            },
            Bytecode::LoadU32(value) => {
                result.push(4);
                result.extend_from_slice(&value.to_le_bytes());
            },
            Bytecode::LoadU64(value) => {
                result.push(5);
                result.extend_from_slice(&value.to_le_bytes());
            },
            Bytecode::LoadI8(value) => {
                result.push(6);
                result.extend_from_slice(&value.to_le_bytes());
            },
            Bytecode::LoadI16(value) => {
                result.push(7);
                result.extend_from_slice(&value.to_le_bytes());
            },
            Bytecode::LoadI32(value) => {
                result.push(8);
                result.extend_from_slice(&value.to_le_bytes());
            },
            Bytecode::LoadI64(value) => {
                result.push(9);
                result.extend_from_slice(&value.to_le_bytes());
            },
            Bytecode::LoadF32(value) => {
                result.push(10);
                result.extend_from_slice(&value.to_le_bytes());
            },
            Bytecode::LoadF64(value) => {
                result.push(11);
                result.extend_from_slice(&value.to_le_bytes());
            },
            Bytecode::LoadSymbol(index) => {
                result.push(12);
                result.extend_from_slice(&index.to_le_bytes());
            }
            Bytecode::Pop => result.push(13),
            Bytecode::Dup => result.push(14),
            Bytecode::Swap => result.push(15),
            Bytecode::StoreLocal(index) => {
                result.push(16);
                result.push(*index);
            },
            Bytecode::LoadLocal(index) => {
                result.push(17);
                result.push(*index);
            },
            Bytecode::StoreArgument(index) => {
                result.push(18);
                result.push(*index);
            },
            Bytecode::AddInt => result.push(19),
            Bytecode::SubInt => result.push(20),
            Bytecode::MulInt => result.push(21),
            Bytecode::DivSigned => result.push(22),
            Bytecode::DivUnsigned => result.push(23),
            Bytecode::ModSigned => result.push(24),
            Bytecode::ModUnsigned => result.push(25),
            Bytecode::AddFloat => result.push(26),
            Bytecode::SubFloat => result.push(27),
            Bytecode::MulFloat => result.push(28),
            Bytecode::DivFloat => result.push(29),
            Bytecode::ModFloat => result.push(30),
            Bytecode::SatAddIntUnsigned => result.push(31),
            Bytecode::SatSubIntUnsigned => result.push(32),
            Bytecode::And => result.push(33),
            Bytecode::Or => result.push(34),
            Bytecode::Xor => result.push(35),
            Bytecode::Not => result.push(36),
            Bytecode::Shl => result.push(37),
            Bytecode::AShr => result.push(38),
            Bytecode::LShr => result.push(39),
            Bytecode::Neg => result.push(40),
            Bytecode::EqualSigned => result.push(41),
            Bytecode::NotEqualSigned => result.push(42),
            Bytecode::EqualUnsigned => result.push(43),
            Bytecode::NotEqualUnsigned => result.push(44),
            Bytecode::GreaterSigned => result.push(45),
            Bytecode::LessSigned => result.push(46),
            Bytecode::GreaterOrEqualSigned => result.push(47),
            Bytecode::LessOrEqualSigned => result.push(48),
            Bytecode::GreaterUnsigned => result.push(49),
            Bytecode::LessUnsigned => result.push(50),
            Bytecode::GreaterOrEqualUnsigned => result.push(51),
            Bytecode::LessOrEqualUnsigned => result.push(52),
            Bytecode::EqualFloat => result.push(53),
            Bytecode::NotEqualFloat => result.push(54),
            Bytecode::GreaterFloat => result.push(55),
            Bytecode::LessFloat => result.push(56),
            Bytecode::GreaterOrEqualFloat => result.push(57),
            Bytecode::LessOrEqualFloat => result.push(58),
            Bytecode::Convert(tag) => {
                result.push(59);
                result.push(tag.as_byte());
            },
            Bytecode::BinaryConvert(tag) => {
                result.push(60);
                result.push(tag.as_byte());
            },
            Bytecode::CreateArray(tag) => {
                result.push(61);
                result.push(tag.as_byte());
            },
            Bytecode::ArrayGet(tag) => {
                result.push(62);
                result.push(tag.as_byte());
            },
            Bytecode::ArraySet(tag) => {
                result.push(63);
                result.push(tag.as_byte());
            },
            Bytecode::NewObject(index) => {
                result.push(64);
                result.extend_from_slice(&index.to_le_bytes());
            },
            Bytecode::GetField(class_name, parent_class_name, member_index, tag) => {
                result.push(65);
                result.extend_from_slice(&class_name.to_le_bytes());
                result.extend_from_slice(&parent_class_name.to_le_bytes());
                result.extend_from_slice(&member_index.to_le_bytes());
                result.push(tag.as_byte());
            },
            Bytecode::SetField(class_name, parent_class_name, member_index, tag) => {
                result.push(66);
                result.extend_from_slice(&class_name.to_le_bytes());
                result.extend_from_slice(&parent_class_name.to_le_bytes());
                result.extend_from_slice(&member_index.to_le_bytes());
                result.push(tag.as_byte());
            },
            Bytecode::IsA(class_name) => {
                result.push(67);
                result.extend_from_slice(&class_name.to_le_bytes());
            },
            Bytecode::InvokeVirt(class_name, method_name) => {
                result.push(68);
                result.extend_from_slice(&class_name.to_le_bytes());
                result.extend_from_slice(&method_name.to_le_bytes());
            },
            Bytecode::InvokeVirtTail(class_name, method_name) => {
                result.push(69);
                result.extend_from_slice(&class_name.to_le_bytes());
                result.extend_from_slice(&method_name.to_le_bytes());
            },
            Bytecode::InvokeStatic(class_name, method_name) => {
                result.push(70);
                result.extend_from_slice(&class_name.to_le_bytes());
                result.extend_from_slice(&method_name.to_le_bytes());
            },
            Bytecode::InvokeStaticTail(class_name, method_name) => {
                result.push(71);
                result.extend_from_slice(&class_name.to_le_bytes());
                result.extend_from_slice(&method_name.to_le_bytes());
            },
            Bytecode::GetStaticMethod(class_name, method_name) => {
                result.push(72);
                result.extend_from_slice(&class_name.to_le_bytes());
                result.extend_from_slice(&method_name.to_le_bytes());
            },
            Bytecode::GetStaticMember(class_name, index, tag) => {
                result.push(73);
                result.extend_from_slice(&class_name.to_le_bytes());
                result.extend_from_slice(&index.to_le_bytes());
                result.push(tag.as_byte());
            }
            Bytecode::SetStaticMember(class_name, index, tag) => {
                result.push(74);
                result.extend_from_slice(&class_name.to_le_bytes());
                result.extend_from_slice(&index.to_le_bytes());
                result.push(tag.as_byte());
            }
            Bytecode::GetStrRef(index) => {
                result.push(75);
                result.extend_from_slice(&index.to_le_bytes());
            },
            Bytecode::Return => result.push(76),
            Bytecode::ReturnVoid => result.push(77),
            Bytecode::RegisterException(index, block_id) => {
                result.push(78);
                result.extend_from_slice(&index.to_le_bytes());
                result.extend_from_slice(&block_id.to_le_bytes());
            }
            Bytecode::UnregisterException(index) => {
                result.push(79);
                result.extend_from_slice(&index.to_le_bytes());
            }
            Bytecode::Throw => result.push(80),
            Bytecode::StartBlock(id) => {
                result.push(81);
                result.extend_from_slice(&id.to_le_bytes());
            },
            Bytecode::Goto(offset) => {
                result.push(82);
                result.extend_from_slice(&offset.to_le_bytes());
            },
            Bytecode::If(true_offset, false_offset) => {
                result.push(83);
                result.extend_from_slice(&true_offset.to_le_bytes());
                result.extend_from_slice(&false_offset.to_le_bytes());
            },
            Bytecode::Switch(cases, default) => {
                result.push(84);
                result.extend_from_slice(&(cases.len() as u64).to_le_bytes());
                for case in cases {
                    result.extend_from_slice(&case.to_le_bytes());
                }
                match default {
                    Some(offset) => {
                        result.push(1);
                        result.extend_from_slice(&offset.to_le_bytes())
                    },
                    None => {
                        result.push(0);
                    }
                }
            },
        }

        result
    }
}





            
            
                                                
