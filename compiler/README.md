# Ruda compiler
<a><img src="logo.png" align="middle" height="256" width="256" ></a>

## Table of Contents

- [About](#about)
- [Getting Started](#getting_started)
- [Project Goals](#goals)
- [Syntax](#syntax)

## About <a name = "about"></a>

Compiler for Ruda programming language. This repository contains everything related to Rusty danda language processing. For bytecode interpreter look at https://github.com/it-2001/rusty-vm. Standard library will get proper documentation later.

## Getting Started <a name = "getting_started"></a>

All you need to have is cargo installed and run this as any other rust project.

## Project Goals <a name = "goals"></a>

Main goal is to learn Rust, earn bragging rights and have some language to write my side projects in.

### Important
- [x] tokenization
- [x] preprocessing
- [x] parsing ast rules
- [x] generating ast
- [x] ast analysis - partially
- [ ] type check
- [ ] binary generation

### Other plans
- [ ] optimization on ast
- [ ] optimization on binary
- [ ] create installer for end user
- [x] standard library
- [ ] game dev library (considering raylib)
- [ ] (almost) fully featured compiler + vm on browser
- [x] draw mascot (credits: https://github.com/antosmichael07)
- [ ] integrate Lua (might be fun)

### End Goal
- [ ] write a 2d game to demonstrate my languge
- [x] learn Rust
- [x] have fun

## Syntax <a name = "syntax"></a>
This is not really place to talk about syntax, but if you are interested, you can look at ``ast/ruda.ast``, where you will find source code for ast and also have a look at demos (only test.rd is guaranteed to be up to date).
