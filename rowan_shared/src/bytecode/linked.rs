use crate::bytecode::compiled::StringIndex;
use crate::TypeTag;

pub type BlockIdOffset = i64;
pub type Symbol = u64;

#[derive(Clone, Debug)]
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
    /// Load Symbol
    LoadSymbol(Symbol),
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
    Add,
    /// Wrapping Subtraction
    Sub,
    /// Wrapping Multiplication
    Mul,
    /// Wrapping Division
    Div,
    /// Wrapping Modulus
    Mod,
    /// Saturating Addition
    SatAddSigned,
    /// Saturating Subtraction
    SatSubSigned,
    /// Saturating Addition
    SatAddUnsigned,
    /// Saturating Subtraction
    SatSubUnsigned,
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
    Equal,
    /// Not equal comparison
    NotEqual,
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
    /// The top value on the stack is the index, the second value is the array and the third value is the value to set
    ArraySet(TypeTag),
    /// Create a new object of the specified class
    NewObject(Symbol),
    /// Get a field from an object of the specified class
    /// The first Symbol is the class name we are accesssing, the second Symbol is another classname
    /// that allows for selecting the particular parent to access the field.
    GetField(Symbol, Symbol, u64),
    /// Set a field in an object of the specified class
    /// The first Symbol is the class name we are accesssing, the second Symbol is another classname
    /// that allows for selecting the particular parent to access the field.
    SetField(Symbol, Symbol, u64),
    /// Check if an object is of a specified class
    IsA(Symbol),
    /// Invoke a virtual method on an object of the specified class
    /// The StringIndices are class names. The two class names allow for calling super methods as well
    /// as overridden super methods
    /// The third Symbol is the method name
    InvokeVirt(Symbol, Option<Symbol>, Symbol),
    /// Invoke a virtual method on an object of the specified class
    /// The StringIndices are class names. The two class names allow for calling super methods as well
    /// as overridden super methods
    /// The third Symbol is the method name
    /// This is for tail recursion
    InvokeVirtTail(Symbol, Option<Symbol>, Symbol),
    /// Emit a signal from an object of the specified class
    /// The first Symbol is the class name and the second Symbol is the signal name
    EmitSignal(Symbol, Symbol),
    /// Emit a static signal from an object of the specified class
    /// The first Symbol is the class name and the second Symbol is the signal name
    EmitStaticSignal(Symbol, Symbol),
    /// Connect a signal from an object of the specified class to a method on another object of a specified class
    /// The top two stack values are used for this. The top object is connected to the bottom object's signal
    /// via the 2nd and 3rd Class Names + the Method Name
    /// The parameters are as follows:
    /// 1. The signal's name
    /// 2. The class name of the signal
    /// 3. The ancestor class name of the signal
    /// 4. The method's name
    ConnectSignal(Symbol, Symbol, Symbol, Symbol),
    /// Disconnect a signal from an object with the specified signal name and method name
    /// The parameters are as follows:
    /// 1. The signal's name
    /// 2. The method's name
    DisconnectSignal(Symbol, Symbol),
    /// Get a string reference from the string table
    /// These are like Rust's &'static str
    /// There isn't much to do with them other than pass them around to construct the String object
    GetStrRef(Symbol),
    /// Return from a function
    /// This pops the top value off the stack and returns it
    Return,
    /// Return from a function
    /// This pops nothing off the stack and returns void
    ReturnVoid,
    /// Catch an exception with the specified class symbol
    Catch(StringIndex),
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



