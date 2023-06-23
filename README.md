# PaaSTech Pomegranate
The deployment manager for the PaaSTech Platform-as-a-Service.

This service manages deploying applications for the PaaS clients in an execution engine, such as Docker or Kubernetes.

## Run this application
To run Pomegranate, you must first [install the Rust toolchain](https://www.rust-lang.org/tools/install) for your platform.

You must also install an execution engine, such as [Docker](https://docs.docker.com/get-docker/). For Docker, make sure that
you have access to the Docker socket.

Once everything is installed, you can use the Cargo package manager for building and running the application:

```sh
# build the application under the target/ directory
cargo build

# immediately run the application
cargo run
```

You can also run test tools:

```sh
# run unit tests
cargo test

# run the Clippy linter
cargo clippy

# run the formatter
cargo fmt --check
```
