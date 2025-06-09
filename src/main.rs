use std::fs::File;
use clap::Parser;
use crate::c_type::Args;

mod filter;
mod c_type;

fn main() {
    let args = Args::parse();
    let file = File::open(&args.path).expect("Failed to open input file");
    filter::filter_by_input(args, file);
}
