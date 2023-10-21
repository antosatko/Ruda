# Ruda Package Manager

The Ruda Package Manager is a tool for managing packages written in Ruda. It is used to install, update, remove and search for 

It comes with the Ruda development kit and is installed by default.

It is not a bad idea to learn a thing or two about the package manager, sice it is the main way to interact with your Ruda projects.

## Help

To get help on the package manager, run `ruda help` or `ruda <command> -h`.

## Version

To get the version of the package manager, run `ruda -V`.

## Initialize a project

To initialize a project, run `ruda init` or `ruda init <path>`.

This will use the current directory (or specified) as the project directory and creates:

- A `Ruda.toml` file, which contains the project configuration.
- A `src` directory, which contains the source code of the project.
- A `src/main.rd` file, which contains the main function of the project.
- A `.gitignore` file, which contains the files that should be ignored by Git.

### Options

- `--name <name>`: Sets the name of the project.
- `--verison <version>`: Sets the version of the project.
- `--kind <kind>`: Sets the kind of the project. Can be `bin` or `lib`.
- `--author <author>`: Sets the author of the project.
- `--help`: Prints help information.