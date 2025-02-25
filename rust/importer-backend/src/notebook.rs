use crate::api::Api;
use crate::block::{update_node_blockquote, update_node_math_block, update_node_paragraph};
use anyhow::Result;
use serde_json::Value;
use std::sync::Arc;
use tokio::fs;
use tokio::runtime::Runtime;
use tokio::sync::Mutex;

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
pub(crate) async fn update_notebook(notebook_name: &str, base_url: Option<&str>) -> Result<()> {
    let base_url = base_url.unwrap_or("http://127.0.0.1:6806");
    let mut api = Api::new(base_url);
    api.set_notebook_name(notebook_name).await?;

    let files = api.get_all_sy_files().await?;
    for file in files {
        let data = api.get_file(&file).await?;
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
    pub fn new(base_url: &str) -> Result<Self> {
        let api = Api::new(base_url);
        Ok(Self {
            api: Arc::new(Mutex::new(api)),
        })
    }

    pub fn get_notebook_names(&self) -> Result<Vec<String>> {
        let rt = Runtime::new()?;
        let api = Arc::clone(&self.api);
        let names = rt.block_on(async {
            let api = api.lock().await;
            api.get_notebook_names().await
        })?;
        Ok(names)
    }

    pub fn set_notebook_name(&self, name: &str) -> Result<()> {
        let rt = Runtime::new()?;
        let api = Arc::clone(&self.api);
        rt.block_on(async {
            let mut api = api.lock().await;
            api.notebook_name = Some(String::from(name));
        });
        Ok(())
    }

    pub fn get_all_files(&self) -> Result<Vec<String>> {
        let rt = Runtime::new()?;
        let api = Arc::clone(&self.api);
        let names = rt.block_on(async {
            let api = api.lock().await;
            api.get_all_sy_files().await
        })?;
        Ok(names)
    }

    pub fn process_file(&self, path: &str) -> Result<()> {
        let rt = Runtime::new()?;
        let api = Arc::clone(&self.api);
        rt.block_on(async {
            let api = api.lock().await;
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
        let res = update_notebook("test-notion", Some("http://127.0.0.1:54113")).await;
        println!("{:?}", res);
    }
}
