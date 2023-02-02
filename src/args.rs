use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about=None)]
pub struct Args {
    /// Path to the file to be processed
    #[arg(value_parser)]
    pub infile: String,
    /// Path of the output .ics file
    #[arg(short = 'f', long)]
    pub outfile: Option<String>,
    /// Whether to open the output file after it is created
    #[arg(short = 'o', long)]
    pub open: bool,
    /// Parse only option
    #[arg(long)]
    pub parse_only: bool,
    /// Whether to print the parsed output
    #[arg(long)]
    pub print: bool,
}

pub fn parse() -> Args {
    Args::parse()
}
