use {
    clap::Parser,
    std::fmt::{self, Display, Formatter},
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(short, long, required = true, help = "Name of the file to process")]
    pub files: Vec<String>,

    #[arg(
        short,
        long,
        help = "Maximum number of gadgets in a chain",
        default_value_t = 6
    )]
    pub num: usize,

    #[arg(
        short,
        long,
        help = "Find ROP gadgets",
        required_unless_present_any = ["jop", "end"],
        default_value_t = false
    )]
    pub rop: bool,

    #[arg(
        short,
        long,
        help = "Find JOP gadgets",
        required_unless_present_any = ["rop", "end"],
        default_value_t = false
    )]
    pub jop: bool,

    #[arg(
        short,
        long,
        help = "Find gadgets with custom ending (regex)",
        required_unless_present_any = ["jop", "rop"],
    )]
    pub end: Option<String>,

    #[arg(
        short,
        long,
        help = "Include conditional instructions",
        default_value_t = false
    )]
    pub conditional: bool,

    #[arg(short, long, help = "Verbose output", default_value_t = false)]
    pub verbose: bool,

    #[arg(short, long, help = "Limit the number of results")]
    pub limit: Option<usize>,

    #[arg(
        short,
        long,
        help = "Show bytes of the instructions in the chain",
        default_value_t = false
    )]
    pub bytes: bool,

    #[arg(short, long, help = "Show duplicate chains", default_value_t = false)]
    pub duplicates: bool,

    #[arg(short = 'x', long, help = "Exclude chains matching regex")]
    pub excludes: Vec<String>,

    #[arg(short, long, help = "Regex to apply to output")]
    pub includes: Vec<String>,
}

impl Display for Args {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        writeln!(f, "ARGS")?;
        writeln!(f, "files:")?;

        for file in &self.files {
            writeln!(f, "\t{}", file)?;
        }

        writeln!(f, "num: {}", self.num)?;
        writeln!(f, "rop: {}", self.rop)?;
        writeln!(f, "jop: {}", self.jop)?;
        writeln!(f, "bytes: {}", self.bytes)?;

        for regex in &self.includes {
            writeln!(f, "include: {}", regex)?;
        }

        for regex in &self.excludes {
            writeln!(f, "exclude: {}", regex)?;
        }
        Ok(())
    }
}
