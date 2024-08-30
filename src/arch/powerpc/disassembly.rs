use {
    crate::{
        arch::{powerpc::insn::PowerPcInstructionDetail, DisassemblyArch},
        instruction::InstructionDetail,
        machine::{Machine, MachineEndian},
    },
    anyhow::{anyhow, Result},
    capstone::{
        arch::{
            ppc::{ArchMode, PpcInsn, PpcOperand},
            ArchDetail::{self, PpcDetail},
            BuildsCapstone, BuildsCapstoneEndian, DetailsArchInsn,
        },
        Capstone, Endian, Insn,
    },
    typed_builder::TypedBuilder,
};

#[derive(TypedBuilder)]
pub struct PpcDisassemblyArch<'a> {
    machine: &'a Machine,
}

impl<'a> DisassemblyArch for PpcDisassemblyArch<'a> {
    fn alignment(&self) -> usize {
        4
    }
    fn max_len(&self) -> usize {
        4
    }
    fn capstone(&self) -> Result<Capstone> {
        Ok(Capstone::new()
            .ppc()
            .mode(ArchMode::Mode32)
            .endian(match self.machine.endian {
                MachineEndian::Big => Endian::Big,
                MachineEndian::Little => Endian::Little,
            })
            .detail(true)
            .build()?)
    }

    fn instruction_details(&self, insn: &Insn, detail: &ArchDetail) -> Result<InstructionDetail> {
        match detail {
            PpcDetail(d) => {
                let operands = d.operands().collect::<Vec<PpcOperand>>();
                let id = PpcInsn::from(insn.id().0);
                let conditional = match id {
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
                    | PpcInsn::PPC_INS_BCCTRL => true,
                    _ => false,
                };
                let detail = PowerPcInstructionDetail::builder()
                    .operands(operands)
                    .conditional(conditional)
                    .build();
                Ok(InstructionDetail::PowerPc(detail))
            }
            _ => Err(anyhow!("Unexpected detail type")),
        }
    }
}
