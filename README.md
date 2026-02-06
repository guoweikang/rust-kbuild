# rust-kbuild

A complete Rust implementation of kconfig-standalone (Kconfig configuration system).

## Overview

`rust-kbuild` is a reimplementation of the Linux kernel's Kconfig configuration system in Rust. It provides a complete parser for Kconfig files, including support for:

- Configuration options (bool, tristate, string, int, hex)
- Menus and choice groups
- Dependencies (depends on, select, imply)
- Source directives with circular dependency detection
- Expression evaluation
- Configuration file I/O (.config, auto.conf, autoconf.h)

## Features

-  **Fast**: Written in Rust for performance and safety
-  **Complete Parser**: Full Kconfig syntax support
-  **Source Recursion**: Handles nested source directives with cycle detection
-  **Configuration Management**: Read/write .config files
-  **Build Integration**: Generate auto.conf and autoconf.h
-  **Change Detection**: Oldconfig support for detecting configuration changes
-  **Clean Output**: No CONFIG_ prefix in generated files
-  **Backward Compatible**: Reads configs with or without CONFIG_ prefix
-  **Well Tested**: Comprehensive test suite with real-world examples
-  **Well Documented**: Complete API and usage documentation

## Installation

### From Source

```bash
git clone https://github.com/guoweikang/rust-kbuild.git
cd rust-kbuild
cargo build --release
```

The binary will be available at `target/release/rkconf`.

### As a Library

Add to your `Cargo.toml`:

```toml
[dependencies]
rust-kbuild = "0.1"
```

## Quick Start

### Command Line

Parse a Kconfig file:

```bash
rkconf parse --kconfig Kconfig --srctree .
```

Save configuration with defaults:

```bash
rkconf saveconfig --output .config --kconfig Kconfig --srctree .
```

Load an existing configuration and detect changes:

```bash
rkconf oldconfig --config .config --kconfig Kconfig --srctree .
```

With auto defaults for new symbols:

```bash
rkconf oldconfig --config .config --kconfig Kconfig --srctree . --auto-defaults
```

Generate configuration files:

```bash
rkconf generate --config .config --kconfig Kconfig --srctree .
```

Interactive menu configuration (TUI):

```bash
rkconf menuconfig --kconfig Kconfig --srctree .
```

## Commands

### parse
Parse a Kconfig file and display the AST (Abstract Syntax Tree).

```bash
rkconf parse --kconfig Kconfig --srctree .
```

### saveconfig
Save the current configuration with default values from Kconfig.

```bash
rkconf saveconfig --output .config --kconfig Kconfig --srctree .
```

This command:
- Parses the Kconfig file to get all symbol definitions
- Applies default values from the Kconfig
- Generates `.config`, `auto.conf`, and `autoconf.h` files
- Output files do NOT contain `CONFIG_` prefix

### oldconfig
Load an existing `.config` file and detect changes against the current Kconfig.

```bash
rkconf oldconfig --config .config --kconfig Kconfig --srctree .
```

This command:
- Loads the existing `.config` file
- Compares it with the current Kconfig definitions
- Detects new symbols added to Kconfig
- Detects removed symbols (no longer in Kconfig)
- Preserves existing configuration values
- Shows a summary of changes

With `--auto-defaults` flag:
```bash
rkconf oldconfig --config .config --kconfig Kconfig --srctree . --auto-defaults
```
Automatically applies default values to any new symbols.

### generate
Generate `auto.conf` and `autoconf.h` from an existing `.config` file.

```bash
rkconf generate --config .config --kconfig Kconfig --srctree .
```

### defconfig
Apply a defconfig file.

```bash
rkconf defconfig <defconfig_file> --kconfig Kconfig --srctree .
```

### menuconfig
Interactive terminal UI for configuration (TUI).

```bash
rkconf menuconfig --kconfig Kconfig --srctree .
```

## Configuration File Format

### .config Format (without CONFIG_ prefix)

```bash
#
# Automatically generated file; DO NOT EDIT.
# Rust Kbuild Configuration
#
ARCH_X86=y
ENABLE_LOGGING=y
LOG_LEVEL="debug"
MAX_THREADS=8
# EXPERIMENTAL is not set
```

**Note**: Output files do NOT use the `CONFIG_` prefix. This is different from the Linux kernel's Kconfig but more aligned with Rust naming conventions.

### Backward Compatibility

The reader accepts both formats for backward compatibility:
- With prefix: `CONFIG_ARCH_X86=y`
- Without prefix: `ARCH_X86=y`

Both will be read correctly, but output will always be without the prefix.

## Documentation

- [Usage Guide](docs/USAGE.md) - Detailed usage instructions and examples
- [Design Document](docs/DESIGN.md) - Architecture and design decisions
- [API Documentation](docs/API.md) - Complete API reference

## Example

See the `examples/sample_project` directory for a complete example project with nested Kconfig files.

```
examples/sample_project/
├── Kconfig              # Main configuration
├── arch/
│   ├── x86/Kconfig      # x86 architecture options
│   └── arm/Kconfig      # ARM architecture options
└── kernel/Kconfig       # Kernel options
```

Run the example:

```bash
cargo run -- parse --kconfig examples/sample_project/Kconfig --srctree examples/sample_project
```

## Supported Kconfig Syntax

- **Configuration types**: bool, tristate, string, int, hex
- **Directives**: config, menuconfig, choice, menu, source, comment, mainmenu
- **Dependencies**: depends on, select, imply
- **Expressions**: Logical (&&, ||, !) and comparison (=, !=, <, <=, >, >=) operators
- **Blocks**: if/endif, menu/endmenu, choice/endchoice
- **Source recursion**: Nested source directives with cycle detection
- **Help text**: Multi-line help documentation

## Building and Testing

Build the project:

```bash
cargo build
```

Run tests:

```bash
cargo test
```

Run benchmarks:

```bash
cargo bench
```

Generate documentation:

```bash
cargo doc --open
```

## Project Status

### Implemented

- Complete Kconfig lexer and parser
- Source directive recursion with cycle detection
- Expression evaluation
- Configuration file reader/writer (without CONFIG_ prefix)
- Backward compatible reader (accepts with or without CONFIG_ prefix)
- Configuration generators (auto.conf, autoconf.h)
- Oldconfig support with change detection
- Saveconfig command
- Change tracking in symbol table
- Command-line interface
- Comprehensive test suite
- Documentation

### In Progress

- Interactive menuconfig TUI
- Defconfig support
- Constraint validation

### Planned

- Dependency resolution
- Export to JSON/YAML

## References

- [kbuild-standalone](https://github.com/WangNan0/kbuild-standalone) - Original C implementation
- [Linux Kconfig](https://www.kernel.org/doc/html/latest/kbuild/kconfig-language.html) - Official Kconfig documentation

## Acknowledgments

This project is based on the kbuild-standalone implementation and follows the Linux kernel's Kconfig language specification.

