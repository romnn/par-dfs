## par-dfs

Parallel, serial, and async DFS and BFS traversal iterators in Rust.

```toml
par-dfs = "0"
```

#### Usage
For usage examples, check the [examples](https://github.com/romnn/par-dfs/tree/main/examples) and TODO

#### Examples

```bash
cargo run --example async_fs --features async -- --path ./
cargo run --example sync_fs --features sync,rayon -- --path ./
```

#### Documentation
```bash
RUSTDOCFLAGS="--cfg docsrs" cargo +nightly doc --all-features --no-deps
```

#### Linting

```bash
cargo clippy --tests --benches --examples -- -Dclippy::all -Dclippy::pedantic
```

#### Benchmarking

```bash
cargo install cargo-criterion
# full benchmark suite
cargo criterion --features full
# sync benchmarks only
cargo criterion --features sync -- sync
# dfs benchmarks only
cargo criterion --features full -- dfs
```

#### Acknowledgements

The `rayon::iter::ParallelIterator` implementation for the dynamically growing graph traversal is based on the amazing work in [tavianator's blog post](https://tavianator.com/2022/parallel_graph_search.html).

The implementation of `futures
