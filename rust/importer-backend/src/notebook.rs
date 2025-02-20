use crate::api::Api;
use crate::block::{update_node_blockquote, update_node_math_block, update_node_paragraph};
use anyhow::Result;
use serde_json::Value;
use std::sync::{Arc, Mutex};
use tokio::fs;
use tokio::runtime::Runtime;

async fn update_data(data: &Value, api: &Api) -> Result<()> {
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

#[allow(dead_code)]
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

pub struct Notebook {
    api: Arc<Mutex<Api>>,
}

/// 流程:
///
/// 1. remote确认base_url
/// 2. remote确认data_home
/// 3. 返回notebook列表到remote
/// 4. remote确认notebook_name
/// 5. 返回需要处理文件列表
/// 6. remote传输指定文件本地完成更新
///
impl Notebook {
    pub fn new(data_home: &str, base_url: &str) -> Result<Self> {
        let rt = Runtime::new()?;
        let api = rt.block_on(Api::new2(data_home, base_url));
        Ok(Self {
            api: Arc::new(Mutex::new(api)),
        })
    }

    pub fn get_notebook_names(&self) -> Result<Vec<String>> {
        let api = self.api.lock().unwrap();
        let rt = Runtime::new()?;
        let names = rt.block_on(api.get_notebook_names())?;
        Ok(names)
    }

    pub fn set_notebook_name(&self, name: &str) -> Result<()> {
        let mut api = self.api.lock().unwrap();
        api.notebook_name = Some(String::from(name));
        Ok(())
    }

    pub fn get_all_files(&self) -> Result<Vec<String>> {
        let mut api = self.api.lock().unwrap();
        let rt = Runtime::new()?;
        let names = rt.block_on(api.get_all_sy_files())?;
        let names = names
            .iter()
            .filter_map(|item| item.to_str().map(|item| item.to_string()))
            .collect::<Vec<_>>();
        Ok(names)
    }

    pub fn process_file(&self, path: &str) -> Result<()> {
        let api = self.api.lock().unwrap();
        let rt = Runtime::new()?;
        rt.block_on(async move {
            let data = fs::read_to_string(path).await?;
            let data: Value = serde_json::from_str(&data)?;
            update_data(&data, &api).await?;
            Ok::<(), anyhow::Error>(())
        })?;
        Ok(())
    }
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
