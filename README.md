# Persistence

Implementation of Persistent Homology computation in Rust.

Approach follows [Zomorodian, A., Carlsson, G. Computing Persistent Homology. Discrete Comput Geom 33, 249–274 (2005)](https://doi.org/10.1007/s00454-004-1146-y).

## Testing
To see debug output when running tests:
```bash
RUST_LOG=debug cargo test -- --show-output
```