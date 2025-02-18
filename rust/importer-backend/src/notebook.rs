use crate::api::Api;
use crate::block::{update_node_blockquote, update_node_math_block, update_node_paragraph};
use anyhow::Result;
use serde_json::Value;
use tokio::fs;

async fn update_data(data: &Value, api: &Api) -> Result<()> {
    // TODO: debug
    let raw_data = data.to_string();
    if let Some(data_type) = data["Type"].as_str() {
        match data_type {
            "NodeParagraph" => {
                if let Some(idx) = data["ID"].as_str() {
                    let markdown_data = api.get_block_kramdown(idx).await?;
                    let markdown_data = update_node_paragraph(&markdown_data);
                    api.update_block(&markdown_data, idx).await?;
                }
            }
            "NodeMathBlock" => {
                if let Some(idx) = data["ID"].as_str() {
                    let markdown_data = api.get_block_kramdown(idx).await?;
                    let markdown_data = update_node_math_block(&markdown_data);
                    api.update_block(&markdown_data, idx).await?;
                }
            }
            "NodeBlockquote" => {
                if let Some(idx) = data["ID"].as_str() {
                    let markdown_data = api.get_block_kramdown(idx).await?;
                    let markdown_data = update_node_blockquote(&markdown_data)?;
                    api.update_block(&markdown_data, idx).await?;
                }
            }
            _ => {
                if let Some(children) = data.get("Children") {
                    if let Some(children) = children.as_array() {
                        for child in children {
                            Box::pin(update_data(child, api)).await?;
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

pub(crate) async fn update_notebook(
    notebook_name: &str,
    data_home: &str,
    base_url: Option<&str>,
) -> Result<()> {
    let base_url = base_url.unwrap_or("http://127.0.0.1:6806");
    let mut api = Api::new(notebook_name, data_home, base_url).await;

    let files = api.get_all_sy_files().await?;
    for file in files {
        let data = fs::read_to_string(file).await?;
        let data: Value = serde_json::from_str(&data)?;
        update_data(&data, &api).await?
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_update_notebook() {
        let res = update_notebook(
            "test-notion",
            "/Users/max/Documents/SiYuanTest/data",
            Some("http://127.0.0.1:54837"),
        )
        .await;
        println!("{:?}", res);
    }
}
