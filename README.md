# zktc-asm

zktc-asm は Rust で実装された [ZKTC](https://github.com/kinpoko/zktc)用のアセンブラです。命令セットに関しては [こちら](https://github.com/kinpoko/zktc#%E5%91%BD%E4%BB%A4%E3%82%BB%E3%83%83%E3%83%88)を参照してください。

# インストール

```sh
git clone https://github.com/kinpoko/zktc-asm.git
cargo build --release
```

# 使い方

アセンブラファイルを用意してください。例えば以下のようなファイルを用意します。

`sample.asm`

```
start:
	lil x1, msg@l
	lih x1, msg@h
	sw x2, x1, 0
	sw x3, x1, 2
msg:
	.word 0x6c6c
	.word 0x6548
```

mem ファイルにアセンブルできます。mem ファイルはテキストファイルです。

```sh
zktc-asm sample.asm -o sample.mem
cat sample.mem
2d
08
2e
00
4b
01
6b
11
6c
6c
48
65

```

他のオプションについては`zktc-asm -h`を参照してください。

# アセンブラ構文

基本的には以下の順序です。

```asm
mnemonic destination register, source register
```

使用できるレジスタは[こちら](https://github.com/kinpoko/zktc/tree/main#%E6%B1%8E%E7%94%A8%E3%83%AC%E3%82%B8%E3%82%B9%E3%82%BFgr)を参照してください。

## コメント

`//`を用いてコメントを書くことができます。

```asm
// comment
addi x1, x0, 1 // comment
```

## ディレクティブ

`.word`を用いて数値を配置できます。

```asm
.word 0xffff
```

## ラベル

以下のようにしてアドレスに名前をつけることができます。

```asm
start:
	jal x0, start
```

## シンボル

数値やラベルと組み合わせることで、それらの上位 8 ビット、下位 8 ビットを表現できます。

```asm
start:
	lil x1, 0x1111@l // low 8 bits
	lih x2, start@h // high 8 bits
```