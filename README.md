# gf - Grep-like Pattern Manager

`gf` is a powerful command-line tool written in Rust for managing and using reusable search patterns with grep-like tools. It allows you to save, list, and execute complex search patterns easily, improving your productivity in tasks like code analysis, security auditing, and text processing.

## Features

- **Save Search Patterns**: Save complex grep patterns with custom flags and reuse them anytime.
- **Pattern Listing**: List all saved patterns for easy reference.
- **Customizable Engines**: Use different search engines like `grep`, `rg` (ripgrep), or `ag` (The Silver Searcher).
- **Pattern Execution**: Execute saved patterns directly on files or piped input.
- **Command Dumping**: Print the full command that would be executed without running it.

## Installation

### Pre-built Binaries

Download the latest release for your platform from the [Releases](https://github.com/Periecle/gf/releases) page and place the binary in your `$PATH`.

### Build from Source

To build `gf` from source, ensure you have Rust and Cargo installed.

```bash
# Clone the repository
git clone https://github.com/Periecle/gf.git
cd gf

# Build the project and install it to PATH
cargo install --path .
```
# Usage

## Basic Usage

## Save a pattern

```bash
gf --save <pattern-name> [--engine <engine>] [flags] <search-pattern>
```

## Use a saved pattern

```bash
gf <pattern-name> [file or directory]
```
## List all saved patterns

```bash
gf --list
```

## Dump the command that would be executed

```bash
gf --dump <pattern-name> [file or directory]
```

# Options

```bash
Usage: gf [OPTIONS] [NAME] [ARGS]...

Pattern manager for grep-like tools

Options:
  --save               Save a pattern (e.g., gf --save pat-name -Hnri 'search-pattern')
  --list               List available patterns
  --dump               Print the command rather than executing it
  --engine <ENGINE>    Specify the engine to use (e.g., 'grep', 'rg', 'ag')
  -h, --help           Print help information
  -V, --version        Print version information

Arguments:
  [NAME]         The name of the pattern (when saving or using)
  [ARGS]...      Additional arguments
```

# Examples

## Saving and Using a Pattern

Save a pattern named find-todos to search for TODO comments:

```bash
gf --save find-todos -nri "TODO"
```

Use the saved pattern to search in the current directory:

```bash
gf find-todos
```

## Saving a Pattern with a Custom Engine

Save a pattern using rg (ripgrep) as the search engine:

```bash
gf --save find-errors --engine rg -nri "ERROR"
```

Use the saved pattern:

```bash
gf find-errors /var/log
```

## Listing Saved Patterns

List all saved patterns:

```bash
gf --list
```

## Dumping a Command

Dump the command that would be executed for a pattern:

```bash
gf --dump find-todos src/
```

Output:

```bash
grep -nri "TODO" src/
```

## Executing a Pattern on Piped Input

Use a pattern with piped input:

```bash
cat file.txt | gf find-todos
```

## Saving a Pattern Without Flags
Save a pattern without any flags:

```bash
gf --save simple-search "" "pattern-to-search"
```

## Handling Errors

Attempting to use a non-existent pattern:

```bash
gf nonexistent-pattern
```

Output:

```bash
Error: No such pattern 'nonexistent-pattern'
```

# Original Work
This tool was originally written by [tomnomnom in Go](https://github.com/tomnomnom/gf).


# Contributing
Contributions are welcome! Please follow these steps:

Fork the repository.
Create a new branch with your feature or bug fix.
Commit your changes with clear and descriptive messages.
Push to your branch.
Open a pull request on GitHub.
Please ensure that your code adheres to the existing style and passes all tests.

# License
This project is licensed under the MIT License. See the LICENSE file for details.