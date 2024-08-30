use {
    crate::{
        arch::{
            arm::insn::{ArmInstructionDetail, Mode},
            ChainArch, LookupKey,
        },
        instruction::{Instruction, InstructionDetail},
    },
    anyhow::{anyhow, Result},
    capstone::arch::arm::{ArmInsn, ArmReg},
};

pub type ArmLookupKey = (Mode, u64);

pub struct ArmChainArch;

impl ArmChainArch {
    fn detail<'a>(&self, insn: &'a Instruction) -> Result<&'a ArmInstructionDetail> {
        match &insn.detail {
            InstructionDetail::Arm(detail) => Ok(detail),
            _ => Err(anyhow!("Unexpected instruction detail")),
        }
    }

    fn is_thumb(&self, insn: &Instruction) -> Result<bool> {
        let detail = self.detail(insn)?;
        Ok(detail.mode == Mode::Thumb)
    }

    fn is_rop_arm(&self, insn: &Instruction) -> Result<bool> {
        let id = ArmInsn::from(insn.id.0);
        match id {
            ArmInsn::ARM_INS_LDM | ArmInsn::ARM_INS_POP => self.detail(insn)?.contains_pc(),
            _ => Ok(false),
        }
    }

    fn is_jop_arm(&self, insn: &Instruction) -> Result<bool> {
        let id = ArmInsn::from(insn.id.0);

        match id {
            ArmInsn::ARM_INS_BX | ArmInsn::ARM_INS_BLX => {
                let detail = self.detail(insn)?;
                if let Some(r) = detail
                    .get_reg_operand(0, 1)
                    .map_err(|e| anyhow!("{insn:#?}, {e:}"))?
                {
                    match r.0 as u32 {
                        ArmReg::ARM_REG_LR => Ok(false),
                        _ => Ok(true),
                    }
                } else {
                    Ok(false)
                }
            }
            ArmInsn::ARM_INS_MOV => {
                let detail = self.detail(insn)?;
                if let Some(r) = detail
                    .get_reg_operand(1, 2)
                    .map_err(|e| anyhow!("{insn:#?}, {e:}"))?
                {
                    match r.0 as u32 {
                        ArmReg::ARM_REG_PC => Ok(true),
                        _ => Ok(false),
                    }
                } else {
                    Ok(false)
                }
            }
            _ => Ok(false),
        }
    }

    fn next_insns_arm(&self, insn: &Instruction) -> Result<Vec<LookupKey>> {
        let id = ArmInsn::from(insn.id.0);
        let end = insn.address + insn.length as u64;
        match id {
            ArmInsn::ARM_INS_B | ArmInsn::ARM_INS_BL => {
                let detail = self.detail(insn)?;
                if let Some(i) = detail
                    .get_imm_operand(0, 1)
                    .map_err(|e| anyhow!("{insn:#?}, {e:}"))?
                {
                    if detail.conditional {
                        Ok(vec![
                            LookupKey::Arm((Mode::Arm, i as u64)),
                            LookupKey::Arm((Mode::Arm, end)),
                        ])
                    } else {
                        Ok(vec![LookupKey::Arm((Mode::Arm, i as u64))])
                    }
                } else {
                    Ok(vec![])
                }
            }
            ArmInsn::ARM_INS_BX | ArmInsn::ARM_INS_BLX => {
                let detail = self.detail(insn)?;
                if let Some(i) = detail
                    .get_imm_operand(0, 1)
                    .map_err(|e| anyhow!("{insn:#?}, {e:}"))?
                {
                    if detail.conditional {
                        Ok(vec![
                            LookupKey::Arm((Mode::Thumb, i as u64)),
                            LookupKey::Arm((Mode::Arm, end)),
                        ])
                    } else {
                        Ok(vec![LookupKey::Arm((Mode::Thumb, i as u64))])
                    }
                } else {
                    Ok(vec![])
                }
            }
            ArmInsn::ARM_INS_BXNS | ArmInsn::ARM_INS_BLXNS | ArmInsn::ARM_INS_BXJ => Ok(vec![]),
            _ => Ok(vec![LookupKey::Arm((Mode::Arm, end))]),
        }
    }

    fn should_trim_arm(&self, insn: &Instruction) -> Result<bool> {
        let id = ArmInsn::from(insn.id.0);
        match id {
            ArmInsn::ARM_INS_B
            | ArmInsn::ARM_INS_BL
            | ArmInsn::ARM_INS_BX
            | ArmInsn::ARM_INS_BLX => {
                let detail = self.detail(insn)?;
                Ok(detail
                    .get_imm_operand(0, 1)
                    .map_err(|e| anyhow!("{insn:#?}, {e:}"))?
                    .is_some())
            }
            _ => Ok(false),
        }
    }

    /* Capstone incorrectly parses invalid forms of several instructions */
    fn is_bad_thumb(&self, insn: &Instruction) -> Result<bool> {
        let id = ArmInsn::from(insn.id.0);
        match id {
            ArmInsn::ARM_INS_LDM
            | ArmInsn::ARM_INS_LDMDA
            | ArmInsn::ARM_INS_LDMDB
            | ArmInsn::ARM_INS_LDMIB
            | ArmInsn::ARM_INS_POP => {
                let detail = self.detail(insn)?;
                Ok(detail.contains_sp()? || (detail.contains_lr()? && detail.contains_pc()?))
            }
            _ => Ok(false),
        }
    }

    fn is_rop_thumb(&self, insn: &Instruction) -> Result<bool> {
        let id = ArmInsn::from(insn.id.0);
        match id {
            ArmInsn::ARM_INS_LDM
            | ArmInsn::ARM_INS_LDMDA
            | ArmInsn::ARM_INS_LDMDB
            | ArmInsn::ARM_INS_LDMIB
            | ArmInsn::ARM_INS_POP => self.detail(insn)?.contains_pc(),
            _ => Ok(false),
        }
    }

    fn is_jop_thumb(&self, insn: &Instruction) -> Result<bool> {
        let id = ArmInsn::from(insn.id.0);

        match id {
            ArmInsn::ARM_INS_BX | ArmInsn::ARM_INS_BLX => {
                let detail = self.detail(insn)?;
                if let Some(r) = detail
                    .get_reg_operand(0, 1)
                    .map_err(|e| anyhow!("{insn:#?}, {e:}"))?
                {
                    match r.0 as u32 {
                        ArmReg::ARM_REG_LR => Ok(false),
                        _ => Ok(true),
                    }
                } else {
                    Ok(false)
                }
            }
            _ => Ok(false),
        }
    }

    fn next_insns_thumb(&self, insn: &Instruction) -> Result<Vec<LookupKey>> {
        if self.is_bad_thumb(insn)? {
            return Ok(vec![]);
        }

        let id = ArmInsn::from(insn.id.0);
        let end = insn.address + insn.length as u64;

        match id {
            ArmInsn::ARM_INS_B | ArmInsn::ARM_INS_BL => {
                let detail = self.detail(insn)?;
                if let Some(i) = detail
                    .get_imm_operand(0, 1)
                    .map_err(|e| anyhow!("{insn:#?}, {e:}"))?
                {
                    if detail.conditional {
                        Ok(vec![
                            LookupKey::Arm((Mode::Thumb, i as u64)),
                            LookupKey::Arm((Mode::Thumb, end)),
                        ])
                    } else {
                        Ok(vec![LookupKey::Arm((Mode::Thumb, i as u64))])
                    }
                } else {
                    Ok(vec![])
                }
            }
            ArmInsn::ARM_INS_BX | ArmInsn::ARM_INS_BLX => {
                let detail = self.detail(insn)?;
                if let Some(i) = detail
                    .get_imm_operand(0, 1)
                    .map_err(|e| anyhow!("{insn:#?}, {e:}"))?
                {
                    if detail.conditional {
                        Ok(vec![
                            LookupKey::Arm((Mode::Arm, i as u64)),
                            LookupKey::Arm((Mode::Thumb, end)),
                        ])
                    } else {
                        Ok(vec![LookupKey::Arm((Mode::Arm, i as u64))])
                    }
                } else {
                    Ok(vec![])
                }
            }
            ArmInsn::ARM_INS_CBZ | ArmInsn::ARM_INS_CBNZ => {
                let detail = self.detail(insn)?;
                if let Some(i) = detail
                    .get_imm_operand(1, 2)
                    .map_err(|e| anyhow!("{insn:#?}, {e:}"))?
                {
                    if detail.conditional {
                        Ok(vec![
                            LookupKey::Arm((Mode::Thumb, i as u64)),
                            LookupKey::Arm((Mode::Thumb, end)),
                        ])
                    } else {
                        Ok(vec![LookupKey::Arm((Mode::Thumb, i as u64))])
                    }
                } else {
                    Ok(vec![])
                }
            }
            ArmInsn::ARM_INS_BXNS | ArmInsn::ARM_INS_BLXNS | ArmInsn::ARM_INS_BXJ => Ok(vec![]),
            _ => Ok(vec![LookupKey::Arm((Mode::Thumb, end))]),
        }
    }

    fn should_trim_thumb(&self, insn: &Instruction) -> Result<bool> {
        let id = ArmInsn::from(insn.id.0);
        match id {
            ArmInsn::ARM_INS_B
            | ArmInsn::ARM_INS_BL
            | ArmInsn::ARM_INS_BX
            | ArmInsn::ARM_INS_BLX => {
                let detail = self.detail(insn)?;
                Ok(detail.get_imm_operand(0, 1)?.is_some())
            }
            ArmInsn::ARM_INS_CBZ | ArmInsn::ARM_INS_CBNZ => Ok(true),
            _ => Ok(false),
        }
    }
}

