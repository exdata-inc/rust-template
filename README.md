# rust-template
Template repository for Rust projects with sxutil

## Run
You should run [Synerex Nodeserv](https://github.com/synerex/synerex_nodeserv) and [Synerex Server](https://github.com/synerex/synerex_server) before running commands below.

### Notifier
It will notify demand and supply message alternately.
```
cargo run -- -m notify
```
### DemandSubscriber
It will subscribe demand message from above notifier.
```
cargo run -- -m subscribe -t demand
```
### SupplySubscriber
It will subscribe supply message from above notifier.
```
cargo run -- -m subscribe -t supply
```
### Echo (Subscribe + Notify)
It will subscribe demand and supply message from above notifier and send echo message.
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
