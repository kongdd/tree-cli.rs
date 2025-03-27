use std::path::PathBuf;

// New tree structure to represent file system
pub enum TreeNode {
    File {
        name: String,
        path: PathBuf,
        size: u64,
    },
    Directory {
        name: String,
        path: PathBuf,
        children: Vec<TreeNode>,
        total_files: usize,
        total_size: u64,
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

    // 将方法改为公有
    pub fn new_directory(path: PathBuf) -> Self {
        let name = path.file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.to_string_lossy().into_owned());
        TreeNode::Directory { 
            name, 
            path, 
            children: Vec::new(),
            total_files: 0,
            total_size: 0,
        }
    }

    // // Get name of the node
    // fn name(&self) -> &str {
    //     match self {
    //         TreeNode::File { name, .. } => name,
    //         TreeNode::Directory { name, .. } => name,
    //     }
    // }

    // // Get path of the node
    // fn path(&self) -> &PathBuf {
    //     match self {
    //         TreeNode::File { path, .. } => path,
    //         TreeNode::Directory { path, .. } => path,
    //     }
    // }
}
