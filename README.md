# zktc-asm

zktc-asm is an assembler for [ZKTC](https://github.com/kkinos/zktc) implemented in Rust.

# Install

```sh
cargo install --git https://github.com/kkinos/zktc-asm.git
```

# Usage

Prepare an assembler file. For example, prepare the following file.

`sample.asm`

```
start:
	lil x1, msg@l
	lih x2, msg@h
	or x1, x2
	lw x2, x1, 0
	lw x3, x1, 2
msg:
	.word 0x6c6c
	.word 0x6548
```

You can assemble to a `mem` file. `mem` file is a text file.

```sh
zktc-asm sample.asm -o sample.mem
cat sample.mem
2d
0a
4e
00
20
2a
4a
01
6a
11
6c
6c
48
65

```

See `zktc-asm -h` for other options.

# Assembler syntax

Basically, they are as follows

```asm
mnemonic destination register, source register, immediate value
```

## Comments

```asm
// comment
addi x1, x0, 1 // comment
```

## Directives

```asm
.byte 0x11
.word 0xffff
```

## Labels

```asm
start:
	jal x0, start // loop
```

## Symbols

```asm
start:
	lil x1, 0x1111@l // low 8 bits
	lih x2, start@h // high 8 bits
```

# Tests

```bash
cargo test
```
