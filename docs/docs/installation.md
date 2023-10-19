# Installation

> **_Important:_**  Ruda installer is currently under development. This guide will be updated once it is ready.

## Building from source

### Requirements

- [Rust](https://www.rust-lang.org/tools/install)
- [Python](https://www.python.org/downloads/)

### Steps

#### 1. Clone the repository

```bash
git clone https://github.com/it-2001/Ruda
```

#### 2. Run the build script

```bash
cd Ruda
py ruda_build.py
```

This will create a `build` directory containing the necessary files.

#### 3. Add the `build/bin` directory to your `PATH` environment variable

> See [this guide](https://www.architectryan.com/2018/03/17/add-to-the-path-on-windows-10/) for instructions on how to do this on Windows.

#### 4. Create a new environment variable called `RUDA_PATH` and set it to the `build` directory

#### 5. Run `ruda` to check if everything is working

```bash
ruda -V
```

If everything is working, you should see the following output:

```bash
Ruda pacman x.x.x
```