use {
    crate::{
        arch::{
            arm::insn::{ArmInstructionDetail, Mode},
            DisassemblyArch,
        },
        instruction::InstructionDetail,
        machine::{Machine, MachineEndian},
    },
    anyhow::{anyhow, Result},
    capstone::{
        arch::{
            arm::{ArchMode, ArmCC, ArmOperand},
            ArchDetail::{self, ArmDetail},
            BuildsCapstone, BuildsCapstoneEndian, DetailsArchInsn,
        },
        Capstone, Endian, Insn,
    },
    typed_builder::TypedBuilder,
};

#[derive(TypedBuilder)]
pub struct ArmDisassemblyArch<'a> {
    machine: &'a Machine,
}

impl<'a> DisassemblyArch for ArmDisassemblyArch<'a> {
    fn alignment(&self) -> usize {
        4
    }
    fn max_len(&self) -> usize {
        4
    }
    fn capstone(&self) -> Result<Capstone> {
        Ok(Capstone::new()
            .arm()
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
            ArmDetail(d) => {
                let operands = d.operands().collect::<Vec<ArmOperand>>();
                let conditional = match d.cc() {
                    ArmCC::ARM_CC_AL => false,
                    ArmCC::ARM_CC_INVALID => false,
                    _ => true,
                };
                let detail = ArmInstructionDetail::builder()
                    .operands(operands)
                    .conditional(conditional)
                    .mode(Mode::Arm)
                    .build();
                Ok(InstructionDetail::Arm(detail))
            }
            _ => Err(anyhow!("Unexpected detail type")),
        }
    }
}
