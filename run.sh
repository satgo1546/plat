#!/bin/sh
# must be run in project root
set -e
cargo run -- -riscv "$1" > a.S
clang a.S -c -o a.o -target riscv32-unknown-elf -march=rv32im -mabi=ilp32
podman run -t --rm -v .:/root/compiler docker.io/maxxing/compiler-dev sh -c 'clang -target riscv32-unknown-elf -fuse-ld=lld -nostdlib -march=rv32imf -mabi=ilp32 /root/compiler/a.o /root/compiler/awesome-sysy-master/mandelbrot/fp-math.c -L$CDE_LIBRARY_PATH/riscv32 -lsysy -o /root/compiler/a.out'
rm -f a.o a.S
qemu-riscv32-static a.out