/*impl Bytecode {
    fn try_from(value: impl IntoIterator<Item = u8, IntoIter = impl Iterator<Item = u8>>) -> Result<Vec<Bytecode>, &'static str> {
        let mut iter = value.into_iter();
        let mut result = Vec::new();
        while let Some(opcode) = iter.next() {
            match opcode {
                0 => result.push(Bytecode::Nop),
                1 => result.push(Bytecode::Breakpoint),
                2 => {
                    let value = iter.next().ok_or("Expected u8 value")?;
                    result.push(Bytecode::LoadU8(value));
                },
                3 => {
                    let value = u16::from_le_bytes([
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?
                    ]);
                    result.push(Bytecode::LoadU16(value));
                },
                4 => {
                    let value = u32::from_le_bytes([
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?
                    ]);
                    result.push(Bytecode::LoadU32(value));
                },
                5 => {
                    let value = u64::from_le_bytes([
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                    ]);
                    result.push(Bytecode::LoadU64(value));
                }
                6 => {
                    let value = i8::from_le_bytes([iter.next().ok_or("Expected u8 value")?]);
                    result.push(Bytecode::LoadI8(value));
                },
                7 => {
                    let value = i16::from_le_bytes([
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?
                    ]);
                    result.push(Bytecode::LoadI16(value));
                },
                8 => {
                    let value = i32::from_le_bytes([
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?
                    ]);
                    result.push(Bytecode::LoadI32(value));
                },
                9 => {
                    let value = i64::from_le_bytes([
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                    ]);
                    result.push(Bytecode::LoadI64(value));
                },
                10 => {
                    let value = f32::from_le_bytes([
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?
                    ]);
                    result.push(Bytecode::LoadF32(value));
                },
                11 => {
                    let value = f64::from_le_bytes([
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                    ]);
                    result.push(Bytecode::LoadF64(value));
                },
                12 => result.push(Bytecode::Pop),
                13 => result.push(Bytecode::Dup),
                14 => result.push(Bytecode::Swap),
                15 => {
                    let index = iter.next().ok_or("Expected u8 value")?;
                    result.push(Bytecode::StoreLocal(index));
                },
                16 => {
                    let index = iter.next().ok_or("Expected u8 value")?;
                    result.push(Bytecode::LoadLocal(index));
                },
                17 => {
                    let index = iter.next().ok_or("Expected u8 value")?;
                    result.push(Bytecode::StoreArgument(index));
                },
                18 => result.push(Bytecode::Add),
                19 => result.push(Bytecode::Sub),
                20 => result.push(Bytecode::Mul),
                21 => result.push(Bytecode::Div),
                22 => result.push(Bytecode::Mod),
                23 => result.push(Bytecode::SatAdd),
                24 => result.push(Bytecode::SatSub),
                25 => result.push(Bytecode::SatMul),
                26 => result.push(Bytecode::SatDiv),
                27 => result.push(Bytecode::SatMod),
                28 => result.push(Bytecode::And),
                29 => result.push(Bytecode::Or),
                30 => result.push(Bytecode::Xor),
                31 => result.push(Bytecode::Not),
                32 => result.push(Bytecode::AShl),
                33 => result.push(Bytecode::LShl),
                34 => result.push(Bytecode::AShr),
                35 => result.push(Bytecode::LShr),
                36 => result.push(Bytecode::Neg),
                37 => result.push(Bytecode::Equal),
                38 => result.push(Bytecode::NotEqual),
                39 => result.push(Bytecode::Greater),
                40 => result.push(Bytecode::Less),
                41 => result.push(Bytecode::GreaterOrEqual),
                42 => result.push(Bytecode::LessOrEqual),
                43 => {
                    let tag = TypeTag::from(iter.next().ok_or("Expected u8 value")?);
                    result.push(Bytecode::Convert(tag));
                },
                44 => {
                    let tag = TypeTag::from(iter.next().ok_or("Expected u8 value")?);
                    result.push(Bytecode::BinaryConvert(tag));
                },
                45 => {
                    let tag = TypeTag::from(iter.next().ok_or("Expected u8 value")?);
                    result.push(Bytecode::CreateArray(tag));
                },
                46 => {
                    let tag = TypeTag::from(iter.next().ok_or("Expected u8 value")?);
                    result.push(Bytecode::ArrayGet(tag));
                },
                47 => {
                    let tag = TypeTag::from(iter.next().ok_or("Expected u8 value")?);
                    result.push(Bytecode::ArraySet(tag));
                },
                48 => {
                    let index = u64::from_le_bytes([
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                    ]);
                    result.push(Bytecode::NewObject(index));
                },
                49 => {
                    let class_name = u64::from_le_bytes([
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                    ]);
                    let parent_class_name = u64::from_le_bytes([
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                    ]);
                    let member_index = u64::from_le_bytes([
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                    ]);
                    result.push(Bytecode::GetField(class_name, parent_class_name, member_index));
                },
                50 => {
                    let class_name = u64::from_le_bytes([
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                    ]);
                    let parent_class_name = u64::from_le_bytes([
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                    ]);
                    let member_index = u64::from_le_bytes([
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                    ]);
                    result.push(Bytecode::SetField(class_name, parent_class_name, member_index));
                },
                51 => {
                    let class_name = u64::from_le_bytes([
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                    ]);
                    result.push(Bytecode::IsA(class_name));
                },
                52 => {
                    let class_name = u64::from_le_bytes([
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                    ]);
                    let parent_class_name = u64::from_le_bytes([
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                    ]);
                    let method_name = u64::from_le_bytes([
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                    ]);
                    result.push(Bytecode::InvokeVirt(class_name, parent_class_name, method_name));
                },
                53 => {
                    let class_name = u64::from_le_bytes([
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                    ]);
                    let parent_class_name = u64::from_le_bytes([
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                    ]);
                    let method_name = u64::from_le_bytes([
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                    ]);
                    result.push(Bytecode::InvokeVirtTail(class_name, parent_class_name, method_name));
                },
                54 => {
                    let class_name = u64::from_le_bytes([
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                    ]);
                    let signal_name = u64::from_le_bytes([
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                    ]);
                    result.push(Bytecode::EmitSignal(class_name, signal_name));
                },
                55 => {
                    let class_name = u64::from_le_bytes([
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                    ]);
                    let signal_name = u64::from_le_bytes([
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                    ]);
                    result.push(Bytecode::EmitStaticSignal(class_name, signal_name));
                },
                56 => {
                    let signal_name = u64::from_le_bytes([
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                    ]);
                    let class_name = u64::from_le_bytes([
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                    ]);
                    let parent_class_name = u64::from_le_bytes([
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                    ]);
                    let method_name = u64::from_le_bytes([
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                    ]);
                    result.push(Bytecode::ConnectSignal(signal_name, class_name, parent_class_name, method_name));
                },
                57 => {
                    let signal_name = u64::from_le_bytes([
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                    ]);
                    let method_name = u64::from_le_bytes([
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                    ]);
                    result.push(Bytecode::DisconnectSignal(signal_name, method_name));
                }
                58 => {
                    let index = u64::from_le_bytes([
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                    ]);
                    result.push(Bytecode::GetStrRef(index));
                },
                59 => result.push(Bytecode::Return),
                60 => result.push(Bytecode::ReturnVoid),
                61 => {
                    let id = u64::from_le_bytes([
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                    ]);
                    result.push(Bytecode::StartBlock(id));
                },
                62 => {
                    let offset = i64::from_le_bytes([
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                    ]);
                    result.push(Bytecode::Goto(offset));
                },
                63 => {
                    let true_offset = i64::from_le_bytes([
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                    ]);
                    let false_offset = i64::from_le_bytes([
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                    ]);
                    result.push(Bytecode::If(true_offset, false_offset));
                },
                64 => {
                    let cases_len = u64::from_le_bytes([
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                    ]) as usize;
                    let mut cases = Vec::new();
                    for _ in 0..cases_len {
                        let case_offset = i64::from_le_bytes([
                            iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                            iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                            iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                            iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                        ]);
                        cases.push(case_offset);
                    }
                    let default = match iter.next() {
                        Some(1) => {
                            let default_offset = i64::from_le_bytes([
                                iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                                iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                                iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
                                iter.next().ok_or("Expected u8 value")?, iter.next().ok_or("Expected u8 value")?,
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
}

impl Into<Vec<u8>> for Bytecode {
    fn into(self) -> Vec<u8> {
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
            Bytecode::Pop => result.push(12),
            Bytecode::Dup => result.push(13),
            Bytecode::Swap => result.push(14),
            Bytecode::StoreLocal(index) => {
                result.push(15);
                result.push(index);
            },
            Bytecode::LoadLocal(index) => {
                result.push(16);
                result.push(index);
            },
            Bytecode::StoreArgument(index) => {
                result.push(17);
                result.push(index);
            },
            Bytecode::Add => result.push(18),
            Bytecode::Sub => result.push(19),
            Bytecode::Mul => result.push(20),
            Bytecode::Div => result.push(21),
            Bytecode::Mod => result.push(22),
            Bytecode::SatAdd => result.push(23),
            Bytecode::SatSub => result.push(24),
            Bytecode::SatMul => result.push(25),
            Bytecode::SatDiv => result.push(26),
            Bytecode::SatMod => result.push(27),
            Bytecode::And => result.push(28),
            Bytecode::Or => result.push(29),
            Bytecode::Xor => result.push(30),
            Bytecode::Not => result.push(31),
            Bytecode::AShl => result.push(32),
            Bytecode::LShl => result.push(33),
            Bytecode::AShr => result.push(34),
            Bytecode::LShr => result.push(35),
            Bytecode::Neg => result.push(36),
            Bytecode::Equal => result.push(37),
            Bytecode::NotEqual => result.push(38),
            Bytecode::Greater => result.push(39),
            Bytecode::Less => result.push(40),
            Bytecode::GreaterOrEqual => result.push(41),
            Bytecode::LessOrEqual => result.push(42),
            Bytecode::Convert(tag) => {
                result.push(43);
                result.push(tag.as_byte());
            },
            Bytecode::BinaryConvert(tag) => {
                result.push(44);
                result.push(tag.as_byte());
            },
            Bytecode::CreateArray(tag) => {
                result.push(45);
                result.push(tag.as_byte());
            },
            Bytecode::ArrayGet(tag) => {
                result.push(46);
                result.push(tag.as_byte());
            },
            Bytecode::ArraySet(tag) => {
                result.push(47);
                result.push(tag.as_byte());
            },
            Bytecode::NewObject(index) => {
                result.push(48);
                result.extend_from_slice(&index.to_le_bytes());
            },
            Bytecode::GetField(class_name, parent_class_name, member_index) => {
                result.push(49);
                result.extend_from_slice(&class_name.to_le_bytes());
                result.extend_from_slice(&parent_class_name.to_le_bytes());
                result.extend_from_slice(&member_index.to_le_bytes());
            },
            Bytecode::SetField(class_name, parent_class_name, member_index) => {
                result.push(50);
                result.extend_from_slice(&class_name.to_le_bytes());
                result.extend_from_slice(&parent_class_name.to_le_bytes());
                result.extend_from_slice(&member_index.to_le_bytes());
            },
            Bytecode::IsA(class_name) => {
                result.push(51);
                result.extend_from_slice(&class_name.to_le_bytes());
            },
            Bytecode::InvokeVirt(class_name, parent_class_name, method_name) => {
                result.push(52);
                result.extend_from_slice(&class_name.to_le_bytes());
                result.extend_from_slice(&parent_class_name.to_le_bytes());
                result.extend_from_slice(&method_name.to_le_bytes());
            },
            Bytecode::InvokeVirtTail(class_name, parent_class_name, method_name) => {
                result.push(53);
                result.extend_from_slice(&class_name.to_le_bytes());
                result.extend_from_slice(&parent_class_name.to_le_bytes());
                result.extend_from_slice(&method_name.to_le_bytes());
            },
            Bytecode::EmitSignal(class_name, signal_name) => {
                result.push(54);
                result.extend_from_slice(&class_name.to_le_bytes());
                result.extend_from_slice(&signal_name.to_le_bytes());
            },
            Bytecode::EmitStaticSignal(class_name, signal_name) => {
                result.push(55);
                result.extend_from_slice(&class_name.to_le_bytes());
                result.extend_from_slice(&signal_name.to_le_bytes());
            },
            Bytecode::ConnectSignal(class_name, parent_class_name, method_name, target_class_name) => {
                result.push(56);
                result.extend_from_slice(&class_name.to_le_bytes());
                result.extend_from_slice(&parent_class_name.to_le_bytes());
                result.extend_from_slice(&method_name.to_le_bytes());
                result.extend_from_slice(&target_class_name.to_le_bytes());
            },
            Bytecode::DisconnectSignal(signal_name, method_name) => {
                result.push(57);
                result.extend_from_slice(&signal_name.to_le_bytes());
                result.extend_from_slice(&method_name.to_le_bytes());
            },
            Bytecode::GetStrRef(index) => {
                result.push(58);
                result.extend_from_slice(&index.to_le_bytes());
            },
            Bytecode::Return => result.push(59),
            Bytecode::ReturnVoid => result.push(60),
            Bytecode::StartBlock(id) => {
                result.push(61);
                result.extend_from_slice(&id.to_le_bytes());
            },
            Bytecode::Goto(offset) => {
                result.push(62);
                result.extend_from_slice(&offset.to_le_bytes());
            },
            Bytecode::If(true_offset, false_offset) => {
                result.push(63);
                result.extend_from_slice(&true_offset.to_le_bytes());
                result.extend_from_slice(&false_offset.to_le_bytes());
            },
            Bytecode::Switch(cases, default) => {
                result.push(64);
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
}*/