impl ChainArch for ArmChainArch {
    fn is_conditional(&self, insn: &Instruction) -> Result<bool> {
        let detail = self.detail(insn)?;
        Ok(detail.conditional)
    }

    fn is_rop(&self, insn: &Instruction) -> Result<bool> {
        match self.is_thumb(insn)? {
            true => self.is_rop_thumb(insn),
            false => self.is_rop_arm(insn),
        }
    }

    fn is_jop(&self, insn: &Instruction) -> Result<bool> {
        match self.is_thumb(insn)? {
            true => self.is_jop_thumb(insn),
            false => self.is_jop_arm(insn),
        }
    }

    fn next_insns(&self, insn: &Instruction) -> Result<Vec<LookupKey>> {
        match self.is_thumb(insn)? {
            true => self.next_insns_thumb(insn),
            false => self.next_insns_arm(insn),
        }
    }

    fn should_trim(&self, insn: &Instruction) -> Result<bool> {
        match self.is_thumb(insn)? {
            true => self.should_trim_thumb(insn),
            false => self.should_trim_arm(insn),
        }
    }

    fn get_key(&self, insn: &Instruction) -> Result<LookupKey> {
        match self.is_thumb(insn)? {
            true => Ok(LookupKey::Arm((Mode::Thumb, insn.address as u64))),
            false => Ok(LookupKey::Arm((Mode::Arm, insn.address as u64))),
        }
    }
}
