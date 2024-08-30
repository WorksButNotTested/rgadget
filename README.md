# RGadget
RGadget is a tool written in rust for finding ROP and JOP gadgets in binaries. It's features include:
* Strong performance (thanks to use of `rayon` and `dashmap`)
* Simplistic implementation (< 3000 lines of code, all logic works on capstone representation of instructions meaning no parsing of raw bytes).
* Support for ELF files for Linux (adding support for Windows and PE files or raw binaries should be straight-forward since goblin is used for image parsing).
* Multi-architecture support x86_64, Aarch64, Arm (and Thumb), PowerPC (including both big and little endian variants).
* Support for finding chains including immediate branches (e.g. where instructions are not contiguous).
* Support for finding chains including or ending with conditionally executed instructions.
* Support for filtering using multiple regular expressions (including support for extended regular expressions) both for chains to be included as well as excluded.
* Syntax highlighting of matched groups within regular expressions to make it easier to read the pertinent parts of identified chains (Note that syntax coloring is not shown in the examples below).
* Optional (enabled by default) support for deduplication of identical chains (duplicates may be useful where restrictions exist on the address of gadgets).
* Processing of multiple input files.

# Usage
```bash
$ rgadget --help
Usage: rgadget [OPTIONS] --files <FILES>

Options:
  -f, --files <FILES>        Name of the file to process
  -n, --num <NUM>            Maximum number of gadgets in a chain [default: 6]
  -r, --rop                  Find ROP gadgets
  -j, --jop                  Find JOP gadgets
  -e, --end <END>            Find gadgets with custom ending (regex)
  -c, --conditional          Include conditional instructions
  -v, --verbose              Verbose output
  -l, --limit <LIMIT>        Limit the number of results
  -b, --bytes                Show bytes of the instructions in the chain
  -d, --duplicates           Show duplicate chains
  -x, --excludes <EXCLUDES>  Exclude chains matching regex
  -i, --includes <INCLUDES>  Regex to apply to output
  -h, --help                 Print help
  -V, --version              Print version
```

# Examples
Sample data is included in the `data` folder. These consist of binaries making using the `libpng` library for parsing images.
The following command will show the first 10 ROP and JOP gadgets found within the provided binary.
```bash
$ rgadget -j -r -f ./data/arm.elf -l 10
    Finished `release` profile [optimized] target(s) in 0.03s
     Running `target/release/rgadget -j -r -f ./data/arm.elf -l 10`
[ #1] [T] arm.elf!0x10360: ldr r7, [sp, #0x68]; pop {r1, r2, r4, r6, r7, pc}
[ #2] [T] arm.elf!0x10a48: rsbs r1, r1, #0; ldrsh r3, [r0, r5]; cmp r6, #0x32; subs r1, #0x32; bx r0
[ #3] [T] arm.elf!0x10a54: muls r2, r0, r2; adds r2, #0x5f; adds r4, #0x2e; bx r0
[ #4] [T] arm.elf!0x10dca: movs r0, r0; ands r0, r1; push.w {r0, r1, r3, r4, r5, r7}; add.w r0, r0, r8; pop.w {r2, sp, lr, pc}
[ #5] [A] arm.elf!0x11168: strb r3, [r4]; pop {r4, pc}
[ #6] [A] arm.elf!0x11200: ldr r3, [fp, #-8]; str r2, [r3]; mov r0, r0; sub sp, fp, #4; pop {fp, pc}
[ #7] [T] arm.elf!0x113d6: and.w r0, r0, r4, asr #12; b #0x11874; movs r0, #0; b #0x11fba; pop.w {r2, ip, sp, pc}
[ #8] [T] arm.elf!0x113d8: adds r0, #0x24; b #0x11874; movs r0, #0; b #0x11fba; pop.w {r2, ip, sp, pc}
[ #9] [T] arm.elf!0x11838: movs r3, r0; b #0x11b7e; b #0x11ec2; pop.w {r2, ip, sp, pc}
[#10] [T] arm.elf!0x11874: movs r0, #0; b #0x11fba; pop.w {r2, ip, sp, pc}
Displaying 10 of 436 Gadgets
```

The following is a more advanced example which shows ROP gadgets which load the register which is used for the final branch, but do not also load `r0`:
```bash
$ rgadget -j -c -f ./data/arm.elf -i 'ldr (r[0-9]{1,2}).*bl?x (\1)' -e 'ldr r0'
    Finished `release` profile [optimized] target(s) in 0.03s
     Running `target/release/rgadget -j -c -f ./data/arm.elf -i 'ldr (r[0-9]{1,2}).*bl?x (\1)' -e 'ldr r0'`
[#1] [A] arm.elf!0x1bb98: sub r2, fp, #0x194; str r2, [r3, #8]; ldr r3, [fp, #-0x1ac]; ldr r0, [fp, #-0x1b0]; blx r3
[#2] [A] arm.elf!0x1e370: ldr r3, [fp, #-8]; ldr r3, [r3, #0x3a8]; ldr r1, [fp, #-0xc]; ldr r0, [fp, #-8]; blx r3
[#3] [A] arm.elf!0x263e4: ldr r3, [r3, #0x1a4]; ldr r2, [fp, #-0x10]; ldr r1, [fp, #-0xc]; ldr r0, [fp, #-8]; blx r3
[#4] [A] arm.elf!0x3acd8: ldr r3, [r3, r2, lsl #2]; ldr r2, [fp, #-0x14]; ldr r1, [fp, #-0x10]; ldr r0, [fp, #-0xc]; blx r3
[#5] [A] arm.elf!0x4c1a0: ldr r3, [r3, #0x1a0]; ldr r2, [fp, #-0x10]; ldr r1, [fp, #-0xc]; ldr r0, [fp, #-8]; blx r3
[#6] [A] arm.elf!0x4c26c: ldr r3, [fp, #-8]; ldr r3, [r3, #0x2c0]; ldr r0, [fp, #-8]; blx r3
[#7] [A] arm.elf!0x505a8: add r3, r3, #8; ldr r3, [r3]; ldr r1, [fp, #-0x3c]; ldr r0, [fp, #-0x1c]; blx r3
Displaying 7 of 7 Gadgets
```
