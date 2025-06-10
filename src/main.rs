use std::{fs::File, time::Instant};
use clap::Parser;
use crate::c_type::Args;

mod filter;
mod c_type;

fn main() {
    let args = Args::parse();
    let file = File::open(&args.path).expect("Failed to open input file");

    let start = Instant::now();
    filter::filter_by_input(args, file);
    let duration = start.elapsed();
    println!("filter_by_input took: {:?}", duration);
}
