use clap::Parser;

#[derive(Debug, Parser)]
#[command(author, version, about)]
pub struct Cli {}

impl Cli {
    pub fn parse_args() -> Self {
        Self::parse()
    }
}
