echo "Compiling..."
cargo build --release -j 8 &&
echo "Compiled!"
echo "Zipping..."
zip rayvarust resources/ -r &&
cd target/release/ &&
zip ../../rayvarust.zip rayvarust &&
echo "Zipped!"
