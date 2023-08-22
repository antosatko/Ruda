#!/bin/bash

# This script builds the Ruda compiler and VM and copies them to the build folder
#
# Prerequisites:
# - Rust (https://www.rust-lang.org/tools/install)
#
# After running this script check the build folder
# It should look something like this:
# - bin
#   - rudac
#   - rudavm
# - stdlib
#   - io.so
#   - string.so
#   - ...
# - ruda.ast
# - registry.ast
#
# Those are the files needed to run Ruda programs
#
# To access the compiler and VM from anywhere, add the bin folder to your PATH
# IMPORTANT: Ruda needs to be able to find these files to run
#            In your system variables, add RUDA_PATH and set it to the build folder
#
# This script is distributed with test.rdbin
# To test if everything works, run: rudavm test.rdbin -- "Hello World!"
#                                  This should print "Hello World!"
#                                  If it doesn't, check your PATH and RUDA_PATH

# user defined variables (change to your liking)
root_dir="$(pwd)/build/" # where to put Ruda
current_dir="$(pwd)/"   # current directory (don't change this)
source_libs="${current_dir}ruda_std" # where to find the stdlib source code

# Set RUDA_PATH environment variable
export RUDA_PATH="${root_dir}"

# create bin folder
mkdir -p "${root_dir}bin"

# create stdlib folder
mkdir -p "${root_dir}stdlib"

# build vm
if ! (cd "${current_dir}rusty-vm" && cargo build --release); then
    echo "Build failed"
    exit 1
fi

# build compiler
if ! (cd "${current_dir}Rusty-compiler" && cargo build --release); then
    echo "Build failed"
    exit 1
fi

path_to_vm="${current_dir}rusty-vm/target/release/rusty_vm"
path_to_compiler="${current_dir}Rusty-compiler/target/release/rusty_danda"
path_to_binaries="${root_dir}bin"

# remove old vm and compiler
rm -f "${root_dir}bin/rudavm" "${root_dir}bin/rudac"

# copy vm and compiler
cp "${path_to_vm}" "${path_to_binaries}/rudavm"
cp "${path_to_compiler}" "${path_to_binaries}/rudac"

# copy stdlib
lib_bin="${root_dir}stdlib"

# get all folders excluding .git
folders=( "${source_libs}"/*/ )
for folder in "${folders[@]}"; do
    folder="$(basename "${folder}")"
    # rebuild library
    echo "Building ${folder}"
    if ! (cd "${source_libs}/${folder}" && cargo build --release); then
        echo "Library build failed: ${folder}"
        exit 1
    fi

    path_to_bin="${path_to_binaries}/${folder}/target/release/lib${folder}.so"

    # remove old stdlib
    rm -f "${lib_bin}/${folder}.so"

    # copy new stdlib
    cp "${source_libs}/${folder}/target/release/lib${folder}.so" "${lib_bin}"

    # rename stdlib file without 'lib' prefix
    mv "${lib_bin}/lib${folder}.so" "${lib_bin}/${folder}.so"
done

path_to_ast="${current_dir}Rusty-compiler/ast"
ruda_ast="${path_to_ast}/ruda.ast"
registry_ast="${path_to_ast}/registry.ast"

# remove old ast
rm -f "${root_dir}/ruda.ast" "${root_dir}/registry.ast"

# copy new ast
cp "${ruda_ast}" "${root_dir}"
cp "${registry_ast}" "${root_dir}"
