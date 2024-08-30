use {
    crate::{
        arch::{
            aarch64::{chain::Aarch64ChainArch, disassembly::Aarch64DisassemblyArch},
            arm::{
                chain::ArmChainArch,
                disassembly::{arm::ArmDisassemblyArch, thumb::ThumbDisassemblyArch},
            },
            powerpc::{chain::PpcChainArch, disassembly::PpcDisassemblyArch},
            x64::{chain::X64ChainArch, disassembly::X64DisassemblyArch},
        },
        args::Args,
        chain::Chain,
        chains::Chains,
        disassembler::Disassembler,
        instruction::Instruction,
        section::Section,
    },
    anyhow::Result,
    std::fmt::{self, Display, Formatter},
    typed_builder::TypedBuilder,
};

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
#[allow(dead_code)]
pub enum MachineEndian {
    Big,
    Little,
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
#[allow(dead_code)]
pub enum MachineBits {
    B32,
    B64,
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
#[allow(dead_code)]
pub enum MachineArch {
    PowerPc,
    Arm,
    Aarch64,
    X86_64,
}

#[derive(TypedBuilder, Clone, Debug, Hash, Eq, PartialEq)]
#[readonly::make]
pub struct Machine {
    pub endian: MachineEndian,
    pub bits: MachineBits,
    pub arch: MachineArch,
}

impl Display for Machine {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "endian: {:#?}, bits: {:#?}, arch: {:#?}",
            self.endian, self.bits, self.arch
        )
    }
}

impl Machine {
    pub fn disasemble<'a>(&self, sections: &'a Vec<Section<'a>>) -> Result<Vec<Instruction>> {
        match self.arch {
            MachineArch::Arm => {
                let arm_config = ArmDisassemblyArch::builder().machine(self).build();
                let arm_insns = Disassembler::disassemble(&arm_config, sections)?;

                let thumb_config = ThumbDisassemblyArch::builder().machine(self).build();
                let thumb_insns = Disassembler::disassemble(&thumb_config, sections)?;

                let insns = arm_insns
                    .into_iter()
                    .chain(thumb_insns.into_iter())
                    .collect::<Vec<Instruction>>();
                Ok(insns)
            }
            MachineArch::X86_64 => {
                let arch = X64DisassemblyArch {};
                let insns = Disassembler::disassemble(&arch, sections)?;
                Ok(insns)
            }
            MachineArch::Aarch64 => {
                let arch = Aarch64DisassemblyArch::builder().machine(self).build();
                let insns = Disassembler::disassemble(&arch, sections)?;
                Ok(insns)
            }
            MachineArch::PowerPc => {
                let arch = PpcDisassemblyArch::builder().machine(self).build();
                let insns = Disassembler::disassemble(&arch, sections)?;
                Ok(insns)
            }
        }
    }

    pub fn get_chains<'a>(
        &self,
        args: &Args,
        file_name: &'a String,
        insns: &'a Vec<Instruction>,
    ) -> Result<Vec<Chain<'a>>> {
        match self.arch {
            MachineArch::Arm => {
                let config = ArmChainArch {};
                let chains = Chains::new(&args, &config, file_name, insns)?
                    .into_iter()
                    .collect::<Vec<Chain>>();
                Ok(chains)
            }
            MachineArch::X86_64 => {
                let arch = X64ChainArch {};
                let chains = Chains::new(&args, &arch, file_name, insns)?
                    .into_iter()
                    .collect::<Vec<Chain>>();
                Ok(chains)
            }
            MachineArch::Aarch64 => {
                let arch = Aarch64ChainArch {};
                let chains = Chains::new(&args, &arch, file_name, insns)?
                    .into_iter()
                    .collect::<Vec<Chain>>();
                Ok(chains)
            }
            MachineArch::PowerPc => {
                let arch = PpcChainArch {};
                let chains = Chains::new(&args, &arch, file_name, insns)?
                    .into_iter()
                    .collect::<Vec<Chain>>();
                Ok(chains)
            }
        }
    }
}
