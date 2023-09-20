# Ruda
![Lines of Code](https://aschey.tech/tokei/github/it-2001/Ruda?labelColor=badbe6&color=32a852&style=for-the-badge&label=Lines&logo=https://simpleicons.org/icons/rust.svg)

<a><img src="logo.png" align="middle" height="256" width="256" ></a>
## Table of Contents

- [About](#about)
- [Syntax](#syntax)
- [Building from source](#building-from-source)

## About

Ruda was designed to offer an easy but performant alternative to common languages. It aims to provide flawless experience for hobbyists. Ruda was never meant to go into production but it is more than capable to fill this role. To learn how to use Ruda, see [Ruda docs](https://it-2001.github.io/Ruda-docs/).

Our charming mascot does not have name yet. Help pick one for this nameless quail. 🐦

## Syntax

More examples will be inside `examples/` directory.

### Hello world

```Ruda
import "std.io"

fun main() {
    io.println("Hello world")
}
```

## Building from source

If you are one of those crazy people and want to modify the source code or have any other reasons to, you are more than welcome to!

Just clone the repository and run `py ruda_build.py`. This should create a  `build/` directory with the whole application.

Add `build/bin` to your path variable and create a new variable `RUDA_PATH` with the path to `build`. If this is unclear, look up _How to Change the PATH Environment Variable on Windows_.

Open new terminal and try `rudavm .\test.rdbin -- Hello, World!`.

## Other

For other information about the Ruda components see their directories.
