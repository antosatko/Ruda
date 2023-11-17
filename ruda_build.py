import os
import shutil
import platform

# User-defined variables (change to your liking)
current_dir = os.getcwd()  # Current directory (don't change this)
root_dir = os.path.join(current_dir, "build")  # Where to put Ruda
source_libs = os.path.join(current_dir, "stdlib")  # Where to find the stdlib source code

os.makedirs(root_dir, exist_ok=True)


# Copy LICENSE
shutil.copy("LICENSE", os.path.join(root_dir, "LICENSE"))

# Create bin folder
bin_dir = os.path.join(root_dir, "bin")
os.makedirs(bin_dir, exist_ok=True)

# Create stdlib folder
stdlib_dir = os.path.join(root_dir, "stdlib")
os.makedirs(stdlib_dir, exist_ok=True)

# Build VM
vm_dir = os.path.join(current_dir, "vm")
os.chdir(vm_dir)
os.system("cargo build --release")

# Build compiler
compiler_dir = os.path.join(current_dir, "compiler_cli")
os.chdir(compiler_dir)
os.system("cargo build --release")

# Build package manager
pacman_dir = os.path.join(current_dir, "pacman")
os.chdir(pacman_dir)
os.system("cargo build --release")

# Paths to executables and binaries
path_to_vm = os.path.join(vm_dir, "target", "release", "rusty_vm")
path_to_compiler = os.path.join(compiler_dir, "target", "release", "compiler_cli")
path_to_binaries = bin_dir
path_to_package_manager = os.path.join(pacman_dir, "target", "release", "ruda")

# Determine the file extension based on the platform
if platform.system() == "Windows":
    executable_ext = ".exe"
else:
    executable_ext = ""

# Check if VM binary exists
if not os.path.exists(path_to_vm + executable_ext):
    print("Error: VM binary not found at", path_to_vm + executable_ext)
else:
    # Remove old VM
    old_vm_path = os.path.join(bin_dir, "rudavm" + executable_ext)
    if os.path.exists(old_vm_path):
        os.remove(old_vm_path)

    # Copy VM
    shutil.copy(path_to_vm + executable_ext, os.path.join(bin_dir, "rudavm" + executable_ext))

# Check if compiler binary exists
if not os.path.exists(path_to_compiler + executable_ext):
    print("Error: Compiler binary not found at", path_to_compiler + executable_ext)
else:
    # Remove old compiler
    old_compiler_path = os.path.join(bin_dir, "rudac" + executable_ext)
    if os.path.exists(old_compiler_path):
        os.remove(old_compiler_path)

    # Copy compiler
    shutil.copy(path_to_compiler + executable_ext, os.path.join(bin_dir, "rudac" + executable_ext))

# Check if package manager binary exists
if not os.path.exists(path_to_package_manager + executable_ext):
    print("Error: Package manager binary not found at", path_to_package_manager + executable_ext)
else:
    # Remove old package manager
    old_package_manager_path = os.path.join(bin_dir, "ruda" + executable_ext)
    if os.path.exists(old_package_manager_path):
        os.remove(old_package_manager_path)

    # Copy package manager
    shutil.copy(path_to_package_manager + executable_ext, os.path.join(bin_dir, "ruda" + executable_ext))

# Define a dictionary to map file extensions for different platforms
platform_extensions = {
    "Windows": ".dll",
    "Linux": ".so",
    "Darwin": ".dylib"  # If targeting macOS
}

# Define a dictionary to map file prefexes for different platforms
platform_prefixes = {
    "Windows": "",
    "Linux": "lib",
    "Darwin": "lib"  # If targeting macOS
}

# Determine the file extension based on the platform
executable_ext = platform_extensions.get(platform.system(), "")
executable_pre = platform_prefixes.get(platform.system(), "")


stdlib_binaries = {
}

# Copy stdlib excluding "base"
for folder in os.listdir(source_libs):
    if folder != "base" and os.path.isdir(os.path.join(source_libs, folder)):
        # Rebuild library
        print("Building " + folder)
        folder_path = os.path.join(source_libs, folder)
        os.chdir(folder_path)
        os.system("cargo build --release")

        # Specify the binary name for this library
        binary_name = stdlib_binaries.get(folder, folder + executable_ext)

        print("Removing old " + binary_name)
        # Remove old stdlib
        old_stdlib_path = os.path.join(stdlib_dir, binary_name)
        if os.path.exists(old_stdlib_path):
            os.remove(old_stdlib_path)
        
        print("Copying new " + binary_name)

        temp_bin_name = binary_name

        binary_name = executable_pre + binary_name
        # Copy new stdlib
        shutil.copy(os.path.join(folder_path, "target", "release", binary_name), stdlib_dir)

        os.rename(os.path.join(stdlib_dir, binary_name), os.path.join(stdlib_dir, temp_bin_name))

# Copy AST files
path_to_ast = os.path.join(current_dir, "compiler", "ast")
ruda_ast = os.path.join(path_to_ast, "ruda.ast")
registry_ast = os.path.join(path_to_ast, "registry.ast")

# Remove old AST files
old_ruda_ast_path = os.path.join(root_dir, "ruda.ast")
if os.path.exists(old_ruda_ast_path):
    os.remove(old_ruda_ast_path)

old_registry_ast_path = os.path.join(root_dir, "registry.ast")
if os.path.exists(old_registry_ast_path):
    os.remove(old_registry_ast_path)

# Copy new AST files
shutil.copy(ruda_ast, root_dir)
shutil.copy(registry_ast, root_dir)

# Copy Ruda.toml if it isn't already there
if not os.path.exists(os.path.join(root_dir, "Ruda.toml")):
    shutil.copy(os.path.join(pacman_dir, "templates", "Ruda.toml"), root_dir)

