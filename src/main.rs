use colored::*;
use std::env;
use std::path::Path; // Add Path import
use std::time::Instant;
use std::sync::{Arc, Mutex};

mod file_size; 
use file_size::{format_size, parse_size}; // Assuming file_size.rs is in the same directory

mod tree;
mod list_files;
use list_files::{list_files, FileStats}; // Assuming list_files.rs is in the same directory


fn print_help() {
    println!("ntree - Count files in directories");
    println!("Usage:");
    println!("  ntree [OPTIONS] [DIRECTORY]");
    println!("\nOptions:");
    println!("  --ext EXT        Filter files by extension");
    println!("  --ignore DIR     Ignore directories with this name (can be used multiple times)");
    println!("  --min-size SIZE  Filter files smaller than SIZE (e.g. 1MB, 500KB, 10B)");
    println!("  -c               Include child directory files in directory count");
    println!("  --help, -h       Show this help message");
    println!("  --version, -v    Show version information");
    println!("\nExamples:");
    println!("  ntree                       # Count all files in current directory");
    println!("  ntree /path/to/dir          # Count all files in specified directory");
    println!("  ntree --ext rs              # Count only Rust files");
    println!("  ntree --ignore node_modules # Ignore 'node_modules' directories");
    println!("  ntree --min-size 1MB        # Only count files larger than 1MB");
}

fn main() {
    // Track execution time
    let start_time = Instant::now();
    
    // Get command line arguments
    let args: Vec<String> = env::args().collect();

    // Check for help or version flags first
    if args.len() > 1 {
        match args[1].as_str() {
            "--version" | "-v" => {
                println!("ntree v{}", env!("CARGO_PKG_VERSION"));
                return;
            },
            "--help" | "-h" => {
                print_help();
                return;
            },
            _ => {}
        }
    }

    let mut dir_path = "."; // Default to current directory
    let mut ext = ""; // Default to no extension filter
    let mut ignore_dirs: Vec<String> = vec![];
    let mut min_size: u64 = 0; // 默认不过滤文件大小
    let mut include_children = false; // 默认不将子目录文件计入当前目录

    // Parse arguments
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--ext" => {
                if i + 1 < args.len() {
                    ext = &args[i + 1];
                    i += 2;
                } else {
                    eprintln!("Error: --ext requires an extension argument");
                    return;
                }
            },
            "--ignore" => {
                if i + 1 < args.len() {
                    ignore_dirs.push(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("Error: --ignore requires a directory name argument");
                    return;
                }
            },
            "--min-size" => {
                if i + 1 < args.len() {
                    match parse_size(&args[i + 1]) {
                        Ok(size) => {
                            min_size = size;
                            i += 2;
                        },
                        Err(err) => {
                            eprintln!("Error parsing size: {}", err);
                            return;
                        }
                    }
                } else {
                    eprintln!("Error: --min-size requires a size argument");
                    return;
                }
            },
            "-c" => {
                include_children = true;
                i += 1;
            },
            arg => {
                // First non-flag argument is the directory path
                if !arg.starts_with('-') && dir_path == "." {
                    dir_path = arg;
                    i += 1;
                } else {
                    eprintln!("Unknown argument: {}", arg);
                    print_help();
                    return;
                }
            }
        }
    }

    println!("Counting files in directory: {}", dir_path.blue());
    if !ext.is_empty() {
        println!("Filtering by extension: {}", ext);
    }
    if !ignore_dirs.is_empty() {
        println!("Ignoring directories: {}", ignore_dirs.join(", "));
    }
    if min_size > 0 {
        println!("Filtering files smaller than: {}", format_size(min_size));
    }
    if include_children {
        println!("Including child directory files in count");
    }

    // Initialize stats counter
    let stats = Arc::new(Mutex::new(FileStats {
        total_files: 0,
        total_dirs: 0,
        total_bytes: 0,
    }));
    
    let path = Path::new(dir_path);
    // 传递 include_children 参数
    list_files(path, "", &[], path, ext, &ignore_dirs, Arc::clone(&stats), min_size, include_children);
    
    // Print summary statistics
    let elapsed = start_time.elapsed();
    let stats = stats.lock().unwrap();
    println!("\n{}", "Summary:".yellow().bold());
    println!("Total files : {}", stats.total_files.to_string().blue().bold());
    println!("Total size  : {}", format_size(stats.total_bytes).green().bold());
    println!("Time elapsed: {:.2?}", elapsed);
}
