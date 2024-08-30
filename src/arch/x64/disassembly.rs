use {
    crate::{
        arch::{x64::insn::X64InstructionDetail, DisassemblyArch},
        instruction::InstructionDetail,
    },
    anyhow::{anyhow, Result},
    capstone::{
        arch::{
            x86::{ArchMode, X86Operand, X86XopCC},
            ArchDetail::{self, X86Detail},
            BuildsCapstone, DetailsArchInsn,
        },
        Capstone, Insn,
    },
};

pub struct X64DisassemblyArch;

impl DisassemblyArch for X64DisassemblyArch {
    fn alignment(&self) -> usize {
        1
    }
    fn max_len(&self) -> usize {
        15
    }
    fn capstone(&self) -> Result<Capstone> {
        Ok(Capstone::new()
            .x86()
            .mode(ArchMode::Mode64)
            .detail(true)
            .build()?)
    }

    fn instruction_details(&self, _insn: &Insn, detail: &ArchDetail) -> Result<InstructionDetail> {
        match detail {
            X86Detail(d) => {
                let operands = d.operands().collect::<Vec<X86Operand>>();
                let conditional = match d.xop_cc() {
                    X86XopCC::X86_XOP_CC_INVALID => false,
                    X86XopCC::X86_XOP_CC_FALSE => false,
                    X86XopCC::X86_XOP_CC_TRUE => true,
                    _ => true,
                };
                let detail = X64InstructionDetail::builder()
                    .operands(operands)
                    .conditional(conditional)
                    .build();
                Ok(InstructionDetail::X64(detail))
            }
            _ => Err(anyhow!("Unexpected detail type")),
        }
    }
}
