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
    NewObject(Symbol),
    /// Get a field from an object of the specified class
    /// The first Symbol is the class name we are accessing, the second Symbol is another classname
    /// that allows for selecting the particular parent to access the field.
    GetField(Symbol, Symbol, u64, TypeTag),
    /// Set a field in an object of the specified class
    /// The first Symbol is the class name we are accessing, the second Symbol is another classname
    /// that allows for selecting the particular parent to access the field.
    SetField(Symbol, Symbol, u64, TypeTag),
    /// Check if an object is of a specified class
    IsA(Symbol),
    /// Invoke a virtual method on an object of the specified class
    /// The Symbols are class names. The two class names allow for calling super methods as well
    /// as overridden super methods
    /// The third Symbol is the method name
    InvokeVirt(Symbol, Option<Symbol>, Symbol),
    /// Invoke a virtual method on an object of the specified class
    /// The Symbols are class names. The two class names allow for calling super methods as well
    /// as overridden super methods
    /// The third Symbol is the method name
    /// This is for tail recursion
    InvokeVirtTail(Symbol, Option<Symbol>, Symbol),
    /// Invoke a static method from a class
    /// The first Symbol is the class name
    /// The second Symbol is the Method Name
    InvokeStatic(Symbol, Symbol),
    /// Invoke a static method from a class
    /// The first Symbol is the class name
    /// The second Symbol is the Method Name
    /// This is for tail recursion
    InvokeStaticTail(Symbol, Symbol),
    /// Get a static method from a class and construct an object with the method `call`
    /// The first Symbol is the class name
    /// The second Symbol is the Method Name
    GetStaticMethod(Symbol, Symbol),
    /// Access a static field on a class and get its value
    /// The Symbol is the class
    /// The u64 is the index of the static member
    /// The TypeTag is the type of the variable
    GetStaticMember(Symbol, u64, TypeTag),
    /// Access a static field on a class and update its value
    /// The Symbol is the class
    /// The u64 is the index of the static member
    /// The TypeTag is the type of the variable
    SetStaticMember(Symbol, u64, TypeTag),
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
    /// Register a classname as a catchable exception
    /// Symbol is the classname
    /// BlockIdOffset is the offset to the handler block
    RegisterException(Symbol, BlockIdOffset),
    /// Unregister a classname as a catchable exception
    UnregisterException(Symbol),
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

