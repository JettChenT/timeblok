use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about=None)]
pub struct Args{
    /// Path to the file to process
    #[arg(short, long)]
    pub infile: String,
    #[arg(short, long)]
    pub outfile: Option<String>
}

pub fn parse() -> Args {
    Args::parse()
}