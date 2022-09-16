## par-dfs

Parallel, serial, and async DFS and BFS traversal iterators in Rust.

```toml
[dependencies]
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

The [`rayon::iter::ParallelIterator`](https://docs.rs/rayon/latest/rayon/iter/trait.ParallelIterator.html) implementation for the dynamically growing graph traversal is based on the amazing work in [tavianator's blog post](https://tavianator.com/2022/parallel_graph_search.html).

The implementation of [`futures_util::stream::Buffered`](https://docs.rs/futures-util/latest/src/futures_util/stream/stream/buffered.rs.html#12-25) also greatly helped in the design of the async streams.

#### TODO

- maybe merge the FastNode and Node traits
- do not allow `add` and `add_all` to specify the depth themselves

