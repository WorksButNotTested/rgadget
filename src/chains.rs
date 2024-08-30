use {
    crate::{
        arch::{ChainArch, LookupKey},
        args::Args,
        chain::Chain,
        instruction::Instruction,
        progress::Progress,
    },
    anyhow::{anyhow, Error, Result},
    dashmap::DashMap,
    fancy_regex::Regex,
    indicatif::ParallelProgressIterator,
    itertools::Itertools,
    log::debug,
    rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator},
    std::iter,
};

pub struct Chains<'a> {
    chains: Vec<Chain<'a>>,
}

impl<'a> Chains<'a> {
    pub fn new<C: ChainArch>(
        args: &Args,
        config: &C,
        file_name: &'a String,
        insns: &'a Vec<Instruction>,
    ) -> Result<Chains<'a>> {
        let chains = Chains::chains(&args, config, file_name, &insns)?;
        Ok(Chains { chains })
    }

    fn chains<C: ChainArch>(
        args: &Args,
        config: &C,
        file_name: &'a String,
        insns: &'a Vec<Instruction>,
    ) -> Result<Vec<Chain<'a>>> {
        let ends = Chains::get_chain_ends(args, config, &insns)?;
        debug!("Found {} chain ends", ends.len());
        let progress = Progress::new("Building chains", ends.len());
        let set = DashMap::new();
        let lookup = Chains::get_lookup(args, config, &insns)?;
        ends.into_par_iter()
            .progress_with(progress)
            .try_for_each(|i| {
                let chain = Chain::builder()
                    .instructions(vec![i])
                    .file_name(file_name)
                    .build();
                let chains = Chains::extend_chain(args, config, file_name, &lookup, chain)?;
                let trimmed = chains
                    .into_iter()
                    .map(|c| Chains::trim_chain(config, file_name, c))
                    .collect::<Result<Vec<Chain>>>()?;
                trimmed
                    .into_iter()
                    .filter(|c| c.instructions.len() > 1)
                    .for_each(|c| {
                        set.insert(c.address(), c);
                    });
                Ok::<(), Error>(())
            })?;
        let result = set.into_iter().map(|(_k, v)| v).collect::<Vec<Chain>>();
        Ok(result)
    }

    fn get_lookup<C: ChainArch>(
        args: &Args,
        config: &C,
        insns: &'a Vec<Instruction>,
    ) -> Result<DashMap<LookupKey, Vec<&'a Instruction>>> {
        let lookup = DashMap::new();
        let progress = Progress::new("Initializing chains", insns.len());
        insns.par_iter().progress_with(progress).try_for_each(|i| {
            if args.conditional || !config.is_conditional(i)? {
                for n in config.next_insns(i)? {
                    lookup.entry(n).or_insert_with(Vec::new).push(i);
                }
            }
            Ok::<(), Error>(())
        })?;
        debug!("Indexed {} instructions", lookup.len());
        Ok(lookup)
    }

    fn is_end<C: ChainArch>(args: &Args, config: &C, i: &'a Instruction) -> Result<bool> {
        if !args.conditional && config.is_conditional(i)? {
            Ok(false)
        } else if args.rop && config.is_rop(i)? {
            Ok(true)
        } else if args.jop && config.is_jop(i)? {
            Ok(true)
        } else if let Some(end) = &args.end {
            let re = Regex::new(end)?;
            let text = format!("{} {}", i.mnemonic, i.operands);
            Ok(re.is_match(&text)?)
        } else {
            Ok(false)
        }
    }

    fn get_chain_ends<C: ChainArch>(
        args: &Args,
        config: &C,
        insns: &'a Vec<Instruction>,
    ) -> Result<Vec<&'a Instruction>> {
        let progress_ends = Progress::new("Finding end of chains", insns.len());
        Ok(insns
            .par_iter()
            .progress_with(progress_ends)
            .filter_map(|i| match Self::is_end(args, config, i) {
                Ok(true) => Some(Ok(i)),
                Ok(false) => None,
                Err(e) => Some(Err(e)),
            })
            .collect::<Result<Vec<&Instruction>>>()?)
    }

    fn extend_chain<C: ChainArch>(
        args: &Args,
        config: &C,
        file_name: &'a String,
        lookup: &DashMap<LookupKey, Vec<&'a Instruction>>,
        chain: Chain<'a>,
    ) -> Result<Vec<Chain<'a>>> {
        let len = chain.instructions.len();
        if len == args.num {
            return Ok(vec![]);
        }

        let first = chain.instructions.first().ok_or(anyhow!("Empty chain"))?;
        let key = config.get_key(&first)?;
        if let Some(nexts) = lookup.get(&key) {
            let new_chains = nexts
                .iter()
                .map(|&i| {
                    let instructions = iter::once(i)
                        .chain(chain.instructions.iter().copied())
                        .collect();
                    let chain = Chain::builder()
                        .instructions(instructions)
                        .file_name(file_name)
                        .build();
                    Chains::extend_chain(args, config, file_name, lookup, chain)
                })
                .flatten_ok()
                .collect::<Result<Vec<Chain>>>()?;
            if new_chains.is_empty() {
                Ok(vec![chain])
            } else {
                Ok(new_chains)
            }
        } else {
            Ok(vec![])
        }
    }

    fn trim_chain<C: ChainArch>(
        config: &C,
        file_name: &'a String,
        chain: Chain<'a>,
    ) -> Result<Chain<'a>> {
        let mut iter = chain.instructions.iter();

        while let Some(i) = iter.next() {
            if !config.should_trim(&i)? {
                let trimmeds = iter::once(i)
                    .chain(iter)
                    .map(|&c| c)
                    .collect::<Vec<&Instruction>>();
                return Ok(Chain::builder()
                    .instructions(trimmeds)
                    .file_name(file_name)
                    .build());
            }
        }
        return Ok(Chain::builder()
            .instructions(vec![])
            .file_name(file_name)
            .build());
    }
}

impl<'a> IntoIterator for Chains<'a> {
    type Item = Chain<'a>;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.chains.into_iter()
    }
}
