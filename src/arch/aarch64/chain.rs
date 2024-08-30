use {
    crate::{
        arch::{aarch64::insn::Aarch64InstructionDetail, ChainArch, LookupKey},
        instruction::{Instruction, InstructionDetail},
    },
    anyhow::{anyhow, Result},
    capstone::arch::arm64::Arm64Insn,
};

pub type Aarch64LookupKey = u64;

pub struct Aarch64ChainArch;

impl Aarch64ChainArch {
    fn detail<'a>(&self, insn: &'a Instruction) -> Result<&'a Aarch64InstructionDetail> {
        match &insn.detail {
            InstructionDetail::Aarch64(detail) => Ok(detail),
            _ => Err(anyhow!("Unexpected instruction detail")),
        }
    }
}

impl ChainArch for Aarch64ChainArch {
    fn is_conditional(&self, insn: &Instruction) -> Result<bool> {
        let detail = self.detail(insn)?;
        Ok(detail.conditional)
    }

    fn is_rop(&self, insn: &Instruction) -> Result<bool> {
        let id = Arm64Insn::from(insn.id.0);
        match id {
            Arm64Insn::ARM64_INS_RET => Ok(true),
            _ => Ok(false),
        }
    }

    fn is_jop(&self, insn: &Instruction) -> Result<bool> {
        let id = Arm64Insn::from(insn.id.0);
        match id {
            Arm64Insn::ARM64_INS_BR | Arm64Insn::ARM64_INS_BLR => Ok(self
                .detail(insn)?
                .only_operand_reg()
                .map_err(|e| anyhow!("{insn:#?}, {e:}"))?
                .is_some()),
            _ => Ok(false),
        }
    }

    fn next_insns(&self, insn: &Instruction) -> Result<Vec<LookupKey>> {
        let id = Arm64Insn::from(insn.id.0);
        let end = insn.address + insn.length as u64;
        match id {
            Arm64Insn::ARM64_INS_B | Arm64Insn::ARM64_INS_BL => {
                let detail = self.detail(insn)?;
                if let Some(i) = detail
                    .only_operand_imm()
                    .map_err(|e| anyhow!("{insn:#?}, {e:}"))?
                {
                    if detail.conditional {
                        Ok(vec![LookupKey::Aarch64(i as u64), LookupKey::Aarch64(end)])
                    } else {
                        Ok(vec![LookupKey::Aarch64(i as u64)])
                    }
                } else {
                    Ok(vec![])
                }
            }
            _ => Ok(vec![LookupKey::Aarch64(end)]),
        }
    }

    fn should_trim(&self, insn: &Instruction) -> Result<bool> {
        let id = Arm64Insn::from(insn.id.0);
        match id {
            Arm64Insn::ARM64_INS_B | Arm64Insn::ARM64_INS_BL => {
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
        Ok(LookupKey::Aarch64(insn.address))
    }
}
