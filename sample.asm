start:
	lil x1, msg@l
	lih x2, msg@h
	or x1, x2
	sw x2, x1, 0
	sw x3, x1, 2
msg:
	.word 0x6c6c
	.word 0x6548