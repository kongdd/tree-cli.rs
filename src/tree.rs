use std::path::PathBuf;

// 修改树结构，将children分为files和dirs
pub enum TreeNode {
    File {
        name: String,
        path: PathBuf,
        size: u64,
    },
    Directory {
        name: String,
        files: Vec<TreeNode>,    // 只存储文件节点
        dirs: Vec<TreeNode>,     // 只存储目录节点
        total_files: usize,      // 包含子目录的总文件数
        total_size: u64,         // 包含子目录的总大小
        direct_files: usize,     // 仅当前目录文件数
        direct_size: u64,        // 仅当前目录文件大小
    },
}

impl TreeNode {
    // 将方法改为公有
    pub fn new_file(path: PathBuf, size: u64) -> Self {
        let name = path.file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.to_string_lossy().into_owned());
        TreeNode::File { name, path, size }
    }

    // 将方法改为公有并更新为新结构
    pub fn new_directory(path: PathBuf) -> Self {
        let name = path.file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.to_string_lossy().into_owned());
        TreeNode::Directory { 
            name, 
            files: Vec::new(),
            dirs: Vec::new(),
            total_files: 0,
            total_size: 0,
            direct_files: 0,
            direct_size: 0,
        }
    }
}
