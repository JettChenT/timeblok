use clap::{Parser, ValueEnum};
use clap::builder::PossibleValue;

#[derive(Clone, Debug)]
pub enum OutputTypes {
    Ics,
    Csv,
}

impl ValueEnum for OutputTypes {
    fn value_variants<'a>() -> &'a [Self] {
        &[Self::Ics, Self::Csv]
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        Some(match self {
            Self::Ics => PossibleValue::new("ics").help("internet calendar format"),
            Self::Csv => PossibleValue::new("csv").help("comma-separated values"),
        })
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about=None)]
pub struct Args {
    /// Path to the file to be processed
    #[arg(value_parser)]
    pub infile: Option<String>,
    /// Path of the output .ics file
    #[arg(short = 'f', long)]
    pub outfile: Option<String>,
    /// Whether to open the output file after it is created
    #[arg(short = 'o', long)]
    pub open: bool,
    /// Parse only option
    #[arg(long)]
    pub parse_only: bool,
    /// Whether to print the result ICS file
    #[arg(long)]
    pub print: bool,
    /// Create a new file directly
    /// This takes lower precedence than the `infile` argument
    #[arg(long, short)]
    pub new: bool,
    /// Specify format of the output
    /// Currently only supports `ics` and `csv`
    /// Will try to infer from the file extension if not specified
    #[arg(long, default_value=None)]
    pub format: Option<OutputTypes>,    
}

pub fn parse() -> Args {
    Args::parse()
}
