cargo build
riscv64-unknown-elf-objcopy -O binary ./target/riscv32imac-unknown-none-elf/debug/example-litex app.bin
cp ./target/riscv32imac-unknown-none-elf/debug/example-litex app_debug.elf

# llvm-objdump -d ./target/riscv32imac-unknown-none-elf/debug/example-litex > asm.s
