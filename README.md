# Tanuki

An application for collecting, tagging, browsing, and searching assets, primarily images and videos. Written in [Rust](https://www.rust-lang.org) with a [Leptos](https://leptos.dev) powered front-end and limited [GraphQL](https://graphql.org) support. Metadata is stored in [RocksDB](https://rocksdb.org) and file content is stored unmodified within a date/time formatted directory structure.

## Building and Testing

### Prerequisites

* [Rust](https://www.rust-lang.org) stable (2021 edition)
* [Clang](https://clang.llvm.org) (version 5.0 or higher, as dictated by [rust-bindgen](https://github.com/rust-lang/rust-bindgen))

### Initial Setup

These commands need to be run one time before building this project:

```shell
cargo install cargo-leptos
rustup target add wasm32-unknown-unknown
```

#### Windows

Download the [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) and select the `MSVC ... build tools` (latest version with appropriate architecture) and `Windows 11 SDK` (or `10` if using Windows 10).

### Testing

Run both the backend and front-end tests with one command:

```shell
cargo leptos test
```

Run a single backend test:

```shell
cargo test --features=ssr test_location_from_str
```

### Starting the server locally

```shell
cargo leptos watch
```

The server will be listening at `http://localhost:3000`

The GraphiQL interface is available at `http://localhost:3000/graphiql`

## Tools

### Formatting

Use `cargo fmt` to format all of the Rust code.

Use `leptosfmt` to format the client-side code, like so:

```shell
leptosfmt src/preso/leptos/client/**/*.rs
```

### Finding Outdated Crates

Use https://github.com/kbknapp/cargo-outdated and run `cargo outdated -R`

## Origin of the name

A tanuki is a racoon dog native to Japan, and may also refer to the [Bake-danuki](https://en.wikipedia.org/wiki/Bake-danuki), a shape-shifting supernatural being of Japanese folklore. That has nothing to do with this project, but the name is unique and it makes for a cute mascot.
