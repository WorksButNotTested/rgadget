use {
    ansi_term::{
        Colour::{self, Blue, Cyan, Green, Purple, Red},
        Style,
    },
    anyhow::Result,
};

#[derive(Clone, PartialEq, Debug)]
pub enum ColourType {
    Mnemonic,
    Operand,
    Regex(usize),
}

impl From<&ColourType> for Style {
    fn from(c: &ColourType) -> Style {
        match c {
            ColourType::Mnemonic => Cyan.normal(),
            ColourType::Operand => Blue.normal(),
            ColourType::Regex(num) => {
                let colors = [
                    Red,
                    Green,
                    Purple,
                    Colours::ORANGE,
                    Colours::PINK,
                    Colours::LIME,
                    Colours::BRIGHT_YELLOW,
                    Colours::TURQUIOSE,
                ];
                let idx = num % colors.len();
                colors[idx].bold()
            }
        }
    }
}

#[readonly::make]
#[derive(Debug)]
pub struct Colours {
    pub ranges: Vec<ColourRange>,
}

impl Colours {
    pub const PINK: Colour = Colour::Fixed(164);
    pub const LIME: Colour = Colour::Fixed(82);
    pub const BRIGHT_YELLOW: Colour = Colour::Fixed(226);
    pub const ORANGE: Colour = Colour::Fixed(208);
    pub const TURQUIOSE: Colour = Colour::Fixed(6);

    pub fn new(ranges: Vec<ColourRange>) -> Colours {
        Colours { ranges }
    }

    pub fn max(&self) -> usize {
        if self.ranges.len() == 0 {
            0
        } else {
            self.ranges[self.ranges.len() - 1].end
        }
    }

    pub fn colour(&self, idx: usize) -> Option<ColourType> {
        for r in &self.ranges {
            if r.start <= idx && idx < r.end {
                return r.colour.clone();
            }
        }
        None
    }

    pub fn background(&self, other: Colours) -> Result<Colours> {
        let mut ranges = Vec::<ColourRange>::new();
        let limit = usize::max(self.max(), other.max());
        let mut current = ColourRange {
            start: 0,
            end: 0,
            colour: None,
        };
        for i in 0..limit {
            let colour_type = self.colour(i).or(other.colour(i));
            if current.colour == colour_type {
                current.end = i + 1;
            } else {
                if !current.is_empty() {
                    ranges.push(current);
                }
                current = ColourRange {
                    start: i,
                    end: i + 1,
                    colour: colour_type,
                };
            }
        }

        if !current.is_empty() {
            ranges.push(current);
        }
        Ok(Colours { ranges })
    }
}

#[derive(Debug)]
pub struct ColourRange {
    pub start: usize,
    pub end: usize,
    pub colour: Option<ColourType>,
}

impl ColourRange {
    pub fn is_empty(&self) -> bool {
        if self.end > self.start {
            false
        } else {
            true
        }
    }
}
