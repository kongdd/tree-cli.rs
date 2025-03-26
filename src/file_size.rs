// 格式化文件大小显示
pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}

// 将字符串解析为字节大小
pub fn parse_size(size_str: &str) -> Result<u64, String> {
    let size_str = size_str.trim().to_lowercase();
    let mut numeric_part = String::new();
    let mut unit_part = String::new();

    for c in size_str.chars() {
        if c.is_digit(10) || c == '.' {
            numeric_part.push(c);
        } else if !c.is_whitespace() {
            unit_part.push(c);
        }
    }

    let value: f64 = match numeric_part.parse() {
        Ok(v) => v,
        Err(_) => return Err(format!("Invalid numeric value in size: {}", size_str)),
    };

    let multiplier = match unit_part.as_str() {
        "" => 1, // 默认为字节
        "b" | "bytes" => 1,
        "k" | "kb" | "kib" => 1024,
        "m" | "mb" | "mib" => 1024 * 1024,
        "g" | "gb" | "gib" => 1024 * 1024 * 1024,
        _ => return Err(format!("Unknown size unit: {}", unit_part)),
    };

    Ok((value * multiplier as f64) as u64)
}
