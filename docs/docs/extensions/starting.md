# Starting

Ruda source code has a template for creating extensions, so we will use it

> Extension system will most likely have some small changes before full release. Keep that in mind.

Got to [Ruda](https://github.com/it-2001/Ruda) and clone the repository. We will need the source code to compile the extension. (this will be simplified later).

Copy `stdlib/base/` to the same directory and rename it to `test`. This is the root of your project.

Before starting, go to `Cargo.toml` and change the name of your project to `test`. This will give you this:

```toml
[package]
name = "test"
version = "0.1.0"
edition = "2021"

# See more keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html

[dependencies]
runtime = {path = "../../vm/runtime"}

[lib]
path = "lib.rs"
crate-type = ["cdylib"]


[profile.dev]
opt-level = 3
debug = false
```

If everything works, you should be able to compile test using `cargo build --release`. This will output your extension to `target/release/test.dll` (`libtest.so` on Linux).

Take the extension file and place it to some Ruda project root directory. Open `Ruda.toml` file and add:


```toml
[binaries]
test = "test.dll"
```

Compile the project `ruda run`. This should run.