# rust-template
Template repository for Rust projects with sxutil

## Run
You should run [Synerex Nodeserv](https://github.com/synerex/synerex_nodeserv) and [Synerex Server](https://github.com/synerex/synerex_server) before running commands below.

### Notifier
```
cargo run -- -m notify
```
### DemandSubscriber
```
cargo run -- -m subscribe -t demand
```
### SupplySubscriber
```
cargo run -- -m subscribe -t supply
```
### Echo (Subscribe + Notify)
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
