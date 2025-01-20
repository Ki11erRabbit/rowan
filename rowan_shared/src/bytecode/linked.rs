use crate::TypeTag;

pub type BlockIdOffset = i64;
pub type Symbol = usize;


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
    SatAdd,
    /// Saturating Subtraction
    SatSub,
    /// Saturating Multiplication
    SatMul,
    /// Saturating Division
    SatDiv,
    /// Saturating Modulus
    SatMod,
    // Bitwise operations
    /// Bitwise AND
    And,
    /// Bitwise OR
    Or,
    /// Bitwise XOR
    Xor,
    /// Bitwise NOT
    Not,
    /// Arithmetic Shift Left
    AShl,
    /// Logical Shift Left
    LShl,
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
    Greater,
    /// Less than comparison
    Less,
    /// Greater than or equal comparison
    GreaterOrEqual,
    /// Less than or equal comparison
    LessOrEqual,
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
    InvokeVirt(Symbol, Symbol, Symbol),
    /// Invoke a virtual method on an object of the specified class
    /// The StringIndices are class names. The two class names allow for calling super methods as well
    /// as overridden super methods
    /// The third Symbol is the method name
    /// This is for tail recursion
    InvokeVirtTail(Symbol, Symbol, Symbol),
    /// Emit a signal from an object of the specified class
    /// The first Symbol is the class name and the second Symbol is the signal name
    EmitSignal(Symbol, Symbol),
    /// Emit a static signal from an object of the specified class
    /// The first Symbol is the class name and the second Symbol is the signal name
    EmitStaticSignal(Symbol, Symbol),
    /// Connect a signal from an object of the specified class to a method on another object of a specified class
    /// The top two stack values are used for this. The top object is connected to the bottom object's signal
    /// via the 2nd and 3rd Class Names + the Method Name
    ConnectSignal(Symbol, Symbol, Symbol, Symbol),
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
