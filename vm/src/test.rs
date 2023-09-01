pub mod test {
    use std::{collections::HashMap, env, mem, path::PathBuf};

    use crate::runtime::runtime_types::{Context, Instructions::*, Types::*, *};
    use libloading::Library;

    const ID: usize = 10;
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
                    Res(3, 0),
                    // allocating size Usize(5)
                    Rd(1, 0),
                    Alc(0),
                    // writing pointer on stack
                    Wr(2, 0),
                    // writing to pointer
                    Swap(0, POINTER_REG),
                    Rd(3, 0), // value
                    Wrp(0),
                    // writing to pointer[Usize(3)]
                    Rdc(4, 0), // index
                    Idx(0),
                    Rdc(3, 0), // value
                    Wrp(0),
                    // resizing to Usize(4)
                    Rdc(5, 0),          // size
                    Rd(2, POINTER_REG), // pointer
                    RAlc(0),
                    // free
                    //Dalc,
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
                    Res(7, 0), // main stack
                    Goto(15),  // skip function declaration to the main code
                    // function swap stack[bool, (ptr, ptr), tmp] -> bool
                    // write tmp value of pointer1
                    Rd(3, POINTER_REG),
                    Rdp(0),
                    Wr(1, 0),
                    // write pointer2 to pointer1
                    Rd(2, POINTER_REG),
                    Rdp(0), // value of pointer2
                    Rd(3, POINTER_REG),
                    Wrp(0),
                    // write tmp on pointer2
                    Rd(1, 0),
                    Rd(2, POINTER_REG),
                    Wrp(0),
                    // return true
                    Rdc(2, RETURN_REG),
                    Ufrz,
                    Ret,
                    // calling
                    Rd(1 + 3, GENERAL_REG1),
                    Res(4, 0), // function args stack
                    Frz,
                    Ptr(3 + 4 + 3),
                    Wr(3, GENERAL_REG1),
                    Ptr(4 + 4 + 3),
                    Wr(2, GENERAL_REG1),
                    Jump(2),
                    Rd(3, GENERAL_REG1),
                    Rd(1, GENERAL_REG2),
                    Add(GENERAL_REG1, GENERAL_REG2, GENERAL_REG1),
                    Wr(3, GENERAL_REG1),
                    Rd(2, GENERAL_REG2),
                    Less(GENERAL_REG1, GENERAL_REG2, GENERAL_REG1),
                    Brnc(15, 30),
                    End,
                ];
                true
            }
            // function swap (optimized)
            3 => {
                context.memory.stack.data = vec![
                    Int(3),     // value 1
                    Int(7),     // value 2
                    Bool(true), // return value
                    Int(0),     // index
                    Int(50),    // max
                    Int(1),     // step
                ];
                context.code.data = vec![
                    Res(6, 0),
                    Goto(10),
                    // function swap registers[gen3: ptr, ptr:ptr]
                    Rdp(GENERAL_REG1), // load first value
                    // load second value
                    Swap(GENERAL_REG3, POINTER_REG),
                    Rdp(GENERAL_REG2),
                    Wrp(GENERAL_REG1), // write first value
                    // write second value
                    Swap(GENERAL_REG3, POINTER_REG),
                    Wrp(GENERAL_REG2),
                    Rdc(2, RETURN_REG), // return value
                    Back,
                    // calling
                    Ptr(2 + 3),
                    Swap(GENERAL_REG1, GENERAL_REG3),
                    Ptr(3 + 3),
                    Swap(GENERAL_REG1, POINTER_REG),
                    Jump(2),
                    Rd(3, GENERAL_REG1),
                    Rd(1, GENERAL_REG2),
                    Add(GENERAL_REG1, GENERAL_REG2, GENERAL_REG1),
                    Wr(3, GENERAL_REG1),
                    Rd(2, GENERAL_REG2),
                    Less(GENERAL_REG1, GENERAL_REG2, GENERAL_REG1),
                    Brnc(10, 22),
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
                    Res(6, 1),
                    Rdc(1, GENERAL_REG2), // size
                    Alc(GENERAL_REG2),
                    Move(GENERAL_REG1, POINTER_REG),
                    Rd(4, GENERAL_REG1),
                    Rd(3, GENERAL_REG2),
                    Add(GENERAL_REG1, GENERAL_REG2, GENERAL_REG1),
                    Wr(4, GENERAL_REG1),
                    Rd(2, GENERAL_REG2),
                    Less(GENERAL_REG1, GENERAL_REG2, GENERAL_REG1),
                    Res(0, 0),
                    Brnc(1, 12),
                    Debug(POINTER_REG),
                    Rdc(1, GENERAL_REG2), // size
                    Rdc(1, GENERAL_REG1), // size
                    SweepUnoptimized,
                    Alc(GENERAL_REG2),
                    Sub(GENERAL_REG1, GENERAL_REG2, GENERAL_REG1),
                    Idx(GENERAL_REG1),
                    Wrp(GENERAL_REG2),
                    End,
                ];
                true
            }
            5 => {
                context.memory.stack.data = vec![Usize(1), Null, Int(70)];
                context.code.data = vec![
                    Res(3, 0),
                    Rd(3, GENERAL_REG1),
                    Alc(GENERAL_REG1),
                    Wr(2, GENERAL_REG1),
                    Rd(3, GENERAL_REG1),
                    Rd(3, GENERAL_REG2),
                    Add(GENERAL_REG1, GENERAL_REG2, GENERAL_REG1),
                    Rd(2, POINTER_REG),
                    RAlc(GENERAL_REG1),
                    Idx(GENERAL_REG2),
                    Wrp(GENERAL_REG1),
                    Alc(GENERAL_REG2),
                    Move(GENERAL_REG1, GENERAL_REG3),
                    Move(GENERAL_REG1, POINTER_REG),
                    Rd(3, GENERAL_REG1),
                    Sub(GENERAL_REG1, GENERAL_REG2, GENERAL_REG1),
                    Idx(GENERAL_REG1),
                    Rdp(GENERAL_REG1),
                    Debug(GENERAL_REG1),
                    Rd(2, POINTER_REG),
                    //Dalc,
                    Move(GENERAL_REG3, POINTER_REG),
                    Rd(3, GENERAL_REG1),
                    Sub(GENERAL_REG1, GENERAL_REG2, GENERAL_REG1),
                    Idx(GENERAL_REG1),
                    Rd(1, GENERAL_REG1),
                    Wrp(GENERAL_REG1),
                    End,
                ];
                true
            }
            // old version
            6 => {
                context.memory.stack.data = vec![Usize(1), Null, Int(70), Usize(0)];
                context.code.data = vec![
                    Res(3, 0),
                    Rdc(0, GENERAL_REG1),
                    Alc(GENERAL_REG1),
                    Wr(2, POINTER_REG),
                    Rdc(3, GENERAL_REG1),
                    Idx(GENERAL_REG1),
                    Rd(1, GENERAL_REG1),
                    Wrp(GENERAL_REG1),
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
                    NonPrimitiveType {
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
                    Rdc(0, GENERAL_REG1),
                    //StdOut(GENERAL_REG1),
                    Rdc(1, GENERAL_REG1),
                    //StdOut(GENERAL_REG1),
                    Rdc(0, GENERAL_REG1),
                    Move(GENERAL_REG1, POINTER_REG),
                    Len(GENERAL_REG1),
                    Debug(GENERAL_REG1),
                    Rdc(2, POINTER_REG),
                    // pointer to struct Person
                    Rdc(3, POINTER_REG),
                    // use idxk to get name
                    IdxK(1),
                    Rdp(POINTER_REG),
                    // concat with ", his height is: "
                    Rdc(4, GENERAL_REG1),
                    //StrCat(GENERAL_REG1),
                    // store in general reg 3 for later use
                    Move(POINTER_REG, GENERAL_REG3),
                    // use idxk to get height
                    // first get pointer to struct Person
                    Rdc(3, POINTER_REG),
                    IdxK(3),
                    Rdp(GENERAL_REG1),
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
                // 0: drive (takes self, returns nothing)
                // 1: stop (takes self, returns int)

                context.memory.non_primitives = vec![
                    // struct car, 3 fields, brand name, is for sports, speed, id = 0
                    NonPrimitiveType {
                        name: "Car".to_string(),
                        kind: NonPrimitiveTypes::Struct,
                        // brand name, is for sports, speed (includes header)
                        len: 4,
                        // brand name
                        pointers: 1,
                        methods: HashMap::from_iter(vec![(0, vec![9, 19])]),
                    },
                    // struct motorcycle, 3 fields, brand name, model, speed, id = 1
                    NonPrimitiveType {
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
                    Res(10, 0),
                    // first get pointer to car
                    Ptr(10),
                    // then get car struct from stack
                    Rd(10, GENERAL_REG2),
                    // call drive
                    // reserve stack space for arguments
                    Res(1, 0),
                    // first argument is self
                    // note: values are pushed in reverse order and indexing starts from 1
                    Wr(1, GENERAL_REG1),
                    Mtd(GENERAL_REG2, 0, 0),
                    // return registers to their original values
                    Ufrz,
                    SweepUnoptimized,
                    End,
                    // method drive for car
                    // prints "I am driving with BMW at 200 km/h"
                    // methods have 1 argument, self
                    // method return if it is for sports
                    // so we have to read it from the stack using Rd(stack_offset + 1, reg)
                    // rest of the methods will remain undeclared because they are take too long to write for human
                    Rd(1, POINTER_REG),
                    // get brand name
                    IdxK(1),
                    Rdp(GENERAL_REG1),
                    // get speed
                    // first get pointer to struct Car
                    Rd(1, POINTER_REG),
                    IdxK(3),
                    Rdp(GENERAL_REG2),
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
                    Rdc(9, POINTER_REG),
                    //StrCat(GENERAL_REG1),
                    // concatenate with " at "
                    Rdc(11, GENERAL_REG1),
                    //StrCat(GENERAL_REG1),
                    // concatenate with speed
                    //StrCat(GENERAL_REG2),
                    // concatenate with " km/h"
                    Rdc(12, GENERAL_REG1),
                    //StrCat(GENERAL_REG1),
                    //StdOut(POINTER_REG),
                    // load return value into return register
                    Rd(1, POINTER_REG),
                    IdxK(2),
                    Rdp(RETURN_REG),
                    Ret,
                    // method stop for car
                    Rdc(1, GENERAL_REG1),
                    //StdOut(GENERAL_REG1),
                    Ret,
                ];
                true
            }
            9 => {
                context.set_libs(load_libs(vec!["io"]));

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
                    Rdc(0, POINTER_REG),
                    Cal(0, 1),
                    Cal(0, 2),
                    Move(RETURN_REG, GENERAL_REG1),
                    // print it back
                    Rdc(1, POINTER_REG),
                    Cal(0, 0),
                    Swap(GENERAL_REG1, POINTER_REG),
                    Cal(0, 0),
                    // append to file
                    Move(RETURN_REG, GENERAL_REG1),
                    Rdc(3, POINTER_REG),
                    Cal(0, 5),
                    // load file
                    Rdc(2, POINTER_REG),
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
                context.set_libs(load_libs(vec!["io"]));
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
                    Res(4, 0),
                    // get args
                    Cal(0, 4),
                    Wr(4, RETURN_REG),
                    Move(RETURN_REG, GENERAL_REG1),
                    Len(RETURN_REG),
                    Wr(1, RETURN_REG),
                    // loop starts here
                    // get idx
                    Rd(3, GENERAL_REG1),
                    // get len
                    Rd(1, GENERAL_REG2),
                    // compare
                    Less(GENERAL_REG1, GENERAL_REG2, GENERAL_REG1),
                    Brnc(10 /* another round */, 20 /* end of loop */),
                    // loop body
                    // get idx
                    Rd(3, GENERAL_REG1),
                    // get arg
                    Rd(4, POINTER_REG),
                    Idx(GENERAL_REG1),
                    Rdp(POINTER_REG),
                    // print arg
                    Cal(0, 1),
                    // increment idx
                    Rd(3, GENERAL_REG1),
                    Rd(2, GENERAL_REG2),
                    Add(GENERAL_REG1, GENERAL_REG2, GENERAL_REG1),
                    Wr(3, GENERAL_REG1),
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
            _ => {
                context.memory.stack.data = vec![Int(0)];
                context.code.data = vec![End];
                println!("Test id: {test_id} not found.");
                true
            }
        }
    }
    pub fn load_lib(path: &PathBuf) -> Box<dyn runtime::lib::Library + Send> {
        let lib = unsafe { Library::new(path).unwrap() };
        let init_fn: libloading::Symbol<fn() -> Box<dyn runtime::lib::Library + Send>> =
            unsafe { lib.get(b"init").unwrap() };
        let lib_box = init_fn();

        mem::forget(lib);
        lib_box
    }
    pub fn load_libs(libs: Vec<&str>) -> Vec<Box<dyn runtime::lib::Library + Send>> {
        let mut result = vec![];

        for lib_path in &libs {
            let lib = unsafe { Library::new(std_path(lib_path)).unwrap() };
            let init_fn: libloading::Symbol<fn() -> Box<dyn runtime::lib::Library + Send>> =
                unsafe { lib.get(b"init").unwrap() };
            let lib_box = init_fn();

            result.push(lib_box);
            mem::forget(lib);
        }

        drop(libs);
        result
    }
    // Returns path to standard library
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
