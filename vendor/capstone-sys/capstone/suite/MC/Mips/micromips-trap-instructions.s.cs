# CS_ARCH_MIPS, CS_MODE_MIPS32+CS_MODE_MICRO, None
// 0x28,0x01,0x3c,0x00 = teq $t0, $t1
// 0x28,0x01,0x3c,0x02 = tge $t0, $t1
// 0x28,0x01,0x3c,0x04 = tgeu $t0, $t1
// 0x28,0x01,0x3c,0x08 = tlt $t0, $t1
// 0x28,0x01,0x3c,0x0a = tltu $t0, $t1
// 0x28,0x01,0x3c,0x0c = tne $t0, $t1
0xc9,0x41,0x67,0x45 = teqi $t1, 17767
0x29,0x41,0x67,0x45 = tgei $t1, 17767
0x69,0x41,0x67,0x45 = tgeiu $t1, 17767
0x09,0x41,0x67,0x45 = tlti $t1, 17767
0x49,0x41,0x67,0x45 = tltiu $t1, 17767
0x89,0x41,0x67,0x45 = tnei $t1, 17767
