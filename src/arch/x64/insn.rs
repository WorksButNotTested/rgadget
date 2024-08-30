use {
    anyhow::{anyhow, Result},
    capstone::arch::x86::{X86Operand, X86OperandType},
    std::hash::{Hash, Hasher},
    typed_builder::TypedBuilder,
};

#[derive(TypedBuilder)]
#[readonly::make]
pub struct X64InstructionDetail {
    pub conditional: bool,
    pub operands: Vec<X86Operand>,
}

impl X64InstructionDetail {
    pub fn only_operand_imm(&self) -> Result<Option<i64>> {
        let operands = self.operands.as_slice();
        if operands.len() != 1 {
            Err(anyhow!("Unexpected number of operands"))?;
        }
        let op = &operands.get(0).ok_or(anyhow!("Missing operand"))?;
        match op.op_type {
            X86OperandType::Imm(i) => Ok(Some(i)),
            _ => Ok(None),
        }
    }
}

impl PartialEq for X64InstructionDetail {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl Hash for X64InstructionDetail {
    fn hash<H: Hasher>(&self, _state: &mut H) {}
}
