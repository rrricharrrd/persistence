# Persistence

Implementation of Persistent Homology computation in Rust.

Approach follows [Zomorodian, A., Carlsson, G. Computing Persistent Homology. Discrete Comput Geom 33, 249–274 (2005)](https://doi.org/10.1007/s00454-004-1146-y).

## Installation
### Python
To install (including Python bindings):
```bash
pip install .
```

### Rust
For development, it may be helpful to skip the Python build every time:
```bash
cargo build --no-default-features
```

## Testing
### Rust
To see debug output when running just Rust tests:
```bash
RUST_LOG=debug cargo test --no-default-features  -- --nocapture
```

### Python
After installation:
```bash
pytest
```
