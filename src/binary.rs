use {
    crate::{
        machine::{Machine, MachineArch, MachineBits, MachineEndian},
        section::Section,
    },
    anyhow::{anyhow, Result},
    goblin::{
        elf::header::{EM_AARCH64, EM_ARM, EM_PPC, EM_X86_64},
        elf64::program_header::PF_X,
        Object,
    },
    memmap2::Mmap,
    rayon::iter::{IntoParallelRefIterator, ParallelIterator},
    std::{fs::File, path::Path, slice::from_raw_parts},
};

#[allow(dead_code)]
#[readonly::make]
pub struct Binary<'a> {
    file: File,
    map: Mmap,
    object: Object<'a>,
    pub path: &'a String,
    pub bytes: &'a [u8],
}

impl<'a> Binary<'a> {
    const ELF_ARM_BE8_FLAG: u32 = 0x800000;

    pub fn load(path: &'a String) -> Result<Binary<'a>> {
        let file = File::open(path).unwrap();
        let map = unsafe { Mmap::map(&file).unwrap() };
        let bytes = unsafe { from_raw_parts(map.as_ptr(), map.len()) };
        let object = Object::parse(bytes)?;
        Ok(Binary {
            file,
            map,
            object,
            path,
            bytes,
        })
    }

    pub fn sections(&self) -> Result<Vec<Section>> {
        match &self.object {
            Object::Elf(e) => Ok(e
                .program_headers
                .par_iter()
                .filter(|header| header.p_flags & PF_X != 0)
                .map(|header| {
                    let start_offset = header.p_offset as usize;
                    let end_offset = start_offset + header.p_filesz as usize;
                    Section::builder()
                        .base(header.p_vaddr as usize)
                        .bytes(&self.bytes[start_offset..end_offset])
                        .build()
                })
                .collect()),
            _ => Err(anyhow!("Unidentified input: {:#?}", self.object))?,
        }
    }

    pub fn machine(&self) -> Result<Machine> {
        match &self.object {
            Object::Elf(e) => {
                let mut endian = match e.little_endian {
                    true => MachineEndian::Little,
                    false => MachineEndian::Big,
                };
                let bits = match e.is_64 {
                    true => MachineBits::B64,
                    false => MachineBits::B32,
                };
                let e_machine = e.header.e_machine;
                let arch = match e_machine {
                    EM_PPC => {
                        if endian != MachineEndian::Big {
                            Err(anyhow!("Unsupported architecture/endian combination: {e_machine:#?}/{endian:#?}"))?;
                        }

                        if bits != MachineBits::B32 {
                            Err(anyhow!("Unsupported architecture/bits combination: {e_machine:#?}/{bits:#?}"))?;
                        }
                        MachineArch::PowerPc
                    }
                    EM_ARM => {
                        if e.header.e_flags & Binary::ELF_ARM_BE8_FLAG != 0 {
                            match endian {
                                MachineEndian::Little => {
                                    Err(anyhow!("Invalid endian-ness for BE8 image"))?
                                }
                                MachineEndian::Big => endian = MachineEndian::Little,
                            }
                        }
                        if bits != MachineBits::B32 {
                            Err(anyhow!("Unsupported architecture/bits combination: {e_machine:#?}/{bits:#?}"))?;
                        }
                        MachineArch::Arm
                    }
                    EM_X86_64 => {
                        if endian != MachineEndian::Little {
                            Err(anyhow!("Unsupported architecture/endian combination: {e_machine:#?}/{endian:#?}"))?;
                        }
                        MachineArch::X86_64
                    }
                    EM_AARCH64 => {
                        if bits != MachineBits::B64 {
                            Err(anyhow!("Unsupported architecture/bits combination: {e_machine:#?}/{bits:#?}"))?;
                        }
                        MachineArch::Aarch64
                    }
                    _ => Err(anyhow!("Unidentified machine: {:#?}", e_machine))?,
                };
                let machine = Machine::builder()
                    .endian(endian)
                    .bits(bits)
                    .arch(arch)
                    .build();
                Ok(machine)
            }
            _ => Err(anyhow!("Unidentified input: {:#?}", self.object))?,
        }
    }

    pub fn name(&self) -> Result<String> {
        let path = Path::new(self.path);
        let name = path.file_name().ok_or(anyhow!(""))?;
        name.to_str().map(|s| s.to_string()).ok_or(anyhow!(""))
    }
}
