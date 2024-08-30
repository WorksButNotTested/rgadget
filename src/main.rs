use {
    crate::{
        args::Args, binary::Binary, chain::Chain, progress::Progress, regex::RegexFilter,
        section::Section,
    },
    ansi_term::Colour::White,
    anyhow::Result,
    clap::Parser,
    dashmap::DashSet,
    env_logger::Builder,
    instruction::Instruction,
    itertools::Itertools,
    log::debug,
    rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator},
    std::cmp::min,
};

mod arch;
mod args;
mod binary;
mod chain;
mod chains;
mod colours;
mod disassembler;
mod instruction;
mod machine;
mod progress;
mod regex;
mod section;

fn main() -> Result<()> {
    let args = Args::parse();
    let level = if args.verbose {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Info
    };
    Builder::new().filter_level(level).init();
    Progress::set_verbose(args.verbose);

    debug!("{:}", args);

    let binaries = args
        .files
        .par_iter()
        .map(|f| Binary::load(f))
        .collect::<Result<Vec<Binary>>>()?;

    let sections = binaries
        .par_iter()
        .map(|binary| binary.sections())
        .collect::<Result<Vec<Vec<Section>>>>()?;

    let insns = sections
        .iter()
        .zip(binaries.iter())
        .map(|(sections, binary)| Ok((binary.name()?, binary.machine()?.disasemble(&sections)?)))
        .collect::<Result<Vec<(String, Vec<Instruction>)>>>()?;

    let chains = insns
        .iter()
        .zip(binaries.iter())
        .map(|((name, insns), binary)| Ok(binary.machine()?.get_chains(&args, name, &insns)?))
        .flatten_ok()
        .collect::<Result<Vec<Chain>>>()?;

    let mut chains = if args.duplicates {
        chains
    } else {
        let set = chains.into_par_iter().collect::<DashSet<Chain>>();
        set.into_iter().collect::<Vec<Chain>>()
    };

    chains.sort();

    let filter = RegexFilter::new(&args)?;
    let filtered = filter.filter(chains)?;

    let num_filtered = filtered.len();

    let max = args.limit.unwrap_or(num_filtered);
    let max = min(max, num_filtered);

    let limited = filtered.into_iter().take(max).collect::<Vec<String>>();

    let index_len = limited.len().to_string().len();
    limited.into_iter().enumerate().for_each(|(i, c)| {
        let prefixed = format!("#{}", i + 1).to_string();
        let fixed_len = format!("{:>1$}", prefixed, index_len + 1);
        let index = format!("[{}]", White.bold().paint(fixed_len.to_string())).to_string();
        println!("{index:} {c:}");
    });

    println!("Displaying {:#?} of {:#?} Gadgets", max, num_filtered);

    /* Don't hang around to clean-up, just get out */
    unsafe {
        libc::_exit(0);
    }
}
