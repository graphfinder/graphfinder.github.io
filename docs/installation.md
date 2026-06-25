# Installation

## Python (recommended)

```bash
pip install graphfinder
```

This installs prebuilt wheels (no Rust toolchain needed). `numpy` and
`matplotlib` come as dependencies. For GIF export and nicer general-graph
layouts:

```bash
pip install "graphfinder[viz]"   # pillow + networkx
```

## From source

Requires [Rust](https://rustup.rs) and Python ≥ 3.9.

```bash
git clone https://github.com/turboswarm/graphfinder
cd graphfinder
python -m venv .venv && source .venv/bin/activate
pip install maturin
maturin develop --release        # builds the Rust core and installs the package
python examples/quickstart.py
```

## Rust crate

```toml
# Cargo.toml
[dependencies]
graphfinder-core = "0.1"
```

```bash
cargo add graphfinder-core
```
