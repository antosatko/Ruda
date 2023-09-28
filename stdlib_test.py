import os
import platform

libs_dir = os.path.join(os.getcwd(), "stdlib")

# Check if stdlib folder exists
if not os.path.exists(libs_dir):
    print("Error: stdlib folder not found at", libs_dir)

# Define a dictionary to map file extensions for different platforms
platform_extensions = {
    "Windows": ".dll",
    "Linux": ".so",
    "Darwin": ".dylib"  # If targeting macOS
}

# Determine the file extension based on the platform
lib_ext = platform_extensions.get(platform.system(), "")

if lib_ext == "":
    print("Error: Unsupported platform", platform.system())
    exit(1)


for folder in os.listdir(libs_dir):
    # Skip "base" folder
    if folder == "base":
        continue
    
    # Check if folder is a directory
    if not os.path.isdir(os.path.join(libs_dir, folder)):
        continue

    # Compile library
    print("Compiling", folder)
    os.system("cd " + os.path.join(libs_dir, folder) + " && cargo build --release")
    
    # Run rudac libload to test library
    print("Loading", folder)
    if os.system("cd " + os.path.join(libs_dir, folder, "target", "release") + " && rudac libload " + folder + lib_ext + " mute") != 0:
        print("Error: Failed to load", folder)
        exit(1)
    

print("Success")
