pub mod test {
    use std::{collections::HashMap, env, mem, path::PathBuf};

    use crate::runtime::runtime_types::{Context, Instructions::*, Types::*, *};
    use libloading::Library;
    use runtime::runtime_error::ErrTypes;

    const ID: usize = 14;
    pub fn test_init(id: Option<usize>, context: &mut Context) -> bool {
        let test_id = if let Some(num) = id { num } else { ID };
        println!("Running test {test_id}");
        match test_id {
            0 => {
                context.memory.stack.data = vec![Int(0)];
                context.code.data = vec![End];
                true
            }
            // heap test
            1 => {
                context.memory.stack.data = vec![
                    Int(1),     // value to write
                    Null,       // pointer placeholder
                    Usize(5),   // size of object
                    Bool(true), // second value
                    Usize(3),   // position of second value in object
                    Usize(4),   // new size for realloc
                ];
                context.code.data = vec![
                    // stack
                    ReserveStack(3, 0),
                    // allocating size Usize(5)
                    Read(1, 0),
                    Allocate(0),
                    // writing pointer on stack
                    Write(2, 0),
                    // writing to pointer
                    Swap(0, POINTER_REG),
                    Read(3, 0), // value
                    WritePtr(0),
                    // writing to pointer[Usize(3)]
                    ReadConst(4, 0), // index
                    Index(0),
                    ReadConst(3, 0), // value
                    WritePtr(0),
                    // resizing to Usize(4)
                    ReadConst(5, 0),          // size
                    Read(2, POINTER_REG), // pointer
                    Reallocate(0),
                    // free
                    //Deallocate,
                    End,
                ];
                true
            }
            // function swap
            2 => {
                context.memory.stack.data = vec![
                    Int(3), // value 1
                    Int(7), // value 2
                    Bool(true),
                    Null,    // unused value
                    Int(0),  // index
                    Int(50), // max
                    Int(1),  // step
                ];
                context.code.data = vec![
                    ReserveStack(7, 0), // main stack
                    Goto(15),  // skip function declaration to the main code
                    // function swap stack[bool, (ptr, ptr), tmp] -> bool
                    // write tmp value of pointer1
                    Read(3, POINTER_REG),
                    ReadPtr(0),
                    Write(1, 0),
                    // write pointer2 to pointer1
                    Read(2, POINTER_REG),
                    ReadPtr(0), // value of pointer2
                    Read(3, POINTER_REG),
                    WritePtr(0),
                    // write tmp on pointer2
                    Read(1, 0),
                    Read(2, POINTER_REG),
                    WritePtr(0),
                    // Returnurn true
                    ReadConst(2, RETURN_REG),
                    Unfreeze,
                    Return,
                    // calling
                    Read(1 + 3, GENERAL_REG1),
                    ReserveStack(4, 0), // function args stack
                    Freeze,
                    Ptr(3 + 4 + 3),
                    Write(3, GENERAL_REG1),
                    Ptr(4 + 4 + 3),
                    Write(2, GENERAL_REG1),
                    Jump(2),
                    Read(3, GENERAL_REG1),
                    Read(1, GENERAL_REG2),
                    Add(GENERAL_REG1, GENERAL_REG2, GENERAL_REG1),
                    Write(3, GENERAL_REG1),
                    Read(2, GENERAL_REG2),
                    Less(GENERAL_REG1, GENERAL_REG2, GENERAL_REG1),
                    Branch(15, 30),
                    End,
                ];
                true
            }
            // function swap (optimized)
            3 => {
                context.memory.stack.data = vec![
                    Int(3),     // value 1
                    Int(7),     // value 2
                    Bool(true), // Returnurn value
                    Int(0),     // index
                    Int(50),    // max
                    Int(1),     // step
                ];
                context.code.data = vec![
                    ReserveStack(6, 0),
                    Goto(10),
                    // function swap registers[gen3: ptr, ptr:ptr]
                    ReadPtr(GENERAL_REG1), // load first value
                    // load second value
                    Swap(GENERAL_REG3, POINTER_REG),
                    ReadPtr(GENERAL_REG2),
                    WritePtr(GENERAL_REG1), // write first value
                    // write second value
                    Swap(GENERAL_REG3, POINTER_REG),
                    WritePtr(GENERAL_REG2),
                    ReadConst(2, RETURN_REG), // Returnurn value
                    Back,
                    // calling
                    Ptr(2 + 3),
                    Swap(GENERAL_REG1, GENERAL_REG3),
                    Ptr(3 + 3),
                    Swap(GENERAL_REG1, POINTER_REG),
                    Jump(2),
                    Read(3, GENERAL_REG1),
                    Read(1, GENERAL_REG2),
                    Add(GENERAL_REG1, GENERAL_REG2, GENERAL_REG1),
                    Write(3, GENERAL_REG1),
                    Read(2, GENERAL_REG2),
                    Less(GENERAL_REG1, GENERAL_REG2, GENERAL_REG1),
                    Branch(10, 22),
                    End,
                ];
                true
            }
            // memory goes brrrrrrrrr
            4 => {
                context.memory.stack.data = vec![
                    Pointer(1, PointerTypes::Object),
                    Usize(1), // size allocated on each iteration; low for safety measures
                    Int(0),   // index
                    Int(1),   // step
                    Int(300), // range
                    Null,     // placeholder for heap pointer
                ];
                context.code.data = vec![
                    ReserveStack(6, 1),
                    ReadConst(1, GENERAL_REG2), // size
                    Allocate(GENERAL_REG2),
                    Move(GENERAL_REG1, POINTER_REG),
                    Read(4, GENERAL_REG1),
                    Read(3, GENERAL_REG2),
                    Add(GENERAL_REG1, GENERAL_REG2, GENERAL_REG1),
                    Write(4, GENERAL_REG1),
                    Read(2, GENERAL_REG2),
                    Less(GENERAL_REG1, GENERAL_REG2, GENERAL_REG1),
                    ReserveStack(0, 0),
                    Branch(1, 12),
                    Debug(POINTER_REG),
                    ReadConst(1, GENERAL_REG2), // size
                    ReadConst(1, GENERAL_REG1), // size
                    SweepUnoptimized,
                    Allocate(GENERAL_REG2),
                    Sub(GENERAL_REG1, GENERAL_REG2, GENERAL_REG1),
                    Index(GENERAL_REG1),
                    WritePtr(GENERAL_REG2),
                    End,
                ];
                true
            }
            5 => {
                context.memory.stack.data = vec![Usize(1), Null, Int(70)];
                context.code.data = vec![
                    ReserveStack(3, 0),
                    Read(3, GENERAL_REG1),
                    Allocate(GENERAL_REG1),
                    Write(2, GENERAL_REG1),
                    Read(3, GENERAL_REG1),
                    Read(3, GENERAL_REG2),
                    Add(GENERAL_REG1, GENERAL_REG2, GENERAL_REG1),
                    Read(2, POINTER_REG),
                    Reallocate(GENERAL_REG1),
                    Index(GENERAL_REG2),
                    WritePtr(GENERAL_REG1),
                    Allocate(GENERAL_REG2),
                    Move(GENERAL_REG1, GENERAL_REG3),
                    Move(GENERAL_REG1, POINTER_REG),
                    Read(3, GENERAL_REG1),
                    Sub(GENERAL_REG1, GENERAL_REG2, GENERAL_REG1),
                    Index(GENERAL_REG1),
                    ReadPtr(GENERAL_REG1),
                    Debug(GENERAL_REG1),
                    Read(2, POINTER_REG),
                    //Deallocate,
                    Move(GENERAL_REG3, POINTER_REG),
                    Read(3, GENERAL_REG1),
                    Sub(GENERAL_REG1, GENERAL_REG2, GENERAL_REG1),
                    Index(GENERAL_REG1),
                    Read(1, GENERAL_REG1),
                    WritePtr(GENERAL_REG1),
                    End,
                ];
                true
            }
            // old version
            6 => {
                context.memory.stack.data = vec![Usize(1), Null, Int(70), Usize(0)];
                context.code.data = vec![
                    ReserveStack(3, 0),
                    ReadConst(0, GENERAL_REG1),
                    Allocate(GENERAL_REG1),
                    Write(2, POINTER_REG),
                    ReadConst(3, GENERAL_REG1),
                    Index(GENERAL_REG1),
                    Read(1, GENERAL_REG1),
                    WritePtr(GENERAL_REG1),
                    End,
                ];
                true
            }
            7 => {
                context.memory.strings.pool = vec![
                    "Hello world\n".chars().collect(),
                    "Length of h.w. string is: ".chars().collect(),
                    "gzjkh".chars().collect(),
                    "GC goes brrrrrrrrr".chars().collect(),
                    "Jeff Bezos".chars().collect(),
                    ", his height is: ".chars().collect(),
                ];
                context.memory.non_primitives = vec![
                    // struct Person, 3 fields, name, age, height, id = 0
                    runtime::runtime_types::NonPrimitiveType {
                        name: "Person".to_string(),
                        kind: NonPrimitiveTypes::Struct,
                        // name, age, height (includes header)
                        len: 4,
                        pointers: 1,
                        methods: HashMap::new(),
                    },
                ];
                context.memory.heap.data = vec![
                    // struct Person, name = "Jeff Bezos", age = 20, height = 180
                    vec![
                        Types::NonPrimitive(0),
                        Types::Pointer(4, PointerTypes::String),
                        Types::Int(20),
                        Types::Int(180),
                    ],
                ];
                context.memory.stack.data = vec![
                    Types::Pointer(0, PointerTypes::String),
                    Types::Pointer(1, PointerTypes::String),
                    Types::Pointer(3, PointerTypes::String),
                    // pointer to struct Person
                    Types::Pointer(0, PointerTypes::Object),
                    Types::Pointer(5, PointerTypes::String),
                ];
                context.code.data = vec![
                    ReadConst(0, GENERAL_REG1),
                    //StdOut(GENERAL_REG1),
                    ReadConst(1, GENERAL_REG1),
                    //StdOut(GENERAL_REG1),
                    ReadConst(0, GENERAL_REG1),
                    Move(GENERAL_REG1, POINTER_REG),
                    Len(GENERAL_REG1),
                    Debug(GENERAL_REG1),
                    ReadConst(2, POINTER_REG),
                    // pointer to struct Person
                    ReadConst(3, POINTER_REG),
                    // use IndexStatic to get name
                    IndexStatic(1),
                    ReadPtr(POINTER_REG),
                    // concat with ", his height is: "
                    ReadConst(4, GENERAL_REG1),
                    //StrCat(GENERAL_REG1),
                    // store in general reg 3 for later use
                    Move(POINTER_REG, GENERAL_REG3),
                    // use IndexStatic to get height
                    // first get pointer to struct Person
                    ReadConst(3, POINTER_REG),
                    IndexStatic(3),
                    ReadPtr(GENERAL_REG1),
                    // convert to string
                    IntoStr(GENERAL_REG1),
                    // swap with concatenated string
                    Swap(GENERAL_REG3, POINTER_REG),
                    //StrCat(GENERAL_REG3),
                    // print
                    //StdOut(POINTER_REG),
                    End,
                ];
                true
            }
            // test for trait system
            // old verison
            8 => {
                // trait 0
                // implements methods
                // 0: drive (takes self, Returnurns nothing)
                // 1: stop (takes self, Returnurns int)

                context.memory.non_primitives = vec![
                    // struct car, 3 fields, brand name, is for sports, speed, id = 0
                    runtime::runtime_types::NonPrimitiveType {
                        name: "Car".to_string(),
                        kind: NonPrimitiveTypes::Struct,
                        // brand name, is for sports, speed (includes header)
                        len: 4,
                        // brand name
                        pointers: 1,
                        methods: HashMap::from_iter(vec![(0, vec![9, 19])]),
                    },
                    // struct motorcycle, 3 fields, brand name, model, speed, id = 1
                    runtime::runtime_types::NonPrimitiveType {
                        name: "Motorcycle".to_string(),
                        kind: NonPrimitiveTypes::Struct,
                        // brand name, model, speed (includes header)
                        len: 4,
                        // brand name, model
                        pointers: 2,
                        methods: HashMap::new(),
                    },
                ];
                context.memory.fun_table = vec![
                    // random thing just to test if it works
                    FunSpec {
                        name: "todo!()".to_string(),
                        params: vec![],
                        stack_size: Some((13, 5)),
                        loc: 55,
                    },
                    // drive
                    FunSpec {
                        name: "drive".to_string(),
                        params: vec![],
                        stack_size: Some((13, 5)),
                        loc: 56,
                    },
                ];
                context.memory.strings.pool = vec![
                    "I am driving with ".chars().collect(),
                    "I am stopping with ".chars().collect(),
                    "BMW".chars().collect(),
                    "Yamaha".chars().collect(),
                    "R1".chars().collect(),
                    " at ".chars().collect(),
                    " km/h".chars().collect(),
                ];
                context.memory.stack.data = vec![
                    // create a car
                    Types::NonPrimitive(0),
                    Types::Pointer(2, PointerTypes::String),
                    Types::Bool(true),
                    Types::Int(200),
                    // create a motorcycle
                    Types::NonPrimitive(1),
                    Types::Pointer(3, PointerTypes::String),
                    Types::Pointer(4, PointerTypes::String),
                    Types::Int(300),
                    // initialize needed variables
                    Types::Pointer(0, PointerTypes::Stack), // pointer to car
                    Types::Pointer(0, PointerTypes::String), // string "I am driving with"
                    Types::Null,
                    Types::Pointer(5, PointerTypes::String), // string " at "
                    Types::Pointer(6, PointerTypes::String), // string " km/h"
                ];
                context.code.data = vec![
                    // allocate memory on stack for every initialized variable
                    // this marks the entry point of the program
                    ReserveStack(10, 0),
                    // first get pointer to car
                    Ptr(10),
                    // then get car struct from stack
                    Read(10, GENERAL_REG2),
                    // call drive
                    // reserve stack space for arguments
                    ReserveStack(1, 0),
                    // first argument is self
                    // note: values are pushed in reverse order and indexing starts from 1
                    Write(1, GENERAL_REG1),
                    DynMethod(GENERAL_REG2, 0, 0),
                    // Returnurn registers to their original values
                    Unfreeze,
                    SweepUnoptimized,
                    End,
                    // method drive for car
                    // prints "I am driving with BMW at 200 km/h"
                    // methods have 1 argument, self
                    // method Returnurn if it is for sports
                    // so we have to read it from the stack using Read(stack_offset + 1, reg)
                    // rest of the methods will remain undeclared because they are take too long to write for human
                    Read(1, POINTER_REG),
                    // get brand name
                    IndexStatic(1),
                    ReadPtr(GENERAL_REG1),
                    // get speed
                    // first get pointer to struct Car
                    Read(1, POINTER_REG),
                    IndexStatic(3),
                    ReadPtr(GENERAL_REG2),
                    // convert to string
                    IntoStr(GENERAL_REG2),
                    // what do we have now?
                    // GENERAL_REG1 = pointer to brand name
                    // GENERAL_REG2 = pointer to speed
                    // POINTER_REG = speed string
                    // move speed string to GENERAL_REG2
                    Move(POINTER_REG, GENERAL_REG2),
                    // cocnatenate what we have so far so we save space in registers
                    // get pointer to "I am driving with"
                    ReadConst(9, POINTER_REG),
                    //StrCat(GENERAL_REG1),
                    // concatenate with " at "
                    ReadConst(11, GENERAL_REG1),
                    //StrCat(GENERAL_REG1),
                    // concatenate with speed
                    //StrCat(GENERAL_REG2),
                    // concatenate with " km/h"
                    ReadConst(12, GENERAL_REG1),
                    //StrCat(GENERAL_REG1),
                    //StdOut(POINTER_REG),
                    // load Returnurn value into Returnurn register
                    Read(1, POINTER_REG),
                    IndexStatic(2),
                    ReadPtr(RETURN_REG),
                    Return,
                    // method stop for car
                    ReadConst(1, GENERAL_REG1),
                    //StdOut(GENERAL_REG1),
                    Return,
                ];
                true
            }
            9 => {

                context.memory.strings.pool = vec![
                    "Write something: ".chars().collect(),
                    "You wrote: ".chars().collect(),
                    "hello file".chars().collect(),
                    "bye file".chars().collect(),
                ];
                context.memory.stack.data = vec![
                    Types::Pointer(0, PointerTypes::String),
                    Types::Pointer(1, PointerTypes::String),
                    Types::Pointer(2, PointerTypes::String),
                    Types::Pointer(3, PointerTypes::String),
                ];
                context.code.data = vec![
                    ReadConst(0, POINTER_REG),
                    Cal(0, 1),
                    Cal(0, 2),
                    Move(RETURN_REG, GENERAL_REG1),
                    // print it back
                    ReadConst(1, POINTER_REG),
                    Cal(0, 0),
                    Swap(GENERAL_REG1, POINTER_REG),
                    Cal(0, 0),
                    // append to file
                    Move(RETURN_REG, GENERAL_REG1),
                    ReadConst(3, POINTER_REG),
                    Cal(0, 5),
                    // load file
                    ReadConst(2, POINTER_REG),
                    Cal(0, 3),
                    // print file contents
                    Move(RETURN_REG, POINTER_REG),
                    Cal(0, 1),
                    SweepUnoptimized,
                    End,
                ];
                true
            }
            10 => {
                context.memory.heap.data =
                    vec![[Types::Usize(656645), Types::Usize(656645)].to_vec()];
                context.memory.strings.pool = vec![
                    "Write something: ".chars().collect(),
                    "You wrote: ".chars().collect(),
                    "hello file".chars().collect(),
                ];
                context.memory.stack.data = vec![
                    Types::Null,     // args array
                    Types::Usize(1), // idx
                    Types::Usize(1), // step
                    Types::Null,     // len
                ];
                context.code.data = vec![
                    ReserveStack(4, 0),
                    // get args
                    Cal(0, 4),
                    Write(4, RETURN_REG),
                    Move(RETURN_REG, GENERAL_REG1),
                    Len(RETURN_REG),
                    Write(1, RETURN_REG),
                    // loop starts here
                    // get idx
                    Read(3, GENERAL_REG1),
                    // get len
                    Read(1, GENERAL_REG2),
                    // compare
                    Less(GENERAL_REG1, GENERAL_REG2, GENERAL_REG1),
                    Branch(10 /* another round */, 20 /* end of loop */),
                    // loop body
                    // get idx
                    Read(3, GENERAL_REG1),
                    // get arg
                    Read(4, POINTER_REG),
                    Index(GENERAL_REG1),
                    ReadPtr(POINTER_REG),
                    // print arg
                    Cal(0, 1),
                    // increment idx
                    Read(3, GENERAL_REG1),
                    Read(2, GENERAL_REG2),
                    Add(GENERAL_REG1, GENERAL_REG2, GENERAL_REG1),
                    Write(3, GENERAL_REG1),
                    // loop ends here
                    Goto(6),
                    End,
                ];
                /* equivalent C code
                int main(int argc, char** argv) {
                    // here we start from 0 because in our VM index(0) is for header so we have to skip it
                    // but this is not the case in C nor Ruda
                    for (int i = 0; i < argc; i++) {
                        printf("%s\n", argv[i]);
                    }
                }
                */
                true
            }
            11 => {
                context.memory.stack.data = vec![
                    Int(50),
                    Int(50),
                    Int(50),
                    Int(50),
                    Int(50),
                    Int(50),
                    Int(50),
                    Int(50),
                    Int(50),
                    Int(50),
                ];
                context.code.data = vec![];
                true
            }
            12 => {
                context.memory.stack.data = vec![
                    Int(50),
                    Pointer(2, PointerTypes::Stack),
                    NonPrimitive(3),
                    Null,
                ];
                context.memory.heap.data = vec![

                ];
                context.code.data = vec![
                    ReadConst(0, GENERAL_REG1),
                    ReadConst(1, POINTER_REG),
                    Cal(3, 0),
                    Cal(3, 1),
                    End, 
                ];
                true
            }
            // filesystem test
            13 => {
                context.memory.strings.pool = vec![
                    "./file.txt".to_string(),
                    "Hello file!".to_string(),
                ];
                context.memory.stack.data = vec![
                    Pointer(0, PointerTypes::String),
                    Pointer(1, PointerTypes::String),
                ];
                context.memory.heap.data = vec![

                ];
                context.code.data = vec![
                    // filename
                    ReadConst(0, POINTER_REG),
                    // file handle
                    Cal(2, 3),
                    Move(RETURN_REG, POINTER_REG),
                    Move(RETURN_REG, GENERAL_REG2), // save file handle
                        // read from file
                        Cal(2, 5),
                        // print it
                        Move(RETURN_REG, POINTER_REG),
                        Cal(0, 1),
                    // write to file
                    Move(GENERAL_REG2, POINTER_REG),
                    ReadConst(1, GENERAL_REG1),
                    Cal(2, 6),
                    End,
                ];
                true
            }
            14 => {
                context.memory.stack.data = vec![
                    Int(0),
                    Int(1),
                    Int(100000000),
                ];
                context.code.data = vec![
                    ReadConst(0, MEMORY_REG1),
                    ReadConst(1, GENERAL_REG2),
                    ReadConst(2, GENERAL_REG3),
                    Less(MEMORY_REG1, GENERAL_REG3, GENERAL_REG1),
                    Branch(5, 7),
                    Add(MEMORY_REG1, GENERAL_REG2, MEMORY_REG1),
                    Goto(3),
                    End,
                ];
                true
            }
            _ => {
                context.memory.stack.data = vec![Int(0)];
                context.code.data = vec![End];
                println!("Test id: {test_id} not found.");
                true
            }
        }
    }
    pub fn load_lib(path: &PathBuf, id: usize) -> fn(ctx: &mut Context, id: usize) -> Result<Types, ErrTypes> {
        let lib = unsafe { Library::new(path).unwrap() };
        let init_fn: libloading::Symbol<fn(&(), usize) -> fn(ctx: &mut Context, id: usize) -> Result<Types, ErrTypes>> =
            unsafe { lib.get(b"init").unwrap() };
        let lib_box = init_fn(&(), id);

        mem::forget(lib);
        lib_box
    }
    pub fn load_libs(libs: Vec<&str>) -> Vec<fn(ctx: &mut Context, id: usize) -> Result<Types, ErrTypes>> {
        let mut result = vec![];

        for lib_path in libs.iter().enumerate() {
            let lib = unsafe { Library::new(std_path(lib_path.1)).unwrap() };
            let init_fn: libloading::Symbol<fn(a: &(), id: usize) -> fn(ctx: &mut Context, id: usize) -> Result<Types, ErrTypes>> =
                unsafe { lib.get(b"init").unwrap() };
            let lib_box = init_fn(&(), lib_path.0);

            result.push(lib_box);
            mem::forget(lib);
        }

        drop(libs);
        result
    }
    // Returnurns path to standard library
    pub fn std_path(lib: &str) -> String {
        let mut std = env::var("RUDA_PATH").expect("RUDA_PATH not set, please set it to the path of the Ruda directory");
        
        #[cfg(windows)]
        {
            std.push_str("\\stdlib\\{name}.dll");
        }
        
        #[cfg(not(windows))]
        {
            std.push_str("stdlib/{name}.so");
        }
        
        let lib_path = std.replace("{name}", lib);
        lib_path
    }
}
