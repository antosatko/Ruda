# Ruda
![Lines of Code](https://aschey.tech/tokei/github/it-2001/Ruda?labelColor=badbe6&color=32a852&style=for-the-badge&label=Lines&logo=https://simpleicons.org/icons/rust.svg)

> “25 000+ lines of code is a big project” - @ThePrimeagen

<a><img src="logo.png" align="middle" height="256" width="256" ></a>
## Table of Contents

- [About](#about)
- [Syntax](#syntax)
- [Building from source](#building-from-source)

## About

Ruda was designed to offer an easy and user-friendly alternative to common languages. It aims to provide flawless experience for hobbyists. Ruda was never meant to go into production but it is more than capable to fill this role. 

To learn how to use Ruda, see [Ruda docs](https://it-2001.github.io/Ruda-docs/).

Our charming mascot does not have a name yet. We need your help to pick one for this nameless quail. 🐦

## Syntax

More examples will be inside `examples/` directory.

### Hello world

```Ruda
import "#io"

fun main() {
    io.println("Bird!")
}
```

## Building from source

If you are one of those crazy people and want to modify the source code or have any other reasons to, you are more than welcome to!

First you need to have:
 1. Python - https://www.python.org/downloads/
 2. Rust - https://www.rust-lang.org/tools/install

Just clone the repository and run `py ruda_build.py`. This should create a  `build/` directory with the whole application. You can move it anywhere you want.

Add `build/bin` to your enviroment path variable and create a new variable `RUDA_PATH` with the path to `build` directory. If this is unclear, look up _How to Change the PATH Environment Variable on Windows_ (or any other platform).

Ruda also requires `SFLM 2.6+` to be installed. You can download it from https://www.sfml-dev.org/download/sfml/2.6.1/. Make sure to follow the instructions for your platform and toolchain.

Open new terminal and try `ruda --version`.

### Compatability

Ruda is currently only supported on Windows. Linux support is planned for public testing.

Only 64-bit systems are supported. This is very unlikely to change in the future.

## Other

To learn more about the Ruda components see their respective directories. (good luck reading compiler source code)