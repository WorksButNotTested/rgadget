use {
    crate::arch::{
        aarch64::insn::Aarch64InstructionDetail, arm::insn::ArmInstructionDetail,
        powerpc::insn::PowerPcInstructionDetail, x64::insn::X64InstructionDetail,
    },
    ansi_term::Colour::{Blue, Green, Yellow},
    capstone::{InsnGroupId, InsnId, RegId},
    std::{
        fmt::{self, Debug, Display, Formatter},
        hash::{Hash, Hasher},
        string::String,
    },
    typed_builder::TypedBuilder,
};

#[derive(TypedBuilder)]
#[readonly::make]
pub struct Instruction {
    pub id: InsnId,
    pub name: String,

    pub length: usize,
    pub address: u64,
    pub bytes: Vec<u8>,

    pub mnemonic: String,
    pub operands: String,

    regs_read: Vec<(RegId, String)>,
    regs_write: Vec<(RegId, String)>,
    groups: Vec<(InsnGroupId, String)>,
    pub detail: InstructionDetail,
}

impl PartialEq for Instruction {
    fn eq(&self, other: &Self) -> bool {
        self.detail == other.detail && self.bytes == other.bytes
    }
}

impl Eq for Instruction {}

impl Hash for Instruction {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.detail.hash(state);
        self.bytes.hash(state);
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{} {}",
            Green.bold().paint(&self.mnemonic),
            Blue.bold().paint(&self.operands)
        )?;

        Ok(())
    }
}

impl Debug for Instruction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let address = format!("0x{:x} ", self.address);
        write!(f, "{}", Yellow.bold().paint(&address))?;
        writeln!(f, "{self:}")?;

        let bytes = self
            .bytes
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<Vec<String>>()
            .join(" ");
        writeln!(f, "\tbytes: {bytes}")?;

        writeln!(f, "\tname: {} ({})", self.name, self.id.0)?;

        if !self.regs_read.is_empty() {
            writeln!(f, "\tread:")?;
            for (reg, name) in &self.regs_read {
                writeln!(f, "\t\t{} ({})", name, reg.0)?;
            }
        }

        if !self.regs_write.is_empty() {
            writeln!(f, "\twrite:")?;
            for (reg, name) in &self.regs_write {
                writeln!(f, "\t\t{} ({})", name, reg.0)?;
            }
        }

        if !self.groups.is_empty() {
            writeln!(f, "\tgroups:")?;
            for (group, name) in &self.groups {
                writeln!(f, "\t\t{} ({})", name, group.0)?;
            }
        }

        Ok(())
    }
}

pub enum InstructionDetail {
    Arm(ArmInstructionDetail),
    X64(X64InstructionDetail),
    Aarch64(Aarch64InstructionDetail),
    PowerPc(PowerPcInstructionDetail),
}

impl PartialEq for InstructionDetail {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (InstructionDetail::Arm(d1), InstructionDetail::Arm(d2)) => d1 == d2,
            (InstructionDetail::X64(d1), InstructionDetail::X64(d2)) => d1 == d2,
            (InstructionDetail::Aarch64(d1), InstructionDetail::Aarch64(d2)) => d1 == d2,
            (InstructionDetail::PowerPc(d1), InstructionDetail::PowerPc(d2)) => d1 == d2,
            _ => false,
        }
    }
}

impl Hash for InstructionDetail {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            InstructionDetail::Arm(d) => d.hash(state),
            InstructionDetail::X64(d) => d.hash(state),
            InstructionDetail::Aarch64(d) => d.hash(state),
            InstructionDetail::PowerPc(d) => d.hash(state),
        }
    }
}
