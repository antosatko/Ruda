# Configuring Ruda Project

The `Ruda.toml` file is the configuration file for Ruda projects. It contains information about the project, such as the name, version, dependencies, etc.

When you create new project using `ruda init`, it will create a `Ruda.toml` file with the following content:

```toml
author = "author"
license = "MIT"
name = "project"
runtime = "latest"
version = "0.1.0"
kind = "bin"
description = "description"
3rdparty = "allow"
```

Some of the fields are self-explanatory, but some of them are not. So let's go over the confusing ones.

- `runtime`: The version of the Ruda runtime to use. Can be `latest` or `x.x.x`.
- `kind`: The kind of the project. Can be `bin` or `lib`.
- `3rdparty`: Whether to allow 3rd party dependencies. Can be:
    - `allow`: Allow 3rd party dependencies.
    - `std`: Allow only dependencies from the standard library.
    - `sandboxed`: Allow only dependencies from the standard library that are considered safe.
    - `deny`: Deny any dependencies, including the standard library.

## Dependencies

Dependencies are specified in the `[dependencies]` section of the `Ruda.toml` file.

```toml
[dependencies]
algebra = "https://git/repository/algebra.git"
```

The name of the dependency is the name of the package. The value is the URL to the Git repository of the package.

You can also specify additional options for the dependency:

```toml
[dependencies]
algebra = { path = "./algebra", version = "0.1.0", profile = "default", 3rdparty = "allow" }
```

- `path`: The path to the package. Can be a local path or a URL to a Git repository.
- `version`: The version of the package to use. Can be `latest` or `x.x.x`.
- `profile`: The profile of the package to use. Can be any of the specified profiles in the `Ruda.toml` file.
- `3rdparty`: Sets the 3rd party policy for the package and its dependencies.

## Profiles

Profiles are specified in the `[profiles]` section of the `Ruda.toml` file.

```toml
[profiles.embed]
3rdparty = "deny"
```

## Global Configuration

All projects share the same global configuration, which is located in the `path/to/ruda/Ruda.toml` file.

If you do not specify a field in the project configuration, it will use the value from the global configuration.