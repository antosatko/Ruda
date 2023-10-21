# Debugging

## Locate

If you think there is a problem with the source of included libraries, you can locate them by running `ruda locate <name>` or `ruda locate <git_url>`.

This will print the path to the source code of the library.

Example:

```toml
// Ruda.toml
[dependencies]
algebra = "https://git/repository/algebra.git"
```

```
ruda locate algebra
```

Output:

```
/path/to/ruda/packages/author/algebra
```

Then you can open the source code in your editor and debug it.

### No args

If you run `ruda locate` without any arguments, it will print the path to the `Ruda` directory.

To get path to the Ruda packages directory, run `ruda locate` and append `/packages` to the path.

## Restore

If you think there is a problem with the compiler artifacts, you can restore them by running `ruda restore`.

This will remove the `target` directory.

### Options

- `--help`: Prints help information.
- `--profile <profile>`: Sets the profile of the build. Can be any of the specified profiles in the `Ruda.toml` file.
- `--compile`: Compiles the project after restoring the artifacts.
- `--run`: Runs the project after restoring the artifacts.

> Also feel free to report any issues you find with the compiler or standard library on the [GitHub repository](https://github.com/it-2001/Ruda/issues)