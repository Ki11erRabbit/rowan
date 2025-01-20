pub mod compiled;
pub mod linked;
use crate::TypeTag;

type BlockId = u64;

enum Bytecode {
    Nop,
    Breakpoint,
    LoadU8(u8),
    LoadU16(u16),
    LoadU32(u32),
    LoadU64(u64),
    LoadI8(i8),
    LoadI16(i16),
    LoadI32(i32),
    LoadI64(i64),
    LoadF32(f32),
    LoadF64(f64),
    Pop,
    Dup,
    Swap,
    StoreLocal(u8),
    LoadLocal(u8),
    StoreArgument(u8),
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    SatAdd,
    SatSub,
    SatMul,
    SatDiv,
    SatMod,
    And,
    Or,
    Xor,
    Not,
    AShl,
    LShl,
    AShr,
    LShr,
    Neg,
    Equal,
    NotEqual,
    Greater,
    Less,
    GreaterOrEqual,
    LessOrEqual,
    Convert(TypeTag),
    BinaryConvert(TypeTag),
    CreateArray(TypeTag),
    ArrayGet(TypeTag),
    ArraySet(TypeTag),
    NewObject(Symbol),
    GetField(Symbol, Symbol, usize), // Class name, Class name, Member. The second Class name is to allow for selecting the particular parent to access the field.
    SetField(Symbol, Symbol, usize), // Class name, Class name, Member. The second Class name is to allow for selecting the particular parent to access the field.
    IsA(Symbol),
    InvokeVirt(Symbol, Symbol, Symbol), // Class Name, Class Name, Function Name. The two class names allow for calling super methods as well as overridden super methods
    InvokeVirtTail(Symbol, Symbol, Symbol), // Class Name, Class Name, Function Name. The two class names allow for calling super methods as well as overridden super methods
    EmitSignal(Symbol, Symbol), // Class Name, Signal Name
    EmitStaticSignal(Symbol, Symbol), // Class Name, Signal Name
    ConnectSignal(Symbol, Symbol, Symbol, Symbol), // Signal Name, Class Name, Class Name, Method Name. The top two stack values are used for this. The top object is connected to the bottom object's signal via the 2nd and 3rd Class Names + the Method Name
    GetStrRef(Symbol),
    Return,
    ReturnVoid,
    StartBlock(usize),
    Goto(BlockId), // Offset to next block from current block
    If(BlockId, BlockId),
    Switch(Vec<BlockId>, BlockId),
}
