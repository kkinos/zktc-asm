start:
	mov x1, x2
	add x1, x2 
	sub x1, x2
	and x1, x2
	or x1, x2 
	xor x1, x2
	sll x1, x2
	srl x1, x2
	sra x1, x2
	addi x1, x2, 1
	subi x1, x2, 1
	beq x1, x2, -10
	bnq x1, x2, 1
	blt x1, x2, 1
	bge x1, x2, 1
	bltu x1, x2, 1
	bgeu x1, x2, 1
	jalr x1, x2, 1
	lw x1, x2, 1
	sw x1, x2, 1
	jal x1, 1
	lil x1, 0x1@l
	lih x1, 0x1@h
	push x1
	pop x1
	rpc x1
	rsp x1
	rpsr x1
	rtlr x1
	rthr x1
	wsp x1
	wpsr x1
	wtlr x1
	wthr x1
	rfi
	rtr
	wtr
end: