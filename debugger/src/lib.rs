use std::sync::{Arc, Mutex, mpsc};

use runtime::*;


/// Run the debugger.
/// 
/// Context
pub fn run(mut ctx: runtime::runtime_types::Context) {
    let cycles = 10000;
    // create a debug data object
    let debug_data = Arc::new(Mutex::new(DebugData{}));
    let debug_data_reciever = debug_data.clone();
    // create a thread to run the context
    let handle = std::thread::spawn(move || {
        // run context for 10000 instructions and update the debug data
        loop {
            ctx.run_for(cycles);
            // update debug data
            let mut debug_data = debug_data_reciever.lock().unwrap();
        }
    });
    loop {
        // every 5 seconds
        // print debug data
        println!("{:?}", debug_data.lock().unwrap());
        std::thread::sleep(std::time::Duration::from_secs(5));
    }
    handle.join().unwrap();

}

#[derive(Debug)]
pub struct DebugData {

}