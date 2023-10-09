use std::any::Any;
use std::fs::File;
use std::io::Write;

fn main() {
    let file = File::create("example.txt").expect("Failed to create file");

    // Cast the File into Any
    let file_as_any: Box<dyn Any> = Box::new(file);

    // Attempt to cast it back to File
    if let Some(file) = file_as_any.downcast_ref::<File>() {
        // Successfully cast back to File
        let mut file = file.try_clone().expect("Failed to clone file");

        // Write data to the file
        file.write_all(b"Hello, World!").expect("Failed to write to file");
    } else {
        println!("Failed to cast back to File");
    }
}
