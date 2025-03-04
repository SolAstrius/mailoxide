# MailOxide

A blazing fast, parallel EML to MBOX converter written in Rust.

```
  ______   __  __   _          _______    ____      __  __   ____     ____   __   __
 |  ____| |  \/  | | |        |__   __|  / __ \    |  \/  | |  _ \   / __ \  \ \ / /
 | |__    | \  / | | |           | |    | |  | |   | \  / | | |_) | | |  | |  \ V / 
 |  __|   | |\/| | | |           | |    | |  | |   | |\/| | |  _ <  | |  | |   > <  
 | |____  | |  | | | |____       | |    | |__| |   | |  | | | |_) | | |__| |  / . \ 
 |______| |_|  |_| |______|      |_|     \____/    |_|  |_| |____/   \____/  /_/ \_\
```

## Features

- Convert single EML file or directory of EML files to MBOX format
- Parallel processing for high performance on large directories
- Memory-efficient processing with chunked approach
- Progress reporting for large conversions
- Standards-compliant MBOX format
- Simple command-line interface

## Installation

### Using Cargo

```bash
cargo install mailoxide
```

### From Source

```bash
git clone https://github.com/solastrius/mailoxide.git
cd mailoxide
cargo build --release
```

The compiled binary will be available at `target/release/mailoxide`.

## Usage

```bash
mailoxide [INPUT] [OUTPUT]
```

Where:

- `INPUT` is a path to an EML file or a directory containing EML files (default: "input")
- `OUTPUT` is the destination directory for the MBOX file (default: "input")

### Examples

1. Convert a single EML file to MBOX:

```bash
mailoxide /path/to/email.eml /path/to/output
```

2. Convert all EML files in a directory:

```bash
mailoxide /path/to/email/directory /path/to/output
```

3. Use default paths:

```bash
mailoxide
```
This will look for an "input" directory in the current path, find all EML files there, and save the output to the same "input" directory as "output.mbox".

## Performance

MailOxide is designed to be extremely fast, even with large email collections. It achieves this through:

- Parallel processing with Rayon
- Memory-efficient chunked processing
- Buffered file I/O
- Progress reporting for large conversions

In real-world testing, it successfully processed over 30,000 emails (3.6GB) in minutes.

## License

MIT

## Credits

This is a Rust rewrite and optimization of the original Python [eml2mbox](https://github.com/nick88msn/eml2mbox) tool by nick88msn.