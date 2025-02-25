use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use reqwest::StatusCode;
use tokio::sync::Semaphore;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Notebook {
    id: String,
    name: String,
    icon: String,
    sort: u8,
    closed: bool,
}

/// json后的例子数据：
///
/// ```json
/// {
///   "isDir": false,
///   "isSymlink": false,
///   "name": "20210808180303-6yi0dv5.sy",
///   "updated": 1663298365
/// }
/// ```
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FileInfo {
    is_dir: bool,
    is_symlink: bool,
    name: String,
    updated: i32,
}

#[derive(Debug, Serialize, Deserialize)]
struct ResponseData<T> {
    code: i32,
    msg: String,
    data: T,
}

#[derive(Debug)]
pub(crate) struct Api {
    pub(crate) notebook_name: Option<String>,
    base_url: String,
    notebook_home: Option<PathBuf>,
    sem: Semaphore,
}

impl Default for Api {
    fn default() -> Self {
        Self::new("http://127.0.0.1:6806")
    }
}

#[allow(dead_code)]
impl Api {
    /// 参考配置：
    ///
    /// ```text
    /// notebook_name: str = 'notion'
    /// base_url: str = "http://127.0.0.1:6806"
    /// headers: Dict[str, str] = field(default_factory=lambda: {"Content-Type": "application/json"})
    /// _notebook_home: Optional[str] = None
    /// _sem: Semaphore = Semaphore(500)
    /// ```
    pub(crate) fn new(base_url: &str) -> Self {
        Api {
            notebook_name: None,
            base_url: base_url.to_string(),
            notebook_home: None,
            sem: Semaphore::new(500),
        }
    }
}

/// 原始api
#[allow(dead_code)]
impl Api {
    pub async fn list_notebooks(&self) -> Result<Vec<Notebook>> {
        let _permit = self.sem.acquire().await?;
        let client = reqwest::Client::new();
        let url = format!("{}/api/notebook/lsNotebooks", self.base_url);
        let response = client.post(&url).send().await?;
        let res: ResponseData<HashMap<String, Vec<Notebook>>> = response.json().await?;
        if res.code != 0 {
            return Err(anyhow!("Error listing notebooks"));
        }
        let notebooks = res.data["notebooks"].clone();
        Ok(notebooks)
    }

    pub(crate) async fn get_filepath_by_id(&self, idx: &str) -> Result<String> {
        let _permit = self.sem.acquire().await?;
        let client = reqwest::Client::new();
        let url = format!("{}/api/filetree/getPathByID", self.base_url);
        let payload = json!({"id": idx});
        let response = client.post(&url).json(&payload).send().await?;
        let res: ResponseData<String> = response.json().await?;
        if res.code != 0 {
            Err(anyhow!("Error getting filepath by ID"))
        } else {
            Ok(res.data)
        }
    }

    pub(crate) async fn update_block(&self, data: &str, idx: &str) -> Result<()> {
        let _permit = self.sem.acquire().await?;
        let client = reqwest::Client::new();
        let url = format!("{}/api/block/updateBlock", self.base_url);
        let payload = json!({"data": data, "dataType": "markdown", "id": idx});
        let response = client.post(&url).json(&payload).send().await?;
        let res: ResponseData<Value> = response.json().await?;
        if res.code != 0 {
            Err(anyhow!("Error updating block: {}, msg: {}", idx, res.msg))
        } else {
            Ok(())
        }
    }

    pub(crate) async fn get_block_kramdown(&self, idx: &str) -> Result<String> {
        let _permit = self.sem.acquire().await?;
        let client = reqwest::Client::new();
        let url = format!("{}/api/block/getBlockKramdown", self.base_url);
        let payload = json!({"id": idx});
        let response = client.post(&url).json(&payload).send().await?;
        let res: ResponseData<Value> = response
            .json()
            .await
            .map_err(|e| anyhow!("parse response error: {}", e))?;
        if res.code != 0 {
            Err(anyhow!("Error getting block Kramdown"))
        } else {
            let raw_data = res.data.to_string();
            let kramdown_data = res.data["kramdown"].as_str().unwrap_or("").to_string();
            if kramdown_data == "" {
                println!("{}", raw_data)
            }
            Ok(kramdown_data)
        }
    }

    pub(crate) async fn insert_block(
        &self,
        data: &str,
        next_id: Option<&str>,
        previous_id: Option<&str>,
        parent_id: Option<&str>,
    ) -> Result<String> {
        let _permit = self.sem.acquire().await?;
        let client = reqwest::Client::new();
        let url = format!("{}/api/insertBlock", self.base_url);
        let next_id = next_id.unwrap_or("");
        let previous_id = previous_id.unwrap_or("");
        let parent_id = parent_id.unwrap_or("");
        let payload = json!({"data": data, "nextID": next_id, "previousID": previous_id, "parentID": parent_id, "dataType": "markdown"});
        let response = client.post(&url).json(&payload).send().await?;
        let res: ResponseData<Value> = response.json().await?;
        if res.code != 0 {
            Err(anyhow!("Error inserting block: {}", res.msg))
        } else {
            let new_idx = res.data["doOperations"]["id"].to_string();
            Ok(new_idx)
        }
    }

