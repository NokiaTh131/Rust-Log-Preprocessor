use clap::Parser;
use serde::Serialize;

#[derive(Parser)]
#[command(name = "logprep")]
#[command(version = "1.0")]
#[command(author = "nokiath131")]
#[command(about = "Filters log lines by keyword", long_about = None)]
pub struct Args {
        //file path
        #[arg(short,long)]
        pub path: String,
    
        //keyword
        #[arg(short,long)]
        pub filter: Option<String>,
    
        //regex
        #[arg(short,long)]
        pub regex: Option<String>,
    
        /// List all detected services
        #[arg(long)]
        pub list_services: bool,

        #[arg(long)]
        pub dedup: bool,

        #[arg(long)]
        pub json: bool,

        #[arg(long, help = "Write JSON output to a file")]
        pub output: Option<String>,

        #[arg(long)]
        pub summary: bool,
    }

#[derive(Serialize)]
#[derive(Clone)]
pub struct LogRecord {
    pub timestamp: String,
    pub service: String,
    pub message: String,
}