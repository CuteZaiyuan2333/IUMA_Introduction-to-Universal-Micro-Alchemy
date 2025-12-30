# IUMA: Introduction to Universal Micro Alchemy

A field-based 2D particle simulation game built with **Bevy** and **Egui**.

## Concept
Particles do not interact directly. They emit **Fields** (defined by Bezier curves) and react to the local gradients of those fields based on defined **Weights**.

## Prerequisites
- Rust & Cargo (latest stable)

## How to Run
```bash
cargo run --release
```
*Note: Using `--release` is highly recommended for simulation performance.*

## Documentation
See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for technical details.
