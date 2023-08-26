# This script builds the Ruda compiler and VM and copies them to the build folder
#
# Prerequisites:
# - Rust (https://www.rust-lang.org/tools/install)
#
# After running this script check the build folder
# It should look something like this:
# - bin
#   - rudac.exe             (compiler)
#   - rudavm.exe            (vm)
#   - ruda.exe              (package manager)
# - stdlib
#   - io.dll
#   - string.dll
#   - ...
# - ruda.ast
# - registry.ast
# - Ruda.toml               (package manager config)
#
# Those are the files needed to run Ruda programs
#
# To access the compiler and VM from anywhere add the bin folder to your PATH
# IMPORTANT: Ruda needs to be able to find these files to run
#            In your system variables add RUDA_PATH and set it to the build folder
#
# This script is distributed with test.rdbin
# To test if everything works run: rudavm test.rdbin -- "Hello World!"
#                                  This should print "Hello World!"
#                                  If it doesn't check your PATH and RUDA_PATH
#


import os

import ctypes, sys


# user defined variables (change to your liking)
root_dir = os.path.join(os.getcwd(), "build\\") # where to put Ruda
current_dir = os.getcwd() + "\\" # current directory (don't change this)
source_libs = os.path.join(current_dir, "stdlib") # where to find the stdlib source code

# create bin folder
if not os.path.exists(root_dir + "\\bin"):
    os.makedirs(root_dir + "\\bin")

# create stdlib folder
if not os.path.exists(root_dir + "\\stdlib"):
    os.makedirs(root_dir + "\\stdlib")


# build vm
proc = os.system("cd " + current_dir + "vm && cargo build --release")
if proc != 0:
    print("Build failed")
    exit(1)


# build compiler
proc = os.system("cd " + current_dir + "compiler && cargo build --release")
if proc != 0:
    print("Build failed")
    exit(1)


# build package manager
proc = os.system("cd " + current_dir + "pacman && cargo build --release")
if proc != 0:
    print("Build failed")
    exit(1)

path_to_vm = os.path.join(current_dir, "vm\\target\\release\\rusty_vm.exe")
path_to_compiler = os.path.join(current_dir, "compiler\\target\\release\\rusty_danda.exe")
path_to_binaries = os.path.join(root_dir + "\\bin")
path_to_package_manager = os.path.join(current_dir, "pacman\\target\\release\\ruda.exe")


# remove old vm
print('del ' + root_dir +'\\bin\\rusty_vm.exe')
print('del ' + root_dir +'\\bin\\rudavm.exe')
proc = os.system('del ' + root_dir +'\\bin\\rudavm.exe')

# remove old compiler
proc = os.system('del ' + root_dir +'\\bin\\rudac.exe')

# remove old package manager
proc = os.system('del ' + root_dir +'\\bin\\ruda.exe')


# copy vm
print('copy ' + current_dir + 'vm\\target\\release\\rusty_vm.exe ' + root_dir +'\\bin')
proc = os.system('copy ' + current_dir + 'vm\\target\\release\\rusty_vm.exe ' + root_dir +'\\bin')

# copy compiler
proc = os.system('copy ' + current_dir + 'compiler\\target\\release\\rusty_danda.exe ' + root_dir +'\\bin')

# copy package manager
proc = os.system('copy ' + current_dir + 'pacman\\target\\release\\ruda.exe ' + root_dir +'\\bin')

vm_rename = "rudavm.exe"
compiler_rename = "rudac.exe"

# rename vm
proc = os.system('rename "' + root_dir +'\\bin\\rusty_vm.exe" ' + vm_rename)

# rename compiler
proc = os.system('rename "' + root_dir +'\\bin\\rusty_danda.exe" ' + compiler_rename)


# copy stdlib
lib_bin = root_dir + "\\stdlib"

# get all folders excluding .git
folders = [f for f in os.listdir(source_libs) if os.path.isdir(os.path.join(source_libs, f)) and f != ".git"]

for folder in folders:
    # rebuild library
    print("Building " + folder)
    proc = os.system("cd " + os.path.join(source_libs, folder) + " && cargo build --release")

    print("Path to bin: " + path_to_binaries)
    path_to_bin = os.path.join(path_to_binaries, folder, "target\\release", folder + ".dll")

    print("Removing old " + folder)
    # remove old stdlib
    proc = os.system('del ' + lib_bin + "\\" + folder + ".dll")

    print("Copying new " + folder)
    # copy new stdlib
    proc = os.system('copy "' + os.path.join(source_libs, folder, "target\\release", folder + ".dll") + '" "' + lib_bin + '"')


path_to_ast = os.path.join(current_dir, "compiler\\ast")
ruda_ast = path_to_ast + "\\ruda.ast"
registry_ast = path_to_ast + "\\registry.ast"

# remove old ast
proc = os.system('del ' + root_dir + '\\ruda.ast')
proc = os.system('del ' + root_dir + '\\registry.ast')

# copy new ast
proc = os.system('copy ' + ruda_ast + ' ' + root_dir + '')
proc = os.system('copy ' + registry_ast + ' ' + root_dir + '')

# copy Ruda.toml if it isn't already there
if not os.path.exists(root_dir + "\\Ruda.toml"):
    proc = os.system('copy ' + current_dir + '\\pacman\\templates\\Ruda.toml ' + root_dir + '')