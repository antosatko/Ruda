# New Project

## Create a new project

To create a new project, run the following command:

```bash
ruda init hello
```

This will create a new directory called `hello` containing the following files:

```
src/
    main.rd
Ruda.toml
.gitignore
```

## Running the project

First make sure you are in the project directory:

```bash
cd hello
```

Then run the following command:

```bash
ruda run
```

This will build and run the project. You should see the following output:

```
Hello, world!
```