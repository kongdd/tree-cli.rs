use colored::Colorize;
use rayon::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use crate::format_size;

// Structure to hold file counting statistics
pub struct FileStats {
    pub total_files: usize,
    pub total_dirs: usize,
    pub total_bytes: u64,
}

/// 处理单个目录的文件和子目录
fn process_directory_entries(
    entries: Vec<fs::DirEntry>,
    ext: &str,
    ignore_dirs: &[String],
    min_size: u64,
) -> (Vec<(PathBuf, u64)>, Vec<PathBuf>) {
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

            if is_hidden_file {
                return;
            }

            // 已经有了元数据，直接使用
            if metadata.is_file() {
                // 检查文件大小是否满足最小要求
                let file_size = metadata.len();
                if file_size < min_size {
                    return; // 跳过小于指定大小的文件
                }

                if ext.is_empty() || path.extension().and_then(|e| e.to_str()) == Some(ext) {
                    let mut files = files.lock().unwrap();
                    files.push((path, file_size)); // 将文件大小一并保存
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
    files.sort_by(|(a, _), (b, _)| a.file_name().cmp(&b.file_name()));
    dirs.sort_by(|a, b| a.file_name().cmp(&b.file_name()));

    (files, dirs)
}

/// 生成树形结构的前缀
fn generate_tree_prefix(is_last_items: &[bool]) -> String {
    let mut result = String::new();

    // Handle parent levels
    if !is_last_items.is_empty() {
        for &is_last in &is_last_items[..is_last_items.len() - 1] {
            if is_last {
                result.push_str("    "); // Space where vertical line would be
            } else {
                result.push_str("│   "); // Vertical line with space
            }
        }
    }

    // Handle current level
    if let Some(&is_last) = is_last_items.last() {
        if is_last {
            result.push_str("└── "); // Last item in its level
        } else {
            result.push_str("├── "); // Not the last item
        }
    }

    result
}

/// 主要的文件列表处理函数
pub fn list_files<P: AsRef<Path>>(
    indir: P,
    prefix: &str,
    is_last_items: &[bool],
    root: &Path,
    ext: &str,
    ignore_dirs: &[String],
    stats: Arc<Mutex<FileStats>>,
    min_size: u64,
) {
    let dir_path = indir.as_ref();

    if let Ok(entries) = fs::read_dir(dir_path) {
        // 预分配容量可以减少内存重新分配
        let entries: Vec<_> = entries.filter_map(Result::ok).collect();

        // 处理文件和目录条目
        let (files, dirs) = process_directory_entries(entries, ext, ignore_dirs, min_size);

        // 计算当前目录的统计信息
        let files_count = files.len();
        let total_size: u64 = files.iter().map(|(_, size)| *size).sum();

        // 先处理所有子目录，检查它们是否包含匹配的文件
        let mut subdirs_with_matches = Vec::new();
        let mut has_matching_subdirs = false;

        // 创建一个临时统计对象，用于检测子目录是否有匹配文件
        let subdirs_stats = Arc::new(Mutex::new(FileStats {
            total_files: 0,
            total_dirs: 0,
            total_bytes: 0,
        }));

        // 收集所有子目录并预处理它们来检查是否有匹配文件
        for (idx, dir) in dirs.iter().enumerate() {
            let is_last = idx == dirs.len() - 1;
            let mut new_is_last_items = is_last_items.to_vec();
            new_is_last_items.push(is_last);

            // 创建一个临时统计对象来检测这个子目录是否有匹配文件
            let subdir_stats = Arc::new(Mutex::new(FileStats {
                total_files: 0,
                total_dirs: 0,
                total_bytes: 0,
            }));

            // 预先检查子目录是否有匹配文件（不打印）
            check_subdir_for_matches(dir, ext, ignore_dirs, Arc::clone(&subdir_stats), min_size);

            let subdir_stats = subdir_stats.lock().unwrap();
            if subdir_stats.total_files > 0 {
                has_matching_subdirs = true;
                subdirs_with_matches.push((dir.clone(), is_last));

                // 添加到总体子目录统计中
                let mut total_stats = subdirs_stats.lock().unwrap();
                total_stats.total_files += subdir_stats.total_files;
                total_stats.total_bytes += subdir_stats.total_bytes;
            }
        }

        // 决定是否显示当前目录
        // 如果当前目录有文件或者它有包含匹配文件的子目录，则显示它
        let should_display = files_count > 0 || has_matching_subdirs;

        // 显示目录及其文件计数
        if should_display {
            let dir_name = if is_last_items.is_empty() {
                // 这是根目录
                dir_path.to_string_lossy().into_owned()
            } else {
                dir_path
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| dir_path.to_string_lossy().into_owned())
            };

            // 显示目录及其文件计数
            if !is_last_items.is_empty() {
                let tree_prefix = generate_tree_prefix(is_last_items);
                print!("{}{}{} ", prefix, tree_prefix, dir_name.blue().bold());
                if files_count > 0 {
                    println!("({}, {})",
                        format!("{} files", files_count).green(),
                        format_size(total_size).yellow());
                } else {
                    println!("");
                }
            } else {
                // 根目录特殊处理
                println!(
                    "Directory: {} ({}, {})",
                    dir_name.blue().bold(),
                    format!("{} files", files_count).green(),
                    format_size(total_size).yellow()
                );
            }

            // 更新总统计信息 - 添加当前目录的文件
            {
                let mut stats_guard = stats.lock().unwrap();
                stats_guard.total_files += files_count;
                stats_guard.total_dirs += 1; // 增加目录计数
                stats_guard.total_bytes += total_size;
            }
        }

        // 处理子目录
        let new_prefix = prefix.to_string();

        // 现在正常处理子目录
        for (idx, dir) in dirs.iter().enumerate() {
            let is_last = idx == dirs.len() - 1;
            let mut new_is_last_items = is_last_items.to_vec();
            new_is_last_items.push(is_last);

            // 正常处理子目录
            list_files(
                dir,
                &new_prefix,
                &new_is_last_items,
                root,
                ext,
                ignore_dirs,
                Arc::clone(&stats),
                min_size,
            );
        }
    } else {
        // 处理访问错误
        eprintln!("Error accessing directory: {}", dir_path.display());
    }
}

/// 检查子目录是否包含匹配文件而不打印任何输出
fn check_subdir_for_matches<P: AsRef<Path>>(
    dir: P,
    ext: &str,
    ignore_dirs: &[String],
    stats: Arc<Mutex<FileStats>>,
    min_size: u64,
) {
    let dir_path = dir.as_ref();

    if let Ok(entries) = fs::read_dir(dir_path) {
        let entries: Vec<_> = entries.filter_map(Result::ok).collect();
        let (files, dirs) = process_directory_entries(entries, ext, ignore_dirs, min_size);

        // 计算当前目录的统计信息
        let files_count = files.len();
        let total_size: u64 = files.iter().map(|(_, size)| *size).sum();

        // 更新统计信息
        {
            let mut stats_guard = stats.lock().unwrap();
            stats_guard.total_files += files_count;
            stats_guard.total_bytes += total_size;
        }

        // 递归检查所有子目录
        for subdir in dirs {
            check_subdir_for_matches(&subdir, ext, ignore_dirs, Arc::clone(&stats), min_size);
        }
    }
}
