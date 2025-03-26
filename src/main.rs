use colored::*;
use rayon::prelude::*;
use std::env;
use std::fs;
use std::os::windows::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex}; // 添加 rayon 的并行迭代器特性

/// 检查路径是否为隐藏文件或目录
fn is_hidden(path: &Path) -> bool {
    // 在类 Unix 系统上，以点开头的文件和目录被视为隐藏
    let mut status = path
        .file_name()
        .and_then(|n| n.to_str())
        .map(|s| s.starts_with('.'))
        .unwrap_or(false);
    if status {
        return status;
    }

    // Windows 使用文件属性的隐藏标志
    if cfg!(windows) {
        if let Ok(metadata) = fs::metadata(path) {
            // 0x2 是 Windows 的 FILE_ATTRIBUTE_HIDDEN 标志
            status = (metadata.file_attributes() & 0x2) != 0;
        }
    }
    return status;
}

/// 处理单个目录的文件和子目录
fn process_directory_entries(
    entries: Vec<fs::DirEntry>,
    ext: &str,
) -> (Vec<PathBuf>, Vec<PathBuf>) {
    // 使用线程安全的数据结构来存储结果
    let files = Arc::new(Mutex::new(Vec::with_capacity(entries.len())));
    let dirs = Arc::new(Mutex::new(Vec::with_capacity(entries.len())));

    // 并行处理所有条目
    entries.par_iter().for_each(|entry| {
        let path = entry.path();

        // 首先检查是否为隐藏文件/目录
        if is_hidden(&path) {
            return;
        }

        // 使用元数据一次性获取所有需要的文件属性，避免重复调用
        if let Ok(metadata) = fs::metadata(&path) {
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

/// 序列处理子目录
fn process_subdirs_sequential(dirs: Vec<PathBuf>, new_prefix: String, root: &Path, ext: &str) {
    for dir in dirs {
        list_files(&dir, &new_prefix, root, ext);
    }
}

/// 主要的文件列表处理函数
fn list_files<P: AsRef<Path>>(indir: P, prefix: &str, root: &Path, ext: &str) {
    let dir_path = indir.as_ref();

    if let Ok(entries) = fs::read_dir(dir_path) {
        // 预分配容量可以减少内存重新分配
        let entries: Vec<_> = entries.flatten().collect();

        // 处理文件和目录条目
        let (files, dirs) = process_directory_entries(entries, ext);

        // 输出当前目录的文件计数
        let n = files.len();
        if n > 0 {
            let path_str = dir_path.to_string_lossy();
            println!("{}{} | {}", prefix, path_str, n.to_string().green());
        }

        // 处理子目录
        let new_prefix = format!("{}  ", prefix);

        // 少量目录直接顺序处理
        process_subdirs_sequential(dirs, new_prefix, root, ext);
    }
}

fn main() {
    // Get command line arguments
    let args: Vec<String> = env::args().collect();

    // Check for version flag
    if args.len() > 1 && (args[1] == "--version" || args[1] == "-v") {
        println!("file_counter v{}", env!("CARGO_PKG_VERSION"));
        return;
    }

    let mut dir_path = "."; // Default to current directory
    let mut ext = ""; // Default to no extension filter

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
            }
            arg => {
                // First non-flag argument is the directory path
                if !arg.starts_with('-') && dir_path == "." {
                    dir_path = arg;
                    i += 1;
                } else {
                    eprintln!("Unknown argument: {}", arg);
                    return;
                }
            }
        }
    }

    println!("Counting files in directory: {}", dir_path.blue());
    if !ext.is_empty() {
        println!("Filtering by extension: {}", ext);
    }

    let path = Path::new(dir_path);
    list_files(path, "  ", path, ext);
}
