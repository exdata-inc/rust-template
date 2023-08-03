# rust-template
Tempalte repository for Rust projects

## Run
```
cargo run
```

## Build
### for Development
```
cargo build
ls target/debug
```

### for Release
```
cargo rustc --release -- -C opt-level=s -C link-args=-Wl,-x,-S
ls target/release
```