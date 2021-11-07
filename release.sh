echo "Compiling for linux..." &&
cargo build --release -j 8 &&
echo "Compiled!" &&
echo "Zipping..." &&
zip rayvarust.linux.zip resources/ -r &&
cd target/release/ &&
strip rayvarust
zip ../../rayvarust.linux.zip rayvarust &&
echo "Zipped!" &&
echo "Compiling for windows..." &&
cross build --target x86_64-pc-windows-gnu -j 8 --release &&
echo "Compiled!" &&
echo "Zipping..." &&
cd ../../ &&
zip rayvarust.win.zip resources/ -r &&
cd ./target/x86_64-pc-windows-gnu/release &&
zip ../../../rayvarust.win.zip rayvarust.exe &&
echo "Zipped!"
