# Justfile for DayZ Server Manager - Rust Edition

# Set shell for Windows
set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

# Default recipe to display help when no command is specified
_default: help

# Show all available commands with descriptions
help:
    @just --list

# Build the project in debug mode (faster compilation, slower runtime)
build:
    cargo build

# Build the project in release mode (optimized for performance)
build-release:
    cargo build --release

# Run the project in debug mode, passing any additional arguments
run *ARGS:
    cargo run -- {{ARGS}}

# Run the project in release mode, passing any additional arguments
run-release *ARGS:
    cargo run --release -- {{ARGS}}

# Install debug build to specified path
# Usage: just install-debug /path/to/directory
install-debug PATH: build
    cp target/debug/dzsm.exe {{PATH}}/dzsm.exe

# Install release build to specified path  
# Usage: just install /path/to/directory
install PATH: build-release
    cp target/release/dzsm.exe {{PATH}}/dzsm.exe

# Remove build artifacts and intermediate files
clean:
    cargo clean

# Clean everything including distribution directory and removes both Cargo build artifacts and any custom dist folder
clean-all: clean
    rm -rf dist/