    pub(crate) async fn delete_block(&self, idx: &str) -> Result<()> {
        let _permit = self.sem.acquire().await?;
        let client = reqwest::Client::new();
        let url = format!("{}/api/deleteBlock", self.base_url);
        let payload = json!({"id": idx});
        let response = client.post(&url).json(&payload).send().await?;
        let res: ResponseData<()> = response.json().await?;
        if res.code != 0 {
            Err(anyhow!("Error deleting block"))
        } else {
            Ok(())
        }
    }

    /// 读取工作空间下某文件夹下所有文件, 包含嵌套结构
    ///
    /// 返回例子(其中的data部分是返回值)：
    ///
    /// ```json
    /// {
    ///   "code": 0,
    ///   "msg": "",
    ///   "data": [
    ///     {
    ///       "isDir": true,
    ///       "isSymlink": false,
    ///       "name": "20210808180303-6yi0dv5",
    ///       "updated": 1691467624
    ///     },
    ///     {
    ///       "isDir": false,
    ///       "isSymlink": false,
    ///       "name": "20210808180303-6yi0dv5.sy",
    ///       "updated": 1663298365
    ///     }
    ///   ]
    /// }
    /// ```
    pub(crate) async fn read_dir(&self, path: &str) -> Result<Vec<FileInfo>> {
        let _permit = self.sem.acquire().await?;
        let client = reqwest::Client::new();
        let url = format!("{}/api/file/readDir", self.base_url);
        let payload = json!({"path": path});
        let response = client.post(&url).json(&payload).send().await?;
        let res: ResponseData<Vec<FileInfo>> = response.json().await?;
        if res.code != 0 {
            Err(anyhow!("Error reading dir: {}", res.msg))
        } else {
            Ok(res.data)
        }
    }

    /// 读取sy文件, 跟目录是siyuan工作目录，例如: `/data/20210808180117-6v0mkxr/20200923234011-ieuun1p.sy`
    pub(crate) async fn get_file(&self, path: &str) -> Result<String> {
        let _permit = self.sem.acquire().await?;
        let client = reqwest::Client::new();
        let url = format!("{}/api/file/getFile", self.base_url);
        let payload = json!({"path": path});
        let response = client.post(&url).json(&payload).send().await?;

        if response.status() == StatusCode::OK {
            let res = response.text().await?;
            Ok(res)
        } else {
            Err(anyhow!("Error getting file: {}", path))
        }
    }
}

/// 拓展API
impl Api {
    pub(crate) async fn get_notebook_names(&self) -> Result<Vec<String>> {
        let notebooks = self.list_notebooks().await?;
        let names = notebooks
            .iter()
            .map(|item| item.name.clone())
            .collect::<Vec<_>>();
        Ok(names)
    }

    pub(crate) async fn set_notebook_name(&mut self, name: &str) -> Result<()> {
        let notebooks = self.list_notebooks().await?;
        for notebook in notebooks {
            if notebook.name == name {
                self.notebook_name = Some(notebook.name.clone());
                self.notebook_home = Some(Path::new("/data").join(notebook.id.clone()));
                break;
            }
        }
        Ok(())
    }

    pub(crate) async fn get_all_sy_files(&self) -> Result<Vec<String>> {
        if let Some(notebook_home) = &self.notebook_home {
            let notebook_home = notebook_home.to_str().unwrap();
            let files = self.read_dir_all(notebook_home).await?;
            Ok(files)
        } else {
            Err(anyhow!("No notebooks found. Please call `set_notebook_name` first`"))
        }
    }

    pub(crate) async fn read_dir_all(&self, path: &str) -> Result<Vec<String>> {
        let mut sy_files = vec![];
        let mut dirs = vec![path.to_string()];
        while let Some(path) = dirs.pop() {
            let files = self.read_dir(&path).await?;
            for file in files {
                let current_path = Path::new(&path)
                    .join(&file.name)
                    .to_str()
                    .unwrap()
                    .to_string();
                if file.is_dir {
                    dirs.push(current_path);
                } else {
                    sy_files.push(current_path);
                }
            }
        }
        Ok(sy_files)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_list_notebooks() -> Result<()> {
        let api = Api::default();
        let res = api.list_notebooks().await?;
        println!("{:#?}", res);
        Ok(())
    }

    #[tokio::test]
    async fn test_get_block_kramdown() -> Result<()> {
        let api = Api::default();
        let res = api.get_block_kramdown("20250203215609-fl3g10b").await?;
        println!("{}", res);
        Ok(())
    }
}
