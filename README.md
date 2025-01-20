## Design
This language's design is borrowed from Godot. The main idea taken from Godot is that all objects are nodes on a graph. This means that you can change/add functionality by adding new nodes to other nodes.
To take advantage of this design, objects may arbitrarily add or remove object from themselves. Objects attached this way will be updated every heartbeat.

The heartbeat is a message that all objects attached to the main object will respond to via the `tick` method. This method takes a f64 of the amount of seconds elapsed since the last heartbeat. This allows for time dependent code to be processed, without the need of sleep.
Objects will also have a `ready` method that takes zero arguments that is called when an object is attached to to another object. The main's ready method is the first method called on VM startup.

There are 4 different ways of calling a method.
1. Normal Method calls
    * These are pretty much methods from Java and they are synchronous, meaning that calling a method blocks the current method until it is done.
2. Signals
    * These are like signals from Godot. They are asynchronous and call methods on objects that have been connected to to the object that emitted the signal.
3. Static Signals
    * These are a special type of signals. They are connected at link time/compile time and all of the objects that have methods that are listening for the signal will have their methods called.
4. Remote Proceedure calls
    * These are a work in progress and as such, don't have any documentation

### Design Implementation
There will be a max heap priority queue that dictates which messages will be handled first. Tick (the heartbeat) always comes first, and as such, will not be in the queue.
Messages are added to a lockless queue every heartbeat and then dumped into the priority queue during the processing of the tick message. The priority is dictated by a graph coloring algorithm to ensure thread safety.
The way it should work is that is will order the messages based off of availablity. Thread safety is preserved by ensuring that an object is only in one thread at a time. This means that the order of messages shouldn't matter.

There will be 2 lockless queues. This ensures that we can add messages while processing tick and message order. The queues should be of a sufficient size to handle the adding an unknown number of messages

#### Garbage Collection
We can exploit the fact that there is a pause time in the running of the virtual machine.
This means that all live memory only exists in either an object's children, one of an object's members, or in a message.
A garbage collection could occur after a certain amount of ticks and before message processing.
Since we have a threadpool, we can do a threaded divide and conquer algorithm to find all live memory.

#### Object Methods
These are the methods on the base object. All object types inherit from object.
| Method         | Description                                                                  | Signature           |
|:---------------|:-----------------------------------------------------------------------------|:-------------------:|
|`tick`          |Incremental processing method, called each heartbeat.                         |(f64) -> void        |
|`ready`         |Initialization method, called when an object is attached to a parent object.  |() -> void           |
|`upcast`        |Returns the object as a different type if it can be, otherwise raise an error.|\[T\]() -> T         |
|`try-upcast`    |Returns the object as a different type if it can be as an Option type.        |\[T\]() -> Option[T] |


## Structures
These all use Rust-like syntax

```rust
type Symbol = usize; // An index into a symbol table

type Reference = usize;
type Index = usize; // Represents and index into a table
type VTableIndex = usize; // Represents an index into a table of VTables

enum TypeTag {
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
    Char, //U32
    Object, // u64
    RawStr(Index), // Index into String Table
}

```


#### Class
##### VM Structure
```rust
struct Class {
    name: Symbol,
    parents: Vec<Symbol>, // Is mirrored in Object.parent_objects
    vtables: Map<(Symbol, Symbol), VTableIndex>, // Key is (Starting Parent, Particular Child), value is the vtable for that object. This allows for calling parent methods via super while still overloading parent vtables
    members: Vec<MemberInfo>,
    signals: Vec<SignalInfo>,
}

struct VTable {
    symbol_mapper: Map<Symbol, Index>,
    table: [VirtFunc]
}

struct VirtFunc {
    name: Symbol,
    value: VirtFuncValue,
    responds_to: Option<Symbol>,
    arguments: Vec<TypeTag>,
    ReturnType: TypeTag
}

enum VirtFuncValue {
    Builtin,
    Bytecode,
    Compiled,
}

struct MemberInfo {
    type_tag: TypeTag,
    name: Symbol,
}

struct SignalInfo {
    name: Symbol,
    static: bool,
    arguments: Vec<TypeTag>,
}

```

##### File Structure
```rust
type StringIndex = u64;
type BytecodeIndex = u64;


struct ClassFile {
    magic: u8,
    major_ver: u8,
    minor_ver: u8,
    patch_ver: u8,
    name: StringIndex,
    parent_count: u8,
    parent_names: [StringIndex; parent_count],
    vtable_size: u64,
    vtables: [VTable; vtable_size],
    members_size: u64,
    members_names: [StringIndex; members_size],
    signals_count: u64,
    signals: [Signal; signals_count],
    bytecode_table: [BytecodeEntry],
    string_table: [StringEntry]
}

struct VTable {
    size: u64,
    names_bytecode: [(StringIndex, BytecodeIndex); size],
}

struct BytecodeEntry {
    size: u64,
    code: [u8],
}

struct StringEntry {
    size: u64,
    data: [u8],
}

```


#### Object
```rust
struct Object {
    class: Symbol,
    parent_objects_size: usize,
    parent_objects: *mut [Reference], // Mirrors Class.parents
    children_size: usize,
    children: *mut[Reference],
    data: [u8], // Binary data
}
```

#### Bytecode
```rust

type TypeTag = u8;
type BlockId = usize;

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
    StoreLocal(u8)
    LoadLocal(u8)
    StoreArgument(u8)
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
    Return,
    ReturnVoid,
    StartBlock(usize),
    Goto(BlockId), // Offset to next block from current block
    If(BlockId, BlockId),
    Switch(Vec<BlockId>, BlockId),
    
}
    
```
