use {
    crate::{
        arch::{
            aarch64::chain::Aarch64LookupKey, arm::chain::ArmLookupKey,
            powerpc::chain::PowerPcLookupKey, x64::chain::X64LookupKey,
        },
        instruction::{Instruction, InstructionDetail},
    },
    anyhow::Result,
    capstone::{arch::ArchDetail, Capstone, Insn},
};
pub mod aarch64;
pub mod arm;
pub mod powerpc;
pub mod x64;

#[derive(Eq, PartialEq, Hash)]
pub enum LookupKey {
    Arm(ArmLookupKey),
    X64(X64LookupKey),
    Aarch64(Aarch64LookupKey),
    PowerPc(PowerPcLookupKey),
}

pub trait DisassemblyArch: Sync {
    /* Disassembly functions */
    fn alignment(&self) -> usize;
    fn max_len(&self) -> usize;
    fn capstone(&self) -> Result<Capstone>;
    fn instruction_details(&self, insn: &Insn, detail: &ArchDetail) -> Result<InstructionDetail>;
}

pub trait ChainArch: Sync {
    fn is_conditional(&self, insn: &Instruction) -> Result<bool>;
    fn is_rop(&self, insn: &Instruction) -> Result<bool>;
    fn is_jop(&self, insn: &Instruction) -> Result<bool>;
    fn next_insns(&self, insn: &Instruction) -> Result<Vec<LookupKey>>;
    fn should_trim(&self, insn: &Instruction) -> Result<bool>;
    fn get_key(&self, insn: &Instruction) -> Result<LookupKey>;
}
