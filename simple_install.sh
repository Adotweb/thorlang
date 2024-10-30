# build whatever version you need
rustup default stable
cargo build --release

cp ./target/release/thorlang /usr/bin/thorlang
