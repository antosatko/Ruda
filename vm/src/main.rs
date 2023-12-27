extern crate runtime;
extern crate stringify;
use enable_ansi_support::enable_ansi_support;
use runtime::runtime_types::*;
use stringify::ShLib;
use std::time::SystemTime;

mod test;

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(
    name = "Ruda VM",
    version = "0.1.0",
    author = "antosatko",
    about = "Ruda Virtual Machine CLI",
    after_help = "This is a CLI for the Ruda Virtual Machine. It can be used to run Ruda bytecode files (.rdbin)."
)]
struct Args {
    /// Input file
    input: Option<String>,

    /// Post-process data report
    #[clap(short, long, default_value = "false")]
    report: bool,

    /// Measure runtime
    #[clap(short, long, default_value = "false")]
    time: bool,

    /// Runtime arguments for the VM
    #[clap(name = "args", last = true)]
    args: Vec<String>,
    
    /// VM reports each instruction as it is executed
    #[clap(name = "debug", long)]
    debug: bool,
}

fn main() {
    let args = Args::parse();
    let mut report = args.report;
    let mut ctx = match args.input {
        Some(src) => {
            let ruda_path = std::env::var("RUDA_PATH").unwrap();
            let file = match std::fs::read(&src){
                Ok(bytes) => {bytes},
                Err(err) => {
                    let mut not_found = "Failed to read file: ".to_string();
                    not_found.push_str(&src);
                    not_found.push_str("\nReason: ");
                    not_found.push_str(&err.to_string());
                    println!("{not_found}");
                    return;
                }
            };
            let libs = vec![
                ShLib { path: "io".to_string(), owns: stringify::LibOwner::Standard},
                ShLib { path: "string".to_string(), owns: stringify::LibOwner::Standard},
                ShLib { path: "fs".to_string(), owns: stringify::LibOwner::Standard},
                ShLib { path: "algo".to_string(), owns: stringify::LibOwner::Standard},
                ShLib { path: "core".to_string(), owns: stringify::LibOwner::Standard},
                ShLib { path: "time".to_string(), owns: stringify::LibOwner::Standard},
                ShLib { path: "window".to_string(), owns: stringify::LibOwner::Standard},
                ShLib { path: "memory".to_string(), owns: stringify::LibOwner::Standard},
            ];
            let mut libs_read = Vec::new();
            for lib in libs.iter() {
                libs_read.push(test::test::load_lib(&lib.into_real_path(&src, &ruda_path), 0));
            }
            let mut ctx = Context::new(libs_read);
            let data = stringify::parse(&String::from_utf8(file).unwrap());
            ctx.memory.stack.data = data.values;
            ctx.memory.strings.pool = data.strings;
            ctx.code.data = data.instructions;
            ctx.memory.non_primitives = data.non_primitives;
            ctx.memory.fun_table = data.fun_table;
            ctx.memory.heap.data = data.heap;
            ctx.code.ptr = data.entry_point;
            ctx.code.entry_point = data.entry_point;
            ctx
        }
        None => {
            /*println!("Path not specified. Program will terminate."); return;*/
            use test::test::*;
            let ruda_path = std::env::var("RUDA_PATH").unwrap();
            let libs: Vec<ShLib> = vec![/*
                ShLib { path: "io".to_string(), owns: stringify::LibOwner::Standard},
                ShLib { path: "string".to_string(), owns: stringify::LibOwner::Standard},
                ShLib { path: "fs".to_string(), owns: stringify::LibOwner::Standard},
                ShLib { path: "algo".to_string(), owns: stringify::LibOwner::Standard},
                ShLib { path: "core".to_string(), owns: stringify::LibOwner::Standard},
            */];
            let mut libs_read = Vec::new();
            for lib in libs.iter() {
                libs_read.push(test::test::load_lib(&lib.into_real_path(&"", &ruda_path), 0));
            }
            let mut ctx = Context::new(libs_read);
            report = test_init(None, &mut ctx);
            let stringified = stringify::stringify(&ctx, None);
            // write to file
            std::fs::write("test.rdbin", stringified).unwrap();
            ctx
        }
    };
    ctx.memory.runtime_args = args.args;
    match args.time {
        true => {
            let start_time = SystemTime::now();
            match args.debug {
                true => {
                    ctx.run_debug();
                }
                false => {
                    ctx.run();
                }
            }
            match enable_ansi_support() {
                Ok(_) => {
                    println!(
                        "\x1b[90mTotal run time: {} ms\x1b[0m",
                        SystemTime::now()
                            .duration_since(start_time)
                            .unwrap()
                            .as_millis()
                    );
                }
                Err(_) => {
                    println!(
                        "Total run time: {} ms",
                        SystemTime::now()
                            .duration_since(start_time)
                            .unwrap()
                            .as_millis()
                    );
                }
            }
            if report {
                data_report(&ctx);
            }
        }
        false => {
            match args.debug {
                true => {
                    ctx.run_debug();
                }
                false => {
                    ctx.run();
                }
            }
            if report {
                data_report(&ctx);
            }
        }
    }
}

fn data_report(ctx: &Context) {
    use colored::Colorize;
    match enable_ansi_support() {
        Ok(_) => {
            print!("\n");
            println!("{}", "Post-process data report.".yellow());
            println!("{} {:?}", "Heap:".magenta(), ctx.memory.heap.data);
            println!("{} {:?}", "Stack:".magenta(), ctx.memory.stack.data);
            println!("{} {:?}", "Registers:".magenta(), ctx.memory.registers);
            println!("{} {:?}", "Strings:".magenta(), ctx.memory.strings.pool);
        }
        Err(_) => {
            print!("\n");
            println!("{}", "Post-process data report.");
            println!("{} {:?}", "Heap:", ctx.memory.heap.data);
            println!("{} {:?}", "Stack:", ctx.memory.stack.data);
            println!("{} {:?}", "Registers:", ctx.memory.registers);
            println!("{} {:?}", "Strings:", ctx.memory.strings.pool);
        }
    }
}
