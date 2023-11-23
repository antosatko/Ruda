# Ruda extensions

Extension is a native library, that uses the Ruda ABI. In other words it is a library (.dll, .so), that is compatible with Ruda applications.

Extensions run independently to the Ruda binary, meaning that they can use the computer resources directly. This allows them to read files, write to console, send http requests, etc.

You are probably using extensions in all of your Ruda programs, since without access to the computer, your appication would be useless.

Even if you wrote only "useless" programs, Ruda still uses extensions under the hood, to replace some missing instructions. For example if you copy a string, it runs a certain procedure from the core extension.

In this chapter you will learn how to write your own extensions.

> Extensions support only the Rust programming language, it is recommended to know at least basic Rust.
