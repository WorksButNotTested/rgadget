use {
    crate::{
        arch::{powerpc::insn::PowerPcInstructionDetail, ChainArch, LookupKey},
        instruction::{Instruction, InstructionDetail},
    },
    anyhow::{anyhow, Result},
    capstone::arch::ppc::PpcInsn,
};

pub type PowerPcLookupKey = u64;

pub struct PpcChainArch;

impl PpcChainArch {
    fn detail<'a>(&self, insn: &'a Instruction) -> Result<&'a PowerPcInstructionDetail> {
        match &insn.detail {
            InstructionDetail::PowerPc(detail) => Ok(detail),
            _ => Err(anyhow!("Unexpected instruction detail")),
        }
    }

    fn is_branch(&self, insn: &Instruction) -> Result<bool> {
        let id = PpcInsn::from(insn.id.0);
        match id {
            PpcInsn::PPC_INS_BT
            | PpcInsn::PPC_INS_BTA
            | PpcInsn::PPC_INS_BTLR
            | PpcInsn::PPC_INS_BTCTR
            | PpcInsn::PPC_INS_BTL
            | PpcInsn::PPC_INS_BTLA
            | PpcInsn::PPC_INS_BTLRL
            | PpcInsn::PPC_INS_BTCTRL
            | PpcInsn::PPC_INS_BF
            | PpcInsn::PPC_INS_BFA
            | PpcInsn::PPC_INS_BFLR
            | PpcInsn::PPC_INS_BFCTR
            | PpcInsn::PPC_INS_BFL
            | PpcInsn::PPC_INS_BFLA
            | PpcInsn::PPC_INS_BFLRL
            | PpcInsn::PPC_INS_BFCTRL
            | PpcInsn::PPC_INS_BDNZ
            | PpcInsn::PPC_INS_BDNZA
            | PpcInsn::PPC_INS_BDNZLR
            | PpcInsn::PPC_INS_BDNZL
            | PpcInsn::PPC_INS_BDNZLA
            | PpcInsn::PPC_INS_BDNZLRL
            | PpcInsn::PPC_INS_BDNZT
            | PpcInsn::PPC_INS_BDNZTA
            | PpcInsn::PPC_INS_BDNZTLR
            | PpcInsn::PPC_INS_BDNZTL
            | PpcInsn::PPC_INS_BDNZTLA
            | PpcInsn::PPC_INS_BDNZTLRL
            | PpcInsn::PPC_INS_BDNZF
            | PpcInsn::PPC_INS_BDNZFA
            | PpcInsn::PPC_INS_BDNZFLR
            | PpcInsn::PPC_INS_BDNZFL
            | PpcInsn::PPC_INS_BDNZFLA
            | PpcInsn::PPC_INS_BDNZFLRL
            | PpcInsn::PPC_INS_BDZ
            | PpcInsn::PPC_INS_BDZA
            | PpcInsn::PPC_INS_BDZLR
            | PpcInsn::PPC_INS_BDZL
            | PpcInsn::PPC_INS_BDZLA
            | PpcInsn::PPC_INS_BDZLRL
            | PpcInsn::PPC_INS_BDZT
            | PpcInsn::PPC_INS_BDZTA
            | PpcInsn::PPC_INS_BDZTLR
            | PpcInsn::PPC_INS_BDZTL
            | PpcInsn::PPC_INS_BDZTLA
            | PpcInsn::PPC_INS_BDZTLRL
            | PpcInsn::PPC_INS_BDZF
            | PpcInsn::PPC_INS_BDZFA
            | PpcInsn::PPC_INS_BDZFLR
            | PpcInsn::PPC_INS_BDZFL
            | PpcInsn::PPC_INS_BDZFLA
            | PpcInsn::PPC_INS_BDZFLRL
            | PpcInsn::PPC_INS_BCA
            | PpcInsn::PPC_INS_BCL
            | PpcInsn::PPC_INS_BCLA
            | PpcInsn::PPC_INS_BCLRL
            | PpcInsn::PPC_INS_BCCTRL
            | PpcInsn::PPC_INS_B
            | PpcInsn::PPC_INS_BA
            | PpcInsn::PPC_INS_BL
            | PpcInsn::PPC_INS_BLA
            | PpcInsn::PPC_INS_BC
            | PpcInsn::PPC_INS_BLR
            | PpcInsn::PPC_INS_BCTR
            | PpcInsn::PPC_INS_BLRL
            | PpcInsn::PPC_INS_BCTRL => Ok(true),
            _ => Ok(false),
        }
    }
}

impl ChainArch for PpcChainArch {
    fn is_conditional(&self, insn: &Instruction) -> Result<bool> {
        let detail = self.detail(insn)?;
        Ok(detail.conditional)
    }

    fn is_rop(&self, insn: &Instruction) -> Result<bool> {
        let id = PpcInsn::from(insn.id.0);
        match id {
            PpcInsn::PPC_INS_BLR => Ok(true),
            _ => Ok(false),
        }
    }

    fn is_jop(&self, insn: &Instruction) -> Result<bool> {
        let id = PpcInsn::from(insn.id.0);
        match id {
            PpcInsn::PPC_INS_BCTR | PpcInsn::PPC_INS_BCTRL => Ok(true),
            _ => Ok(false),
        }
    }

    fn next_insns(&self, insn: &Instruction) -> Result<Vec<LookupKey>> {
        let end = insn.address + insn.length as u64;
        let detail = self.detail(insn)?;
        let branch = self.is_branch(insn)?;
        if branch {
            let dest = detail
                .only_operand_imm()
                .map_err(|e| anyhow!("{insn:#?}, {e:}"))?;
            if detail.conditional {
                if let Some(d) = dest {
                    Ok(vec![LookupKey::PowerPc(d as u64), LookupKey::PowerPc(end)])
                } else {
                    Ok(vec![LookupKey::PowerPc(end)])
                }
            } else {
                if let Some(d) = dest {
                    Ok(vec![LookupKey::PowerPc(d as u64)])
                } else {
                    Ok(vec![])
                }
            }
        } else {
            Ok(vec![LookupKey::PowerPc(end)])
        }
    }

    fn should_trim(&self, insn: &Instruction) -> Result<bool> {
        self.is_branch(insn)
    }

    fn get_key(&self, insn: &Instruction) -> Result<LookupKey> {
        Ok(LookupKey::PowerPc(insn.address))
    }
}
