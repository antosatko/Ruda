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
#            In your shell configuration, add RUDA_PATH and set it to the build folder

# user defined variables (change to your liking)
root_dir=$(pwd)/build/       # where to put Ruda
current_dir=$(pwd)/          # current directory (don't change this)
source_libs="${current_dir}stdlib" # where to find the stdlib source code

# create bin folder
mkdir -p "${root_dir}/bin"

# create stdlib folder
mkdir -p "${root_dir}/stdlib"

# build vm
cd "${current_dir}vm" && cargo build --release
if [ $? -ne 0 ]; then
    echo "Build failed"
    exit 1
fi

# build compiler
cd "${current_dir}compiler" && cargo build --release
if [ $? -ne 0 ]; then
    echo "Build failed"
    exit 1
fi

path_to_vm="${current_dir}vm/target/release/rusty_vm"
path_to_compiler="${current_dir}compiler/target/release/rusty_danda"
path_to_binaries="${root_dir}/bin"

# remove old vm
rm -f "${root_dir}/bin/rudavm"

# remove old compiler
rm -f "${root_dir}/bin/rudac"

# copy vm
cp "${current_dir}vm/target/release/rusty_vm" "${root_dir}/bin/rudavm"

# copy compiler
cp "${current_dir}compiler/target/release/rusty_danda" "${root_dir}/bin/rudac"

vm_rename="rudavm"
compiler_rename="rudac"

# rename vm
mv "${root_dir}/bin/rusty_vm" "${root_dir}/bin/${vm_rename}"

# rename compiler
mv "${root_dir}/bin/rusty_danda" "${root_dir}/bin/${compiler_rename}"

# copy stdlib
lib_bin="${root_dir}/stdlib"

# get all folders excluding .git
folders=( $(find "${source_libs}" -mindepth 1 -maxdepth 1 -type d ! -name ".git" -exec basename {} \;) )

for folder in "${folders[@]}"; do
    # rebuild library
    echo "Building ${folder}"
    cd "${source_libs}/${folder}" && cargo build --release

    echo "Path to bin: ${path_to_binaries}"
    path_to_bin="${path_to_binaries}/${folder}/target/release/${folder}.so"

    echo "Removing old ${folder}"
    # remove old stdlib
    rm -f "${lib_bin}/${folder}.so"

    echo "Copying new ${folder}"
    # copy new stdlib
    cp "${source_libs}/${folder}/target/release/${folder}.so" "${lib_bin}"
done

path_to_ast="${current_dir}compiler/ast"
ruda_ast="${path_to_ast}/ruda.ast"
registry_ast="${path_to_ast}/registry.ast"

# remove old ast
rm -f "${root_dir}/ruda.ast"
rm -f "${root_dir}/registry.ast"

# copy new ast
cp "${ruda_ast}" "${root_dir}"
cp "${registry_ast}" "${root_dir}"
