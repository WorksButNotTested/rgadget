use {
    crate::{
        arch::{x64::insn::X64InstructionDetail, ChainArch, LookupKey},
        instruction::{Instruction, InstructionDetail},
    },
    anyhow::{anyhow, Result},
    capstone::arch::x86::X86Insn,
};

pub type X64LookupKey = u64;

pub struct X64ChainArch;

impl X64ChainArch {
    fn detail<'a>(&self, insn: &'a Instruction) -> Result<&'a X64InstructionDetail> {
        match &insn.detail {
            InstructionDetail::X64(detail) => Ok(detail),
            _ => Err(anyhow!("Unexpected instruction detail")),
        }
    }
}

impl ChainArch for X64ChainArch {
    fn is_conditional(&self, insn: &Instruction) -> Result<bool> {
        let detail = self.detail(insn)?;
        Ok(detail.conditional)
    }

    fn is_rop(&self, insn: &Instruction) -> Result<bool> {
        let id = X86Insn::from(insn.id.0);
        match id {
            X86Insn::X86_INS_RET => Ok(true),
            _ => Ok(false),
        }
    }

    fn is_jop(&self, insn: &Instruction) -> Result<bool> {
        let id = X86Insn::from(insn.id.0);
        match id {
            X86Insn::X86_INS_JMP | X86Insn::X86_INS_CALL => Ok(self
                .detail(insn)?
                .only_operand_imm()
                .map_err(|e| anyhow!("{insn:#?}, {e:}"))?
                .is_none()),
            _ => Ok(false),
        }
    }

    fn next_insns(&self, insn: &Instruction) -> Result<Vec<LookupKey>> {
        let id = X86Insn::from(insn.id.0);
        let end = insn.address + insn.length as u64;
        match id {
            X86Insn::X86_INS_JMP | X86Insn::X86_INS_CALL => {
                let detail = self.detail(insn)?;
                if let Some(i) = detail
                    .only_operand_imm()
                    .map_err(|e| anyhow!("{insn:#?}, {e:}"))?
                {
                    if detail.conditional {
                        Ok(vec![LookupKey::X64(i as u64), LookupKey::X64(end)])
                    } else {
                        Ok(vec![LookupKey::X64(i as u64)])
                    }
                } else {
                    Ok(vec![])
                }
            }
            _ => Ok(vec![LookupKey::X64(end)]),
        }
    }

    fn should_trim(&self, insn: &Instruction) -> Result<bool> {
        let id = X86Insn::from(insn.id.0);
        match id {
            X86Insn::X86_INS_JMP | X86Insn::X86_INS_CALL => {
                let detail = self.detail(insn)?;
                Ok(detail
                    .only_operand_imm()
                    .map_err(|e| anyhow!("{insn:#?}, {e:}"))?
                    .is_some())
            }
            _ => Ok(false),
        }
    }

    fn get_key(&self, insn: &Instruction) -> Result<LookupKey> {
        Ok(LookupKey::X64(insn.address))
    }
}
