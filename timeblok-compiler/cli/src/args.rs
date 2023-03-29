use clap::Parser;

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
}

pub fn parse() -> Args {
    Args::parse()
}
