# rust-template
Template repository for Rust projects with sxutil

## Run
You should run [Synerex Nodeserv](https://github.com/synerex/synerex_nodeserv) and [Synerex Server](https://github.com/synerex/synerex_server) before running commands below.

### DemandSubscriber (Service)
It will subscribe demand message from DemandNotifier and propse supply.
```
cargo run -- -m subscribe -t Demand
```
### SupplySubscriber (Client)
It will subscribe supply message from SupplyNotifier and propose demand.
```
cargo run -- -m subscribe -t Supply
```
### DemandNotifier (Client)
It will notify demand message and subscribe supply from DemandSubscriber.
Note: Please start DemandSubscriber before starting this.
```
cargo run -- -m notify -t Demand
```
### SupplyNotifier (Service)
It will notify supply message and subscribe demand from SupplySubscriber.
Note: Please start SupplySubscriber before starting this.
```
cargo run -- -m notify -t Supply
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
