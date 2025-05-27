cargo build --release
riscv64-unknown-elf-objcopy -O binary ./target/riscv32imac-unknown-none-elf/release/example-litex  app.bin
# llvm-objdump -d ./target/riscv32imac-unknown-none-elf/release/example-litex > asm.s