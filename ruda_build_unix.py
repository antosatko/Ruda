import os
import shutil

# This script builds the Ruda compiler and VM and copies them to the build folder

# Prerequisites:
# - Rust (https://www.rust-lang.org/tools/install)

# After running this script, check the build folder.
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

# To access the compiler and VM from anywhere, add the bin folder to your PATH.
# IMPORTANT: Ruda needs to be able to find these files to run.
#            In your system variables, add RUDA_PATH and set it to the build folder.

# User-defined variables (change to your liking)
root_dir = os.path.join(os.getcwd(), "build")  # where to put Ruda
current_dir = os.getcwd()  # current directory (don't change this)
source_libs = os.path.join(current_dir, "stdlib")  # where to find the stdlib source code

# Create bin folder
if not os.path.exists(os.path.join(root_dir, "bin")):
    os.makedirs(os.path.join(root_dir, "bin"))

# Create stdlib folder
if not os.path.exists(os.path.join(root_dir, "stdlib")):
    os.makedirs(os.path.join(root_dir, "stdlib"))

# Build vm
os.system(f"cd {current_dir}/vm && cargo build --release")
if os.WEXITSTATUS(os.system("echo $?")) != 0:
    print("Build failed")
    exit(1)

# Build compiler
os.system(f"cd {current_dir}/compiler && cargo build --release")
if os.WEXITSTATUS(os.system("echo $?")) != 0:
    print("Build failed")
    exit(1)

path_to_vm = os.path.join(current_dir, "vm/target/release/rusty_vm")
path_to_compiler = os.path.join(current_dir, "compiler/target/release/rusty_danda")
path_to_binaries = os.path.join(root_dir, "bin")

# Remove old vm
os.remove(os.path.join(path_to_binaries, "rudavm"))

# Remove old compiler
os.remove(os.path.join(path_to_binaries, "rudac"))

# Copy vm
shutil.copy(path_to_vm, os.path.join(path_to_binaries, "rudavm"))

# Copy compiler
shutil.copy(path_to_compiler, os.path.join(path_to_binaries, "rudac"))

vm_rename = "rudavm"
compiler_rename = "rudac"

# Rename vm
os.rename(os.path.join(path_to_binaries, "rusty_vm"), os.path.join(path_to_binaries, vm_rename))

# Rename compiler
os.rename(os.path.join(path_to_binaries, "rusty_danda"), os.path.join(path_to_binaries, compiler_rename))

# Copy stdlib
lib_bin = os.path.join(root_dir, "stdlib")

# Get all folders excluding .git
folders = [f for f in os.listdir(source_libs) if os.path.isdir(os.path.join(source_libs, f)) and f != ".git"]

for folder in folders:
    # Rebuild library
    print(f"Building {folder}")
    os.system(f"cd {os.path.join(source_libs, folder)} && cargo build --release")

    path_to_bin = os.path.join(path_to_binaries, folder, "target/release", folder + ".so")

    # Remove old stdlib
    os.remove(os.path.join(lib_bin, f"{folder}.so"))

    # Copy new stdlib
    shutil.copy(os.path.join(source_libs, folder, f"target/release/{folder}.so"), lib_bin)

path_to_ast = os.path.join(current_dir, "compiler/ast")
ruda_ast = os.path.join(path_to_ast, "ruda.ast")
registry_ast = os.path.join(path_to_ast, "registry.ast")

# Remove old ast
os.remove(os.path.join(root_dir, "ruda.ast"))
os.remove(os.path.join(root_dir, "registry.ast"))

# Copy new ast
shutil.copy(ruda_ast, root_dir)
shutil.copy(registry_ast, root_dir)