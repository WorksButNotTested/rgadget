use {
    anyhow::{anyhow, Result},
    capstone::arch::ppc::PpcOperand,
    std::hash::{Hash, Hasher},
    typed_builder::TypedBuilder,
};

#[derive(TypedBuilder)]
#[readonly::make]
pub struct PowerPcInstructionDetail {
    pub conditional: bool,
    pub operands: Vec<PpcOperand>,
}

impl PowerPcInstructionDetail {
    pub fn only_operand_imm(&self) -> Result<Option<i64>> {
        let operands = self.operands.as_slice();
        let len = operands.len();
        if len != 1 {
            return Ok(None);
        }
        let op = operands.get(0).ok_or(anyhow!("Missing operand"))?;
        match op {
            PpcOperand::Imm(i) => Ok(Some(*i)),
            _ => Ok(None),
        }
    }
}

impl PartialEq for PowerPcInstructionDetail {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl Hash for PowerPcInstructionDetail {
    fn hash<H: Hasher>(&self, _state: &mut H) {}
}
