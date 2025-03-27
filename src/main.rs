use clap::Parser;
use colored::*;
use regex::Regex;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Instant;

mod file_size;
mod list_files;
mod print;
mod tree;

use file_size::{format_size, parse_size};
use list_files::{list_files, FileStats};

/// Command line arguments structure
#[derive(Parser)]
#[command(author, version, about = "Count files in directories")]
struct Cli {
    /// Target paths, can include directories and patterns, defaults to current directory
    #[arg(default_value = ".")]
    paths: Vec<String>,

    /// Filter by file extension
    #[arg(long, value_name = "EXT")]
    ext: Option<String>,

    /// Ignore directories with specified names
    #[arg(long, value_name = "DIR", action = clap::ArgAction::Append)]
    ignore: Vec<String>,

    /// Filter files smaller than specified size
    #[arg(long = "min", value_name = "SIZE")]
    min_size: Option<String>,
    
    /// Filter files larger than specified size
    #[arg(long = "max", value_name = "SIZE")]
    max_size: Option<String>,

    /// Include child directory files in current directory statistics
    #[arg(short = 'c', long = "children")]
    include_children: bool,

    /// Show only directory statistics, not the file tree
    #[arg(short = 'n', long = "num")]
    show_stats_only: bool,

    /// Limit search depth, 0 means unlimited
    #[arg(short = 'L', long = "level", default_value = "0")]
    max_depth: usize,

    /// Filter files by pattern (supports glob patterns like *.rs)
    #[arg(short = 'p', long = "pattern")]
    pattern: Option<String>,
}

fn main() {
    // Track execution time
    let start_time = Instant::now();

    // Use clap to parse command line arguments
    let args = Cli::parse();

    // Process arguments - 分离目录和模式
    let (directory, pattern_from_args) = process_paths(&args.paths);
    let dir_path = directory.to_string_lossy();
    let ext = args.ext.unwrap_or_default();
    let ignore_dirs = args.ignore;

    // 如果从命令行参数中解析出了模式，优先使用它
    let pattern_str = if !pattern_from_args.is_empty() {
        Some(pattern_from_args)
    } else {
        args.pattern.clone()
    };

    // Handle minimum file size limit
    let min_size = if let Some(size_str) = args.min_size {
        match parse_size(&size_str) {
            Ok(size) => size,
            Err(err) => {
                eprintln!("Error parsing size: {}", err);
                return;
            }
        }
    } else {
        0
    };
    
    // Handle maximum file size limit
    let max_size = if let Some(size_str) = args.max_size {
        match parse_size(&size_str) {
            Ok(size) => size,
            Err(err) => {
                eprintln!("Error parsing size: {}", err);
                return;
            }
        }
    } else {
        u64::MAX // Maximum possible value
    };

    // Handle pattern matching
    let pattern = if let Some(pattern_str) = &pattern_str {
        // Convert glob pattern to regex pattern
        let regex_pattern = pattern_str
            .replace(".", "\\.")  // Escape dots
            .replace("*", ".*")   // Convert * to .*
            .replace("?", ".");   // Convert ? to .

        match Regex::new(&regex_pattern) {
            Ok(re) => {
                println!("Filtering by pattern: {}", pattern_str);
                Some(re)
            },
            Err(err) => {
                eprintln!("Error parsing pattern: {}", err);
                return;
            }
        }
    } else {
        None
    };

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
    if max_size < u64::MAX {
        println!("Filtering files larger than: {}", format_size(max_size));
    }
    if args.include_children {
        println!("Including child directory files in count");
    }
    if args.show_stats_only {
        println!("Showing directory statistics only (no file tree)");
    }
    if args.max_depth > 0 {
        println!("Maximum directory depth: {}", args.max_depth);
    }

    // Initialize stats counter
    let stats = Arc::new(Mutex::new(FileStats {
        total_files: 0,
        total_dirs: 0,
        total_bytes: 0,
    }));

    let path = Path::new(&directory);
    list_files(
        path,
        "",
        &[],
        path,
        &ext,
        &ignore_dirs,
        Arc::clone(&stats),
        min_size,
        max_size,
        args.include_children,
        args.show_stats_only,
        args.max_depth,
        pattern.as_ref(),  // 传递正则表达式引用
    );

    // Print summary statistics
    let elapsed = start_time.elapsed();
    let stats = stats.lock().unwrap();
    println!("\n{}", "Summary:".yellow().bold());
    println!(
        "Total files : {}",
        stats.total_files.to_string().blue().bold()
    );
    println!(
        "Total size  : {}",
        format_size(stats.total_bytes).green().bold()
    );
    println!("Time elapsed: {:.2?}", elapsed);
}

/// 处理输入路径，将其分割为目录和模式
fn process_paths(paths: &[String]) -> (PathBuf, String) {
    if paths.is_empty() {
        return (PathBuf::from("."), String::new());
    }
    
    // 查找含有通配符的参数
    let mut directory = PathBuf::from(".");
    let mut pattern = String::new();
    
    for path in paths {
        // 检查路径是否包含通配符
        if path.contains('*') || path.contains('?') {
            // 处理类似 ./**.exe 或 dir/*.txt 的情况
            if let Some(pos) = path.rfind('/') {
                // 分割目录和模式
                let (dir_part, pat_part) = path.split_at(pos + 1);
                if !pat_part.is_empty() {
                    pattern = pat_part.to_string();
                    
                    // 如果目录部分不为空，设置为目录
                    if dir_part != "./" && dir_part != "/" {
                        directory = PathBuf::from(dir_part.trim_end_matches('/'));
                    }
                }
            } else {
                // 没有路径分隔符，整个是模式
                pattern = path.clone();
            }
        } else {
            // 没有通配符的参数视为目录
            directory = PathBuf::from(path);
        }
    }
    
    // 如果只有一个参数且它是模式，使用当前目录
    if directory == PathBuf::from(".") && !pattern.is_empty() && paths.len() == 1 {
        // 保持当前目录
    } else if pattern.is_empty() && paths.len() == 1 {
        // 如果只有一个参数且不含通配符，视为目录
        directory = PathBuf::from(&paths[0]);
    }
    
    // 调试打印
    // println!("解析结果 - 目录: {:?}, 模式: {}", directory, pattern);
    (directory, pattern)
}
