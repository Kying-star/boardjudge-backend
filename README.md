# Boardjudge-backend

## Get started

To build from the source code, you need to install Rust~~ and `nightly-x86_64-unknown-linux-gnu` toolchain~~.

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# rustup toolchain install nightly-x86_64-unknown-linux-gnu
RUSTFLAGS='-C target-cpu=native' cargo build --release
```

To deploy the application, docker, MariaDB, libseccomp and compilers (now only C/C++ is supported) are needed.

```sh
sudo pacman -S libseccomp docker mariadb clang
```

Get a database and an account for your MariaDB, and initialize the database by the SQL in `/assets/starter.sql`.

Then create a data directory with the template directory `data`, then you can run the application in the docker.

```sh
cargo run -- --config ./data/config.toml --level trace
```

## Acknowledgement

Rust ecosystem is the base of this project. Thank for all contributors.

The judger sandbox is a work of QingdaoU. Thank for QingdaoU contributors too.
