use std::fs;
use std::path::Path;
use std::env;
use colored::*;
use std::os::windows::fs::MetadataExt;

fn is_file<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref().exists() && path.as_ref().is_file()
}

fn is_dir<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref().exists() && path.as_ref().is_dir()
}

fn list_files<P: AsRef<Path>>(indir: P, prefix: &str, root: &Path, ext: &str) {
    let dir_path = indir.as_ref();
    
    if let Ok(entries) = fs::read_dir(dir_path) {
        let mut files = Vec::new();
        let mut dirs = Vec::new();
        
        for entry in entries.flatten() {
            let path = entry.path();
            if is_file(&path) {
                let is_hidden = if cfg!(windows) {
                    let attr = fs::metadata(&path).unwrap().file_attributes();
                    (attr & 0x2) != 0
                } else {
                    path.file_name()
                        .and_then(|n| n.to_str())
                        .map(|s| s.starts_with('.')).unwrap_or(false)
                };
                if !is_hidden && (ext.is_empty() || path.extension().and_then(|e| e.to_str()) == Some(ext)) {
                    files.push(path);
                }
            } else if is_dir(&path) {
                let is_hidden = if cfg!(windows) {
                    let attr = fs::metadata(&path).unwrap().file_attributes();
                    (attr & 0x2) != 0
                } else {
                    path.file_name()
                       .and_then(|n| n.to_str())
                       .map(|s| s.starts_with('.') || s.starts_with('.')).unwrap_or(false)
                };
                if !is_hidden &&!path.file_name().and_then(|n| n.to_str()).map(|s| s.starts_with('.')).unwrap_or(false) {
                    dirs.push(path);
                }
            }
        }
        
        let n = files.len();
        let path_str = dir_path.to_string_lossy();
        
        if n > 0 {
            println!("{}{} | {}", prefix, path_str, n.to_string().green());
        }
        
        let new_prefix = format!("{}  ", prefix);
        for dir in dirs {
            list_files(&dir, &new_prefix, root, ext);
        }
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
    
    let mut dir_path = ".";  // Default to current directory
    let mut ext = "";        // Default to no extension filter
    
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
    
    println!("Counting files in directory: {}", dir_path);
    if !ext.is_empty() {
        println!("Filtering by extension: {}", ext);
    }
    
    let path = Path::new(dir_path);
    list_files(path, "  ", path, ext);
}
