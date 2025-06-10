use crate::c_type::{Args, LogRecord};
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};
use regex::Regex;
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use once_cell::sync::Lazy;


pub fn filter_by_input(args: Args, file: File) {
    
        let reader = BufReader::new(file);
        let regex = args.regex.as_ref().map(|pattern| {
            Regex::new(&pattern).expect("Invalid Pattern")
        });
    
        if args.list_services {
            list_services(reader);
            return;
        }
    
        let seen_hashes = Arc::new(Mutex::new(HashSet::new()));
        let json_records = Arc::new(Mutex::new(Vec::new()));
        let lines: Vec<String> = reader.lines().flatten().collect();
        let total_lines_read = lines.len();
        let kept_lines = Arc::new(AtomicUsize::new(0));
        let tmp_kept_lines = Arc::new(AtomicUsize::new(0));
 
        lines.into_par_iter().for_each(|line| {
                let is_match = match (&args.filter, &regex) {
                    (Some(filter), _) if line.contains(filter) => true,
                    (_, Some(re)) if re.is_match(&line) => true,
                    _ => false,
                };
            
                if is_match {
                    // Deduplication
                    tmp_kept_lines.fetch_add(1, Ordering::Relaxed);
                    if args.dedup {
                        let mut hasher = Sha256::new();
                        hasher.update(line.as_bytes());
                        let hash_hex = format!("{:x}", hasher.finalize());
            
                        let mut hashes = seen_hashes.lock().unwrap();
                        if hashes.contains(&hash_hex) {
                            return; // skip duplicate
                        }
                        hashes.insert(hash_hex);
                    }

                    kept_lines.fetch_add(1, Ordering::Relaxed);

                    // Output
                    if args.json {
                        if let Some(record) = parse_log_line(&line) {
                            if let Some(_) = args.output {
                                let mut records = json_records.lock().unwrap();
                                records.push(record);
                            } else {
                                match serde_json::to_string(&record) {
                                    Ok(json) => println!("{}", json),
                                    Err(e) => eprintln!("Error encoding JSON: {}", e),
                                }
                            }
                        }
                    } else {
                        println!("{}", line);
                    }
                }
            });

            if let Some(output_file) = args.output {
                match File::create(&output_file) {
                    Ok(mut file) => {
                        if let Ok(records) = json_records.lock() {
                            let json_string = serde_json::to_string_pretty(&*records)
                                .unwrap_or_else(|_| "[]".to_string());
                            if let Err(e) = file.write_all(json_string.as_bytes()) {
                                eprintln!("Failed to write to file: {}", e);
                            }
                        }
                    }
                    Err(e) => eprintln!("Failed to create file: {}", e),
                }
            }

            if args.summary {
                println!("- Total lines read: {}", total_lines_read);
                println!("- Lines kept after filtering{}: {}",
                  if args.dedup { " & deduplication" } else { "" },
                    kept_lines.load(Ordering::Relaxed)
                );
                if args.dedup {
                    let int_kept_lines = tmp_kept_lines.load(Ordering::Relaxed);
                    let dedup_length = seen_hashes.lock().unwrap().len();
                    println!("- Total duplicates: {}", int_kept_lines - dedup_length);
                }
            }
    }

    

static LIST_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"([a-zA-Z0-9._-]+)\[\d+\]:").expect("Invalid regex pattern")
});

pub fn list_services(reader: BufReader<File>) {
        
        let lines: Vec<String> = reader.lines().flatten().collect();
        // Regex to extract: "service[pid]:"    
        let services: HashSet<String> = lines.par_iter().filter_map(|line| {
                LIST_REGEX.captures(line).map(|cap| {cap[1].to_string()})
        }).collect();
    
        let mut sorted: Vec<_> = services.into_iter().collect();
        sorted.sort();
    
        println!("Services found:");
        for service in sorted {
            println!("- {}", service);
        }
}

static LOG_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"^(?P<timestamp>\S+)\s+\S+\s+(?P<service>[^\[]+)(?:\[\d+\])?:\s*(?P<message>.+)$")
            .expect("Invalid regex pattern")
    });

fn parse_log_line(line: &str) -> Option<LogRecord> {        
        if let Some(caps) = LOG_REGEX.captures(line) {
            Some(LogRecord {
                timestamp: caps["timestamp"].to_string(),
                service: caps["service"].trim().to_string(),
                message: caps["message"].to_string(),
            })
        } else {
            None
        }
}
    

