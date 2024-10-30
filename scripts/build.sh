# automatically builds both windows and linux binaries

cargo build --release 
cargo build --target x86_64-pc-windows-gnu --release

mkdir ../release

cp ../target/release/thorlang ../release/thorlang-linux
cp ../target/x86_64-pc-windows-gnu/release/thorlang.exe ../release/thorlang-windows.exe

cd ../release

tar -czvf thorlang-linux.tar.gz thorlang-linux
zip thorlang-windows.zip thorlang-windows.exe
