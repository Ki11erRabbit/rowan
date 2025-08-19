# TODO
Features that need to be implemented in base compiler or runtime before the text editor project
can be implemented.

## Text Editor Must haves
* [ ] Functional booleans in compiler
* [ ] Stability Improvements
  * [ ] In runtime, convert panics into exceptions thrown
  * [ ] Only panic when state is invalid.
* [ ] Compiler Error Reporting
  * [ ] Parse Errors
  * [ ] Typechecker Errors
* [ ] Generics
  * [x] Classes
  * [ ] Methods
* [ ] Throwing Exceptions
* [x] Module Paths
  * [x] Static method Paths
  * [x] Method Paths
  * [x] Static Field Paths
  * [x] Core paths
* [x] Native Method Interface
  * [x] Generate C Header File
  * [x] Generate Function that gives a function pointer for cleaning up native members
  * [x] Runtime Code that can handle the loading of dlls/so files during the link phase
* [x] Garbage Collection
  * [x] Garbage Collection doesn't collect too much 
  * [x] Garbage Collection is triggered when we are out of memory
  * [x] Garbage Collection is cross-platform
    * [x] Garbage Collection works on Unix
    * [x] Garbage Collection works on Windows
  * [x] Garbage collection doesn't collect static class fields
  * [x] There is a way to pass objects to FFI and mark that memory as uncollectable
  * [x] Garbage collection operation skips over native methods if they call Rowan code
* [x] Closure Expressions
  * [x] Captured Primitives (ints, floats) are boxed if they are mutated in closures
  * [x] Construction of Objects that are the closure
* [x] Safepoint markers in loops
  * [x] GC checks in loops
* [ ] loop expressions
  * [ ] continue
  * [ ] break can return from loop
* [ ] Unions
  * [ ] Generation into closed Class inheritance
  * [ ] More complicated Matching integration
  * [ ] Option Class
    * [ ] Option class is desugared into null pointer checks and erased
* [ ] Incremental Compilation
  * [ ] Detect if file is newer than output
  * [ ] Prevent generation if output already exists
* Basic Matching
  * [ ] Match on different object types
  * [ ] Match on Characters, ints, and bools
  * [ ] Match on strings
* [ ] STDLIB
  * [ ] Thread Safety
    * [ ] Mutex Class
    * [ ] RwLock Class
  * Base Functionality
    * [ ] Wrapper Classes for Primitives
    * [ ] Method attribute for dot notation on static methods
  * [ ] Collections
    * Unordered Structures 
      * [ ] HashMap
      * [ ] HashSet
    * [ ] List Collections
      * [ ] ArrayList
    * [ ] Threading API
    * [ ] Filesystem IO
    * [ ] Network IO

## Text Editor Nice to Haves
* [ ] Traits
  * [ ] Existential Types
  * [ ] Trait conversion into Universal Class Types
  * [ ] Generic Trait Constraints
  * [ ] Iterator Based For Loop
  * [ ] With Context Manager for RAII
* [ ] Try/Catch
  * [ ] Setting up exception handling
* [ ] break, continue, and labels on any loop
* [ ] Marker Traits for Thread Safety
* [ ] Static Method References
  * [ ] Generation of objects who just call static methods (very similar to closures) 

## Other Features Not needed for Text Editor but would be very nice
* [ ] partial method evaluation
* [ ] Operator Overloading Traits
* [ ] Higher Kinded Types
* [ ] Tuple Generation