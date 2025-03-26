use colored::*;
use rayon::prelude::*;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Instant;
// #[cfg(windows)]
// use std::os::windows::fs::MetadataExt;

// Structure to hold file counting statistics
struct FileStats {
    total_files: usize,
    total_dirs: usize,
}

/// 处理单个目录的文件和子目录
fn process_directory_entries(
    entries: Vec<fs::DirEntry>,
    ext: &str,
    ignore_dirs: &[String],
) -> (Vec<PathBuf>, Vec<PathBuf>) {
    // 使用线程安全的数据结构来存储结果
    let files = Arc::new(Mutex::new(Vec::with_capacity(entries.len())));
    let dirs = Arc::new(Mutex::new(Vec::with_capacity(entries.len())));

    // 并行处理所有条目
    entries.par_iter().for_each(|entry| {
        let path = entry.path();

        // Check if directory should be ignored
        if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
            if ignore_dirs.iter().any(|ignored| ignored == dir_name) {
                return;
            }
        }

        // 优化：仅获取一次元数据
        if let Ok(metadata) = fs::metadata(&path) {
            // 以点开头的文件和目录被视为隐藏
            let is_hidden_file = path
                .file_name()
                .and_then(|n| n.to_str())
                .map(|s| s.starts_with('.'))
                .unwrap_or(false);
                
            // #[cfg(windows)]
            // if !is_hidden_file {
            //     is_hidden_file = (metadata.file_attributes() & 0x2) != 0; // Windows 使用文件属性的隐藏标志
            // };
            
            if is_hidden_file {
                return;
            }

            // 已经有了元数据，直接使用
            if metadata.is_file() {
                if ext.is_empty() || path.extension().and_then(|e| e.to_str()) == Some(ext) {
                    let mut files = files.lock().unwrap();
                    files.push(path);
                }
            } else {
                // 不是文件，则认为是目录
                let mut dirs = dirs.lock().unwrap();
                dirs.push(path);
            }
        }
    });

    // 获取结果并排序
    let mut files = Arc::try_unwrap(files).unwrap().into_inner().unwrap();
    let mut dirs = Arc::try_unwrap(dirs).unwrap().into_inner().unwrap();

    // 对文件和目录进行排序，保持顺序稳定
    files.sort_by(|a, b| a.file_name().cmp(&b.file_name()));
    dirs.sort_by(|a, b| a.file_name().cmp(&b.file_name()));

    (files, dirs)
}

/// 并行处理子目录
fn process_subdirs_parallel(dirs: Vec<PathBuf>, new_prefix: String, root: &Path, ext: &str, 
                            ignore_dirs: &[String], stats: Arc<Mutex<FileStats>>) {
    // 对于大量目录，使用并行处理
    // if (dirs.len() > 10) {
    //     dirs.par_iter().for_each(|dir| {
    //         list_files(dir, &new_prefix, root, ext, ignore_dirs, Arc::clone(&stats));
    //     });
    // } else {
        // 少量目录顺序处理
        for dir in dirs {
            list_files(&dir, &new_prefix, root, ext, ignore_dirs, Arc::clone(&stats));
        }
    // }
}

/// 主要的文件列表处理函数
fn list_files<P: AsRef<Path>>(
    indir: P, 
    prefix: &str, 
    root: &Path, 
    ext: &str,
    ignore_dirs: &[String],
    stats: Arc<Mutex<FileStats>>
) {
    let dir_path = indir.as_ref();

    if let Ok(entries) = fs::read_dir(dir_path) {
        // 预分配容量可以减少内存重新分配
        let entries: Vec<_> = entries.filter_map(Result::ok).collect();

        // 处理文件和目录条目
        let (files, dirs) = process_directory_entries(entries, ext, ignore_dirs);

        // 更新统计信息
        {
            let mut stats_guard = stats.lock().unwrap();
            stats_guard.total_files += files.len();
            stats_guard.total_dirs += dirs.len();
        }

        // 输出当前目录的文件计数
        let n = files.len();
        if n > 0 {
            let path_str = dir_path.to_string_lossy();
            println!("{}{} | {}", prefix, path_str, n.to_string().green());
        }

        // 处理子目录
        let new_prefix = format!("{}  ", prefix);

        // 并行处理子目录
        process_subdirs_parallel(dirs, new_prefix, root, ext, ignore_dirs, stats);
    } else {
        // 处理访问错误
        eprintln!("Error accessing directory: {}", dir_path.display());
    }
}

fn print_help() {
    println!("file_counter - Count files in directories");
    println!("Usage:");
    println!("  file_counter [OPTIONS] [DIRECTORY]");
    println!("\nOptions:");
    println!("  --ext EXT        Filter files by extension");
    println!("  --ignore DIR     Ignore directories with this name (can be used multiple times)");
    println!("  --help, -h       Show this help message");
    println!("  --version, -v    Show version information");
    println!("\nExamples:");
    println!("  file_counter                      # Count all files in current directory");
    println!("  file_counter /path/to/dir         # Count all files in specified directory");
    println!("  file_counter --ext rs             # Count only Rust files");
    println!("  file_counter --ignore node_modules # Ignore 'node_modules' directories");
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
                println!("file_counter v{}", env!("CARGO_PKG_VERSION"));
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

    // Initialize stats counter
    let stats = Arc::new(Mutex::new(FileStats {
        total_files: 0,
        total_dirs: 0,
    }));
    
    let path = Path::new(dir_path);
    list_files(path, "  ", path, ext, &ignore_dirs, Arc::clone(&stats));
    
    // Print summary statistics
    let elapsed = start_time.elapsed();
    let stats = stats.lock().unwrap();
    println!("\n{}", "Summary:".yellow().bold());
    println!("Total files: {}, ", stats.total_files.to_string().blue().bold());
    // println!("Total directories: {}", stats.total_dirs.to_string().blue().bold());
    println!("Time elapsed: {:.2?}", elapsed);
}
