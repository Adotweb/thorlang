# build whatever version you need
rustup default stable
cargo build --release


# automatically copy the executable to the bin folder
# means that this needs to run in sudo mode
cp ./target/release/thorlang /usr/bin/thorlang
