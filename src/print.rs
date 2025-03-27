use colored::Colorize;

use crate::file_size::format_size;
use crate::tree::TreeNode;
use crate::FileStats;

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

/// 打印树结构 - 显示目录统计信息
pub fn print_tree_num(
    node: &TreeNode,
    prefix: &str,
    is_last_items: &[bool], 
    stats: &mut FileStats,
    include_children: bool,
) {
    match node {
        TreeNode::Directory { name, files, dirs, total_files, total_size, direct_files, direct_size, .. } => {
            // 使用新的字段，根据include_children选择显示方式
            let (_total_files, _total_size) = if include_children {
                (*total_files, *total_size)
            } else {
                (*direct_files, *direct_size)
            };

            // Display directory with file count and size
            if !is_last_items.is_empty() {
                let tree_prefix = generate_tree_prefix(is_last_items);
                print!("{}{}{} ", prefix, tree_prefix, name.blue().bold());
                
                if _total_files > 0 {
                    print!(
                        "({}, {})",
                        format!("{} files", _total_files).green(),
                        format_size(_total_size).yellow()
                    );
                }
            } else {
                // Root directory special handling
                print!("Directory: {} ", name.blue().bold());
                
                if _total_files > 0 {
                    print!(
                        "({}, {})",
                        format!("{} files", _total_files).green(),
                        format_size(_total_size).yellow()
                    );
                }
            }
            println!();
            
            // Update statistics
            stats.total_dirs += 1;
            stats.total_files += files.len(); // 直接使用files数组的长度
            stats.total_bytes += *total_size;
            
            // 处理子目录
            for (idx, child) in dirs.iter().enumerate() {
                let is_last = idx == dirs.len() - 1;
                let mut new_is_last_items = is_last_items.to_vec();
                new_is_last_items.push(is_last);
                
                print_tree_num(child, prefix, &new_is_last_items, stats, include_children);
            }
        },
        TreeNode::File { .. } => {
            // 文件节点不单独打印
        }
    }
}

/// 打印树结构 - 显示目录和文件完整结构
pub fn print_tree_file(
    node: &TreeNode,
    prefix: &str,
    is_last_items: &[bool], 
    stats: &mut FileStats,
) {
    match node {
        TreeNode::Directory { name, path, files, dirs, total_size, .. } => {
            // 显示目录名
            if !is_last_items.is_empty() {
                let tree_prefix = generate_tree_prefix(is_last_items);
                println!("{}{}{} ({})", prefix, tree_prefix, name.blue().bold(), format_size(*total_size).yellow());
            } else {
                // 根目录特殊处理
                println!("Directory: {} ({})", name.blue().bold(), format_size(*total_size).yellow());
            }
            
            // 更新统计信息
            stats.total_dirs += 1;
            stats.total_bytes += *total_size;
            
            // 先处理文件
            let total_items = files.len() + dirs.len();
            let mut current_idx = 0;
            
            for file in files {
                let is_last = current_idx == total_items - 1;
                let mut new_is_last_items = is_last_items.to_vec();
                new_is_last_items.push(is_last);
                
                print_tree_file(file, prefix, &new_is_last_items, stats);
                current_idx += 1;
            }
            
            // 再处理目录
            for dir in dirs {
                let is_last = current_idx == total_items - 1;
                let mut new_is_last_items = is_last_items.to_vec();
                new_is_last_items.push(is_last);
                
                print_tree_file(dir, prefix, &new_is_last_items, stats);
                current_idx += 1;
            }
        },
        TreeNode::File { name, path, size } => {
            // 显示文件名和大小
            let tree_prefix = generate_tree_prefix(is_last_items);
            
            // 检查是否是可执行文件（.exe扩展名）
            let is_exe = path.extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.eq_ignore_ascii_case("exe"))
                .unwrap_or(false);
            
            // 根据文件类型选择颜色
            let colored_name = if is_exe {
                name.green()
            } else {
                name.normal()
            };
            
            println!("{}{}{} ({})", prefix, tree_prefix, colored_name, format_size(*size).yellow());
            
            // 更新统计信息
            stats.total_files += 1;
            stats.total_bytes += *size;
        }
    }
}
