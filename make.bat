cargo rustc --release --bin rust_swcl -- -C target-cpu=native 
cd target
cd release
rust_swcl.exe log
pause
