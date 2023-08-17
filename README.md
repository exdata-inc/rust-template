# rust-template
Template repository for Rust projects with sxutil

## Run
You should run [Synerex Nodeserv](https://github.com/synerex/synerex_nodeserv) and [Synerex Server](https://github.com/synerex/synerex_server) before running commands below.

### Supplier
```
cargo run -- -m supply
```
### Subscriber
```
cargo run -- -m subscribe
```
### Echo (Subscribe + Supply)
```
cargo run -- -m echo
```

## Build
### for Development
```
cargo build
ls target/debug
```

### for Release
```
cargo rustc --release -- -C link-args=-Wl,-x,-S
ls target/release
```
