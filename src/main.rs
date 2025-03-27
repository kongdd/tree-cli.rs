use clap::Parser;
use colored::*;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Instant;

mod file_size;
mod list_files;
mod print;
mod tree;

use file_size::{format_size, parse_size};
use list_files::{list_files, FileStats};

/// 命令行参数解析结构体
#[derive(Parser)]
#[command(author, version, about = "Count files in directories")]
struct Cli {
    /// 目标目录，默认为当前目录
    #[arg(default_value = ".")]
    directory: PathBuf,

    /// 按文件扩展名过滤
    #[arg(long, value_name = "EXT")]
    ext: Option<String>,

    /// 忽略指定名称的目录
    #[arg(long, value_name = "DIR", action = clap::ArgAction::Append)]
    ignore: Vec<String>,

    /// 过滤小于指定大小的文件
    #[arg(long = "min-size", value_name = "SIZE")]
    min_size: Option<String>,

    /// 将子目录文件计入当前目录统计
    #[arg(short = 'c', long = "children")]
    include_children: bool,

    /// 只显示目录统计信息，不显示文件树
    #[arg(long = "num")]
    show_stats_only: bool,
}

fn main() {
    // Track execution time
    let start_time = Instant::now();

    // 使用clap解析命令行参数
    let args = Cli::parse();

    // 处理参数
    let dir_path = args.directory.to_string_lossy();
    let ext = args.ext.unwrap_or_default();
    let ignore_dirs = args.ignore;

    // 处理文件大小限制
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
    if args.include_children {
        println!("Including child directory files in count");
    }
    if args.show_stats_only {
        println!("Showing directory statistics only (no file tree)");
    }

    // Initialize stats counter
    let stats = Arc::new(Mutex::new(FileStats {
        total_files: 0,
        total_dirs: 0,
        total_bytes: 0,
    }));

    let path = Path::new(&args.directory);
    list_files(
        path,
        "",
        &[],
        path,
        &ext,
        &ignore_dirs,
        Arc::clone(&stats),
        min_size,
        args.include_children,
        args.show_stats_only,
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
