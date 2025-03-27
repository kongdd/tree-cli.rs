use rayon::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use crate::tree::TreeNode;
use crate::print::{print_tree_num, print_tree_file}; // 修改导入

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

/// 构建文件系统的树结构
pub fn build_directory_tree<P: AsRef<Path>>(
    dir_path: P, 
    ext: &str,
    ignore_dirs: &[String],
    min_size: u64
) -> Option<TreeNode> {
    let dir_path = dir_path.as_ref();
    
    match fs::read_dir(dir_path) {
        Ok(entries) => {
            let entries: Vec<_> = entries.filter_map(Result::ok).collect();
            let (files, dirs) = process_directory_entries(entries, ext, ignore_dirs, min_size);
            
            // Create a directory node
            let mut dir_node = TreeNode::new_directory(dir_path.to_path_buf());
            
            // 先统计当前目录的直接文件
            let direct_files = files.len();
            let direct_size: u64 = files.iter().map(|(_, size)| *size).sum();
            
            // 初始化total等于direct的值
            let mut total_files = direct_files;
            let mut total_size = direct_size;
            
            // Process files
            for (file_path, file_size) in files {
                if let TreeNode::Directory { files, .. } = &mut dir_node {
                    files.push(TreeNode::new_file(file_path, file_size));
                }
            }
            
            // Process subdirectories
            for subdir_path in dirs {
                if let Some(subdir_node) = build_directory_tree(subdir_path, ext, ignore_dirs, min_size) {
                    // Only add directories that have files (directly or in subdirs)
                    let has_files = match &subdir_node {
                        TreeNode::Directory { total_files, .. } => *total_files > 0,
                        _ => false,
                    };
                    
                    if has_files {
                        if let TreeNode::Directory { dirs, .. } = &mut dir_node {
                            // Update total counts by adding subdir values
                            if let TreeNode::Directory { total_files: subdir_files, total_size: subdir_size, .. } = &subdir_node {
                                total_files += subdir_files;
                                total_size += subdir_size;
                            }
                            dirs.push(subdir_node);
                        }
                    }
                }
            }
            
            // Update directory stats
            if let TreeNode::Directory { 
                total_files: ref mut tf, 
                total_size: ref mut ts, 
                direct_files: ref mut df, 
                direct_size: ref mut ds, 
                .. 
            } = dir_node {
                *tf = total_files;
                *ts = total_size;
                *df = direct_files;
                *ds = direct_size;
            }
            
            // Only return directory if it has files (directly or in subdirs)
            match &dir_node {
                TreeNode::Directory { total_files, .. } if *total_files > 0 => Some(dir_node),
                _ => None,
            }
        },
        Err(_) => {
            eprintln!("Error accessing directory: {}", dir_path.display());
            None
        }
    }
}

/// 主要的文件列表处理函数
pub fn list_files<P: AsRef<Path>>(
    indir: P,
    prefix: &str,
    is_last_items: &[bool],
    _root: &Path,  // 添加下划线前缀表明这是一个有意未使用的变量
    ext: &str,
    ignore_dirs: &[String],
    stats: Arc<Mutex<FileStats>>,
    min_size: u64,
    include_children: bool,
    show_stats_only: bool, // 新增参数
) {
    // Build the tree structure
    if let Some(tree) = build_directory_tree(indir, ext, ignore_dirs, min_size) {
        if !show_stats_only {  // 修复括号错误
            // 使用print_tree_file打印完整的文件树结构
            let mut local_stats = FileStats {
                total_files: 0,
                total_dirs: 0,
                total_bytes: 0,
            };
            
            print_tree_file(&tree, prefix, is_last_items, &mut local_stats);
            
            // Update the global stats
            let mut stats_guard = stats.lock().unwrap();
            stats_guard.total_files += local_stats.total_files;
            stats_guard.total_dirs += local_stats.total_dirs;
            stats_guard.total_bytes += local_stats.total_bytes;
        } else {
            // 使用print_tree_num打印目录统计信息
            let mut local_stats = FileStats {
                total_files: 0,
                total_dirs: 0,
                total_bytes: 0,
            };
            
            print_tree_num(&tree, prefix, is_last_items, &mut local_stats, include_children);
            
            // Update the global stats
            let mut stats_guard = stats.lock().unwrap();
            stats_guard.total_files += local_stats.total_files;
            stats_guard.total_dirs += local_stats.total_dirs;
            stats_guard.total_bytes += local_stats.total_bytes;
        }
    }
}
