use {
    std::fmt::{self, Display, Formatter},
    typed_builder::TypedBuilder,
};

#[derive(TypedBuilder)]
#[readonly::make]
pub struct Section<'a> {
    pub base: usize,
    pub bytes: &'a [u8],
}

impl<'a> Display for Section<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        writeln!(
            f,
            "section - base: 0x{:x}, len: 0x{:x}",
            self.base,
            self.bytes.len()
        )?;
        Ok(())
    }
}
