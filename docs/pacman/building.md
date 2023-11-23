# Build Project

This includes all the commands that could be used to build a project.

## Build

To build a project, run `ruda build` or `ruda build <path>`.

This will build the project in the current directory (or specified) and creates:

- A `target` directory, which contains the compiled code + compiler artifacts.

### Options

- `--profile <profile>`: Sets the profile of the build. Can be any of the specified profiles in the `Ruda.toml` file.
- `--help`: Prints help information.

## Run

To run a project, run `ruda run` or `ruda run <path>`.

This is the same as running `ruda build` and then running the compiled code.

Only difference is that you can specify arguments to the program by adding them after the command (e.g. `ruda run -- arg0 arg1`).