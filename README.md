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

### Running the Operating system

in the Rust_OS Directory Run

- Cargo run
