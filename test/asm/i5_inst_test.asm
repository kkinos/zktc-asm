// I5 Instruction Test
start:	// address 0
	addi x0,zero, 1
	subi x1, ra, 1
	beq x2, fp, 1
	bnq x3, a0, -1
	blt x4, a1, 1
	bge x5, a2, -1
	bltu x6, t0, 1
	bgeu x7, t1, -1
	jalr x0, x1, 1
	lh x2, x3, -1
	lhu x4, x5, 1
	lw x6, x7, -1
	sh x0, x1, 1
	sw x2, x3, -1
end:	// address 28