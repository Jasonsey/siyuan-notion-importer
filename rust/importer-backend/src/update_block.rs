use fancy_regex::Regex;

/// 更新inline math
fn update_node_paragraph(data: &str) -> String {
    // 移除最后一行
    let data = data
        .lines()
        .take(data.lines().count() - 1)
        .collect::<Vec<&str>>()
        .join("\n");

    // 替换掉 \\(\\*) 为 \1
    let data = if data.contains(r"\$") {
        let re = Regex::new(r"\\(\\*)").unwrap();
        re.replace_all(&data, r"$1").to_string()
    } else {
        data
    };

    // 优化图片显示的正则表达式替换
    let re = Regex::new(r"!?\[(.*?)]\((.*?)\.(bmp|jpg|png|tif|gif|pcx|tga|exif|fpx|svg|psd|cdr|pcd|dxf|ufo|eps|ai|raw|WMF|webp|jpeg)\)").unwrap();
    let data = re.replace_all(&data, r"![${1}](${2}.${3})").to_string();

    data
}

/// 更新match block
fn update_node_math_block(data: &str) -> String {
    // 移除字符串末尾的换行符
    let data = data
        .lines()
        .take(data.lines().count() - 1)
        .collect::<Vec<&str>>()
        .join("\n");

    // 使用正则表达式替换指定的模式
    let re = Regex::new(r"\$\$\n\$*(.*?)\$*\n\$\$").unwrap();
    let data = re.replace_all(&data, "$$$$\n$1\n$$$$").to_string();

    data
}

/// 更新注视代码
fn update_node_blockquote(data: &str) -> String {
    let mut res = Vec::new();

    let important_re =
        Regex::new(r"(?s)> \[!important](.*?)(?:\[!.*?]|\{: .*?=.*? .*?=.*?})").unwrap();
    let info_re = Regex::new(r"(?s)> \[!info](.*?)(?:\[!.*?]|\{: .*?=.*? .*?=.*?})").unwrap();
    let url_re = Regex::new(r"\[.*?]\((.*?)\)").unwrap();

    for item in important_re.captures_iter(data) {
        if let Ok(item) = item {
            if let Some(m) = item.get(1) {
                let data = format!("> {}", m.as_str().trim());
                res.push(data);
            }
        }
    }
    for item in info_re.captures_iter(data) {
        if let Ok(item) = item {
            if let Some(m) = item.get(1) {
                let data = m.as_str();
                let lines = data.split('\n').collect::<Vec<_>>();
                let title = lines[0].trim();

                let mut url_data = title.to_string();
                if let Ok(url_cap) = url_re.captures(data) {
                    if let Some(url_cap) = url_cap {
                        if let Some(m) = url_cap.get(1) {
                            url_data = format!("[{}]({})", title, m.as_str())
                        }
                    }
                }
                res.push(url_data);
            }
        }
    }

    if res.len() > 0 {
        res.join("\n\n")
    } else {
        data.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_node_math_block() {
        let input = "Some text before\n$$\n$ some math block $\n$$\nSome text after";
        let updated_data = update_node_math_block(input);
        println!("Updated data:\n{}", updated_data);
    }

    #[test]
    fn test_update_node_blockquote() {
        let data = r#"
> [!important] This is important content
> This is important content 2
> [!info] This is info content with 3
> data [link](http://example.com)
> [!info] 544335 info
> data [link](http://example.com)
    "#;
        let updated_data = update_node_blockquote(data);
        println!("Updated data:\n{}", updated_data);
    }
}
