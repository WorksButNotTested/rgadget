use {
    anyhow::{anyhow, Result},
    capstone::{
        arch::arm64::{Arm64Operand, Arm64OperandType},
        RegId,
    },
    std::hash::{Hash, Hasher},
    typed_builder::TypedBuilder,
};

#[derive(TypedBuilder)]
#[readonly::make]
pub struct Aarch64InstructionDetail {
    pub conditional: bool,
    pub operands: Vec<Arm64Operand>,
}

impl Aarch64InstructionDetail {
    pub fn only_operand_imm(&self) -> Result<Option<i64>> {
        let operands = self.operands.as_slice();
        if operands.len() != 1 {
            Err(anyhow!("Unexpected number of operands"))?;
        }
        let op = &operands.get(0).ok_or(anyhow!("Missing operand"))?;
        match op.op_type {
            Arm64OperandType::Imm(i) => Ok(Some(i)),
            _ => Ok(None),
        }
    }

    pub fn only_operand_reg(&self) -> Result<Option<RegId>> {
        let operands = self.operands.as_slice();
        if operands.len() != 1 {
            Err(anyhow!("Unexpected number of operands"))?;
        }
        let op = &operands.get(0).ok_or(anyhow!("Missing operand"))?;
        match op.op_type {
            Arm64OperandType::Reg(i) => Ok(Some(i)),
            _ => Ok(None),
        }
    }
}

impl PartialEq for Aarch64InstructionDetail {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl Hash for Aarch64InstructionDetail {
    fn hash<H: Hasher>(&self, _state: &mut H) {}
}
