use {
    crate::{
        arch::arm::insn::Mode,
        args::Args,
        chain::Chain,
        colours::{ColourRange, ColourType, Colours},
        instruction::InstructionDetail,
        progress::Progress,
    },
    ansi_term::{
        Colour::{Purple, White, Yellow},
        Style,
    },
    anyhow::{anyhow, Result},
    fancy_regex::{Captures, Match, Regex},
    indicatif::ParallelProgressIterator,
    itertools::Itertools,
    rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator},
};

pub struct RegexFilter<'a> {
    args: &'a Args,
}

impl<'a> RegexFilter<'a> {
    pub fn new(args: &Args) -> Result<RegexFilter> {
        Ok(RegexFilter { args })
    }

    fn filter_includes<'t>(&self, text: &'t str) -> Result<Vec<(usize, Vec<Match<'t>>)>> {
        let includes = self
            .args
            .includes
            .iter()
            .map(|s| Regex::new(&s).map_err(|e| anyhow!("Failed to compile regex: {e:}")))
            .collect::<Result<Vec<Regex>>>()?;

        let caps = includes
            .into_iter()
            .map(|re| {
                re.captures(&text)
                    .map_err(|e| anyhow!("Failed to gather captures: {e:}"))
            })
            .collect::<Result<Vec<Option<Captures>>>>()?;

        if caps.iter().any(|c| c.is_none()) {
            return Ok(vec![]);
        }

        let matches = caps
            .into_iter()
            .enumerate()
            .map(|(i, c)| {
                (
                    i,
                    c.unwrap()
                        .iter()
                        .skip(1)
                        .filter_map(|c| c)
                        .collect::<Vec<Match>>(),
                )
            })
            .collect::<Vec<(usize, Vec<Match>)>>();

        Ok(matches)
    }

    fn filter_excludes(&self, text: &str) -> Result<bool> {
        let excludes = self
            .args
            .excludes
            .iter()
            .map(|s| Regex::new(&s).map_err(|e| anyhow!("Failed to compile regex: {e:}")))
            .collect::<Result<Vec<Regex>>>()?;

        let matches = excludes
            .iter()
            .map(|re| {
                re.is_match(&text)
                    .map_err(|e| anyhow!("Failed to test if excluded: {e:}"))
            })
            .collect::<Result<Vec<bool>>>()?;

        Ok(matches.iter().any(|&m| m))
    }

    pub fn filter(&self, chains: Vec<Chain>) -> Result<Vec<String>> {
        let progress_text = Progress::new("Generating Text", chains.len());

        let chain_text = chains
            .into_par_iter()
            .progress_with(progress_text)
            .map(|c| {
                let text = self.get_text(&c);
                (c, text)
            })
            .collect::<Vec<(Chain, String)>>();

        let progress_excludes = Progress::new("Filtering Excludes", chain_text.len());

        let excludes = if self.args.excludes.is_empty() {
            chain_text
        } else {
            chain_text
                .into_par_iter()
                .progress_with(progress_excludes)
                .filter_map(|(c, t)| {
                    self.filter_excludes(&t)
                        .map(|r| if r { None } else { Some(Ok((c, t))) })
                        .unwrap_or_else(|e| Some(Err(e)))
                })
                .collect::<Result<Vec<(Chain, String)>>>()?
        };

        let progress_includes = Progress::new("Filtering Includes", excludes.len());

        let includes = if self.args.includes.is_empty() {
            excludes
                .par_iter()
                .progress_with(progress_includes)
                .map(|(c, t)| Ok((c, t, Vec::<(usize, Vec<Match>)>::new())))
                .collect::<Result<Vec<(&Chain, &String, Vec<(usize, Vec<Match>)>)>>>()?
        } else {
            excludes
                .par_iter()
                .progress_with(progress_includes)
                .filter_map(|(c, t)| {
                    self.filter_includes(t)
                        .map(|m| {
                            if m.is_empty() {
                                None
                            } else {
                                Some(Ok((c, t, m)))
                            }
                        })
                        .unwrap_or_else(|e| Some(Err(e)))
                })
                .collect::<Result<Vec<(&Chain, &String, Vec<(usize, Vec<Match>)>)>>>()?
        };

        let progress_rendering = Progress::new("Rendering", excludes.len());

        includes
            .into_par_iter()
            .progress_with(progress_rendering)
            .map(|(c, t, m)| {
                let text = self.format(c, t, &m)?;
                Ok(text)
            })
            .collect::<Result<Vec<String>>>()
    }

    fn get_text(&self, chain: &Chain) -> String {
        chain
            .instructions
            .iter()
            .map(|i| format!("{} {}", i.mnemonic, i.operands))
            .join("; ")
    }

    fn get_instruction_colours(&self, chain: &Chain) -> Result<Colours> {
        let mut ranges = Vec::<ColourRange>::new();
        let mut offset = 0;
        for i in &chain.instructions {
            let mnemonic_start = offset;
            let mnemonic_end = offset + i.mnemonic.len();
            ranges.push(ColourRange {
                start: mnemonic_start,
                end: mnemonic_end,
                colour: Some(ColourType::Mnemonic),
            });

            /* Skip the space between mnemonic and operand */
            let operands_start = mnemonic_end + 1;
            let operands_end = operands_start + i.operands.len();
            ranges.push(ColourRange {
                start: operands_start,
                end: operands_end,
                colour: Some(ColourType::Operand),
            });

            /* Skip the '; ' seperator */
            offset = operands_end + 2;
        }
        Ok(Colours::new(ranges))
    }

    fn get_regex_colours(&self, matches: &Vec<(usize, Vec<Match>)>) -> Result<Colours> {
        let ranges = matches
            .iter()
            .map(|(i, m)| {
                let ranges = m
                    .iter()
                    .map(|c| ColourRange {
                        start: c.start(),
                        end: c.end(),
                        colour: Some(ColourType::Regex(*i)),
                    })
                    .collect::<Vec<ColourRange>>();
                ranges
            })
            .flatten()
            .collect::<Vec<ColourRange>>();

        return Ok(Colours::new(ranges));
    }

    fn format_instructions(
        &self,
        chain: &Chain,
        text: &String,
        matches: &Vec<(usize, Vec<Match>)>,
    ) -> Result<String> {
        let instruction_ranges = self.get_instruction_colours(chain)?;
        let regex_ranges = self.get_regex_colours(matches)?;
        let colours = regex_ranges.background(instruction_ranges)?;

        let coloured = colours
            .ranges
            .iter()
            .map(|c| {
                let fragment = &text[c.start..c.end];
                c.colour
                    .as_ref()
                    .map(|c| Style::from(c))
                    .map(|c| c.paint(fragment).to_string())
                    .unwrap_or(fragment.to_string())
            })
            .collect::<Vec<String>>();
        Ok(coloured.join(""))
    }

    fn format(
        &self,
        chain: &Chain,
        text: &String,
        matches: &Vec<(usize, Vec<Match>)>,
    ) -> Result<String> {
        let modes = chain
            .instructions
            .iter()
            .filter_map(|i| match &i.detail {
                InstructionDetail::Arm(a) => Some(a.mode.clone()),
                _ => None,
            })
            .collect::<Vec<Mode>>();

        let mode = if let Some(first_mode) = modes.first() {
            let mixed = modes.iter().skip(1).any(|x| x.eq(first_mode) == false);
            let marker = if mixed {
                format!("{first_mode:}*")
            } else {
                format!("{first_mode:}")
            };
            format!("[{}]", White.bold().paint(marker)).to_string()
        } else {
            String::new()
        };

        let address = if let Some(first) = chain.instructions.first() {
            let addr = format!("{}!0x{:x}", chain.file_name, first.address);
            format!("{}: ", Yellow.bold().paint(addr))
        } else {
            String::new()
        };

        let instructions = self.format_instructions(chain, text, matches)?;

        let bytes = if self.args.bytes {
            let bytes = chain
                .instructions
                .iter()
                .map(|i| i.bytes.iter().map(|b| format!("{b:02x}")).join(" "))
                .collect::<Vec<String>>();
            let coloured = bytes
                .into_iter()
                .map(|t| Purple.dimmed().paint(t).to_string())
                .collect::<Vec<String>>()
                .join("; ");
            let prefix = "\n\t".to_string();
            prefix + coloured.as_str()
        } else {
            String::new()
        };

        Ok(format!("{mode:} {address:}{instructions:}{bytes:}"))
    }
}
