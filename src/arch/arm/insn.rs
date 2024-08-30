use {
    anyhow::{anyhow, Result},
    capstone::{
        arch::arm::{ArmOperand, ArmOperandType, ArmReg},
        RegId,
    },
    std::{
        fmt::{self, Display, Formatter},
        hash::{Hash, Hasher},
    },
    typed_builder::TypedBuilder,
};

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub enum Mode {
    Thumb,
    Arm,
}

impl Display for Mode {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Mode::Thumb => {
                write!(f, "T")
            }
            Mode::Arm => {
                write!(f, "A")
            }
        }
    }
}

#[derive(TypedBuilder)]
#[readonly::make]
pub struct ArmInstructionDetail {
    pub conditional: bool,
    pub operands: Vec<ArmOperand>,
    pub mode: Mode,
}

impl ArmInstructionDetail {
    pub fn contains_pc(&self) -> Result<bool> {
        Ok(self.operands.iter().any(|op| {
            if let ArmOperandType::Reg(reg) = op.op_type {
                reg.0 as u32 == ArmReg::ARM_REG_PC
            } else {
                false
            }
        }))
    }

    pub fn contains_lr(&self) -> Result<bool> {
        Ok(self.operands.iter().any(|op| {
            if let ArmOperandType::Reg(reg) = op.op_type {
                reg.0 as u32 == ArmReg::ARM_REG_LR
            } else {
                false
            }
        }))
    }

    pub fn contains_sp(&self) -> Result<bool> {
        Ok(self.operands.iter().any(|op| {
            if let ArmOperandType::Reg(reg) = op.op_type {
                reg.0 as u32 == ArmReg::ARM_REG_SP
            } else {
                false
            }
        }))
    }

    pub fn get_imm_operand(&self, idx: usize, count: usize) -> Result<Option<i32>> {
        let operands = self.operands.as_slice();
        let len = operands.len();
        if len != count {
            Err(anyhow!(
                "Unexpected number of operands (imm), got: {len:}, expected: {count:}"
            ))?;
        }

        if idx >= count {
            Err(anyhow!("Out of bounds, n: {idx:}, m: {count:}"))?;
        }

        let op = &operands.get(0).ok_or(anyhow!("Missing operand"))?;
        match op.op_type {
            ArmOperandType::Imm(i) => Ok(Some(i)),
            _ => Ok(None),
        }
    }

    pub fn get_reg_operand(&self, idx: usize, count: usize) -> Result<Option<RegId>> {
        let operands = self.operands.as_slice();
        let len = operands.len();
        if len != count {
            Err(anyhow!(
                "Unexpected number of operands (reg), got: {len:}, expected: {count:}"
            ))?;
        }
        if idx >= count {
            Err(anyhow!("Out of bounds, n: {idx:}, m: {count:}"))?;
        }

        let op = &operands.get(idx).ok_or(anyhow!("Missing operand"))?;
        match op.op_type {
            ArmOperandType::Reg(r) => Ok(Some(r)),
            _ => Ok(None),
        }
    }
}

impl PartialEq for ArmInstructionDetail {
    fn eq(&self, other: &Self) -> bool {
        self.mode == other.mode
    }
}

impl Hash for ArmInstructionDetail {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.mode.hash(state);
    }
}
