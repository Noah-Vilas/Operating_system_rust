# Rust OS

A simple operating system kernel written in Rust, targeting the `x86_64` architecture. This project runs in [QEMU](https://www.qemu.org/) and can be built and launched using `cargo run`.

## Features

- Written in `#![no_std]` Rust
- Boots on `x86_64` hardware (emulated via QEMU)
- Uses `cargo` for building and running
- Custom target specification
- VGA text output support

## Getting Started

### Prerequisites

- [Rust nightly](https://rustup.rs/)
- `bootimage` crate
- QEMU

Install dependencies:

```bash
rustup default nightly
rustup component add rust-src
cargo install bootimage
```
### Running the Operating system

in the Rust_OS Directory Run
```bash
- Cargo run
```


## Current Stage

Here You can see the Drive image from my .img in the folder, and after that info is shown you'll see a kernel able to do simple 1 expression arithmetic.
![Drive_Info](https://github.com/user-attachments/assets/574baf9d-9dc9-49b5-81fe-cd17aad57575)

## Next Stages

Getting the ability to access the hard drive and read the file system, then read and write as well as create new files.
