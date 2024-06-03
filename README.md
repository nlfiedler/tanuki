# Tanuki

A system for organizing, browsing, and searching assets, primarily images and videos. Written in [Rust](https://www.rust-lang.org) and [Flutter](https://flutter.dev) with a [GraphQL](https://graphql.org) wire protocol. Metadata is stored in [RocksDB](https://rocksdb.org) and file content is stored unmodified within a date/time formatted directory structure.

## Building and Testing

### Prerequisites

* [Rust](https://www.rust-lang.org) stable (2021 edition)
* [Flutter](https://flutter.dev) **stable** channel
* [Clang](https://clang.llvm.org) (version 5.0 or higher, as dictated by [rust-bindgen](https://github.com/rust-lang/rust-bindgen))

### Initial Setup

Use [fvm](https://pub.dev/packages/fvm) to select a specific version of Flutter
to be installed and used by the application. This is the most reliable method
and produces consistent results when building the application.

```shell
brew install dart
pub global activate fvm
fvm install stable
fvm flutter config --enable-web
```

#### Windows

Download the [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) and select the `MSVC ... build tools` (latest version with appropriate architecture) and `Windows 11 SDK` (or `10` if using Windows 10).

### Building, Testing, Starting the Backend

```shell
cargo update
cargo build
cargo test
RUST_LOG=info cargo run --release
```

For more verbose debugging output, use `RUST_LOG=debug` in the command above.
For extremely verbose logging, use `RUST_LOG=trace` which will dump large
volumes of output.

### Building, Testing, Starting the Frontend

```shell
fvm flutter pub get
fvm flutter pub run environment_config:generate
fvm flutter test
fvm flutter run -d chrome
```

### environment_config

The frontend has some configuration that is set up at build time using the
[environment_config](https://pub.dev/packages/environment_config) package. The
generated file (`lib/environment_config.dart`) is not version controlled, and
the values can be set at build-time using either command-line arguments or
environment variables. See the `pubspec.yaml` for the names and the
`environment_config` README for instructions.

## Tools

### Finding Outdated Crates

Use https://github.com/kbknapp/cargo-outdated and run `cargo outdated -R`

## Origin of the name

A tanuki is a racoon dog native to Japan, and may also refer to the [Bake-danuki](https://en.wikipedia.org/wiki/Bake-danuki), a shape-shifting supernatural being of Japanese folklore. That has nothing to do with this project, but the name is unique and it makes for a cute mascot.
