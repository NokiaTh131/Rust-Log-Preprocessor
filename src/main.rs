use clap::Parser;
use std::fs::File;
use std::io::{BufRead, BufReader};
use regex::Regex;
use std::collections::HashSet;


#[derive(Parser)]
#[command(name = "logprep")]
#[command(version = "1.0")]
#[command(author = "nokiath131")]
#[command(about = "Filters log lines by keyword", long_about = None)]
struct Args {
    //file path
    #[arg(short,long)]
    path: String,

    //keyword
    #[arg(short,long)]
    filter: Option<String>,

    //regex
    #[arg(short,long)]
    regex: Option<String>,

    /// List all detected services
    #[arg(long)]
    list_services: bool,
}

fn print_result(args: Args, file: File) {
    
    let reader = BufReader::new(file);
    let regex = args.regex.as_ref().map(|pattern| {
        Regex::new(&pattern).expect("Invalid Pattern")
    });

    if args.list_services {
        list_services(reader);
        return;
    }

    for line in reader.lines() {
        if let Ok(line) = line {
            let is_match = match (&args.filter, &regex) {
                (Some(filter), _) if line.contains(filter) => true,
                (_, Some(re)) if re.is_match(&line) => true,
                _ => false,
            };
            if is_match {
                println!("{}",line);
            }
        }
    }
}

fn list_services(reader: BufReader<File>) {
    let mut services = HashSet::new();

    // Regex to extract: "service[pid]:"
    let re = Regex::new(r"([a-zA-Z0-9._-]+)\[\d+\]:").unwrap();

    for line in reader.lines().flatten() {
        if let Some(caps) = re.captures(&line) {
            let service = caps.get(1).unwrap().as_str().to_string();
            services.insert(service);
        }
    }

    let mut sorted: Vec<_> = services.into_iter().collect();
    sorted.sort();

    println!("Services found:");
    for service in sorted {
        println!("- {}", service);
    }
}

fn main() {
    let args = Args::parse();
    let file = File::open(&args.path).expect("Failed to open input file");

    print_result(args, file);
}
