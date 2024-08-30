use {
    crate::instruction::Instruction,
    std::cmp::Ordering,
    std::{
        hash::{Hash, Hasher},
        string::String,
    },
    typed_builder::TypedBuilder,
};

#[derive(TypedBuilder)]
#[readonly::make]
pub struct Chain<'a> {
    pub instructions: Vec<&'a Instruction>,
    pub file_name: &'a String,
}

impl<'a> PartialOrd for Chain<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> Ord for Chain<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.file_name
            .cmp(other.file_name)
            .then(self.address().cmp(&other.address()))
    }
}

impl<'a> PartialEq for Chain<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.file_name == other.file_name && self.instructions == other.instructions
    }
}

impl<'a> Eq for Chain<'a> {}

impl Hash for Chain<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.file_name.hash(state);
        self.instructions.iter().for_each(|i| i.hash(state));
    }
}

impl<'a> Chain<'a> {
    pub fn address(&self) -> u64 {
        self.instructions
            .get(0)
            .map(|i| i.address)
            .unwrap_or_default()
    }
}
