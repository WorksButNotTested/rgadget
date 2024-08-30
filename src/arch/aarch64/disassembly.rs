use {
    crate::{
        arch::{aarch64::insn::Aarch64InstructionDetail, DisassemblyArch},
        instruction::InstructionDetail,
        machine::{Machine, MachineEndian},
    },
    anyhow::{anyhow, Result},
    capstone::{
        arch::{
            arm64::{ArchMode, Arm64CC, Arm64Operand},
            ArchDetail::{self, Arm64Detail},
            BuildsCapstone, BuildsCapstoneEndian, DetailsArchInsn,
        },
        Capstone, Endian, Insn,
    },
    typed_builder::TypedBuilder,
};

#[derive(TypedBuilder)]
pub struct Aarch64DisassemblyArch<'a> {
    machine: &'a Machine,
}

impl<'a> DisassemblyArch for Aarch64DisassemblyArch<'a> {
    fn alignment(&self) -> usize {
        4
    }
    fn max_len(&self) -> usize {
        4
    }
    fn capstone(&self) -> Result<Capstone> {
        Ok(Capstone::new()
            .arm64()
            .mode(ArchMode::Arm)
            .endian(match self.machine.endian {
                MachineEndian::Big => Endian::Big,
                MachineEndian::Little => Endian::Little,
            })
            .detail(true)
            .build()?)
    }

    fn instruction_details(&self, _insn: &Insn, detail: &ArchDetail) -> Result<InstructionDetail> {
        match detail {
            Arm64Detail(d) => {
                let operands = d.operands().collect::<Vec<Arm64Operand>>();
                let conditional = match d.cc() {
                    Arm64CC::ARM64_CC_AL => false,
                    Arm64CC::ARM64_CC_INVALID => false,
                    _ => true,
                };
                let detail = Aarch64InstructionDetail::builder()
                    .operands(operands)
                    .conditional(conditional)
                    .build();
                Ok(InstructionDetail::Aarch64(detail))
            }
            _ => Err(anyhow!("Unexpected detail type")),
        }
    }
}
