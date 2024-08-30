use {
    crate::{
        arch::DisassemblyArch, instruction::Instruction, progress::Progress, section::Section,
    },
    anyhow::{anyhow, Error, Result},
    capstone::{Capstone, InsnGroupId, RegId},
    dashmap::DashMap,
    log::debug,
    rayon::iter::{IntoParallelIterator, ParallelIterator},
    std::{cmp::min, ops::Sub, time::Instant},
};

pub struct Disassembler;

impl Disassembler {
    fn disassemble_insn<Arch: DisassemblyArch>(
        arch: &Arch,
        capstone: &Capstone,
        base: u64,
        bytes: &[u8],
    ) -> Result<Instruction> {
        let insns = capstone.disasm_count(bytes, base as u64, 1)?;
        let insn = insns
            .iter()
            .take(1)
            .next()
            .ok_or(anyhow!("Failed to disassemble instruction"))?;
        let detail = capstone.insn_detail(&insn)?;
        let id = insn.id();
        let name = capstone
            .insn_name(id)
            .ok_or(anyhow!("Invalid instruction: {id:#?}"))?;

        let length = insn.len();
        let address = insn.address();
        let bytes = insn.bytes().to_vec();

        let mnemonic = insn
            .mnemonic()
            .map(|s| s.to_string())
            .ok_or(anyhow!("No mnemonic found"))?;
        let operands = insn
            .op_str()
            .map(|s| s.to_string())
            .ok_or(anyhow!("No operandsfound"))?;

        let groups = detail
            .groups()
            .into_iter()
            .map(|&g| {
                let name = capstone
                    .group_name(g)
                    .ok_or(anyhow!("Invalid group: {g:#?}"))?;
                Ok((g, name))
            })
            .collect::<Result<Vec<(InsnGroupId, String)>>>()?;
        let read_regs = detail
            .regs_read()
            .into_iter()
            .map(|&r| {
                let name = capstone.reg_name(r).ok_or(anyhow!("Invalid reg: {r:#?}"))?;
                Ok((r, name))
            })
            .collect::<Result<Vec<(RegId, String)>>>()?;
        let write_regs = detail
            .regs_write()
            .iter()
            .map(|&r| {
                let name = capstone.reg_name(r).ok_or(anyhow!("Invalid reg: {r:#?}"))?;
                Ok((r, name))
            })
            .collect::<Result<Vec<(RegId, String)>>>()?;

        let detail = arch.instruction_details(&insn, &detail.arch_detail())?;

        let insn = Instruction::builder()
            .id(id)
            .name(name)
            .length(length)
            .address(address)
            .bytes(bytes)
            .mnemonic(mnemonic)
            .operands(operands)
            .regs_read(read_regs)
            .regs_write(write_regs)
            .groups(groups)
            .detail(detail)
            .build();
        return Ok(insn);
    }

    pub fn disassemble<Arch: DisassemblyArch>(
        arch: &Arch,
        sections: &Vec<Section>,
    ) -> Result<Vec<Instruction>> {
        let total_len: usize = sections.iter().map(|s| s.bytes.len()).sum();

        let num_threads = num_cpus::get();
        let ranges = sections
            .into_par_iter()
            .map(|s| {
                let length = s.bytes.len();
                let chunk_len = (length + num_threads - 1) / num_threads;
                let ranges = (0..length)
                    .step_by(chunk_len)
                    .map(|i| {
                        let end = min(i + chunk_len, length);
                        (s, i, end)
                    })
                    .collect::<Vec<(&Section, usize, usize)>>();
                ranges
            })
            .flatten()
            .collect::<Vec<(&Section, usize, usize)>>();

        let start = Instant::now();
        let insns = DashMap::new();

        let alignment = arch.alignment();
        let max_len = arch.max_len();

        let progress = Progress::new("Disassembling", total_len);

        ranges.into_par_iter().try_for_each(|(s, start, end)| {
            let capstone = arch.capstone()?;
            for i in start..end {
                progress.inc(1);
                let start_address = s.base + i;
                if start_address % alignment != 0 {
                    continue;
                }
                let end = min(i + max_len, end);
                let bytes = &s.bytes[i..end];
                if let Ok(insn) =
                    Disassembler::disassemble_insn(arch, &capstone, start_address as u64, bytes)
                {
                    insns.insert(start_address as u64, insn);
                }
            }
            Ok::<(), Error>(())
        })?;
        progress.finish();
        let duration = Instant::now().sub(start);
        debug!("Dissassembly took: {duration:#?}");
        let result = insns
            .into_iter()
            .map(|(_k, v)| v)
            .collect::<Vec<Instruction>>();
        debug!("Found {} instructions", result.len());
        Ok(result)
    }
}
