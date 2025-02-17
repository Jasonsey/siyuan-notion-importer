use anyhow::{Result, anyhow};
use reqwest::header::HeaderMap;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use glob::glob;
use tokio::sync::Semaphore;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Notebook {
    id: String,
    name: String,
    icon: String,
    sort: u8,
    closed: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct ResponseData<T> {
    code: u8,
    msg: String,
    data: T,
}

#[derive(Debug)]
struct Api {
    notebook_name: String,
    data_home: String,
    base_url: String,
    headers: HeaderMap,
    _notebook_home: Option<PathBuf>,
    _sem: Semaphore,
}

impl Api {
    async fn default() -> Self {
        Self::new(
            "test-notion",
            "/Users/max/SiYuan/data",
            "http://127.0.0.1:6806",
        )
        .await
    }
    /// 参考配置：
    ///
    /// ```text
    /// notebook_name: str = 'notion'
    /// data_home: str = "/Users/max/SiYuan/data"
    /// base_url: str = "http://127.0.0.1:6806"
    /// headers: Dict[str, str] = field(default_factory=lambda: {"Content-Type": "application/json"})
    /// _notebook_home: Optional[str] = None
    /// _sem: Semaphore = Semaphore(500)
    /// ```
    async fn new(notebook_name: &str, data_home: &str, base_url: &str) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", "application/json".parse().unwrap());

        Api {
            notebook_name: notebook_name.to_string(),
            data_home: data_home.to_string(),
            base_url: base_url.to_string(),
            headers,
            _notebook_home: None,
            _sem: Semaphore::new(500),
        }
    }

    async fn notebook_home(&mut self) -> Result<PathBuf> {
        if self._notebook_home.is_none() {
            let notebooks = self.list_notebooks().await?;
            for notebook in notebooks {
                if notebook.name == self.notebook_name {
                    self._notebook_home = Some(Path::new(&self.data_home).join(notebook.id));
                    break;
                }
            }
            if self._notebook_home.is_none() {
                return Err(anyhow!("No notebook named"));
            }
        }
        Ok(self._notebook_home.clone().unwrap())
    }

    async fn get_all_sy_files(&mut self) -> Result<Vec<PathBuf>> {
        let path = self.notebook_home().await?;
        let pattern = path.join("**/*.sy").to_string_lossy().to_string();
        let mut sy_files = Vec::new();
        for entry in glob(&pattern)?.filter_map(Result::ok) {
            sy_files.push(entry);
        }
        Ok(sy_files)
    }

    async fn list_notebooks(&self) -> Result<Vec<Notebook>> {
        let _permit = self._sem.acquire().await?;
        let client = reqwest::Client::new();
        let url = format!("{}/api/notebook/lsNotebooks", self.base_url);
        let response = client
            .post(&url)
            .headers(self.headers.clone())
            .send()
            .await?;
        let res: ResponseData<HashMap<String, Vec<Notebook>>> = response.json().await?;
        if res.code != 0 {
            return Err(anyhow!("Error listing notebooks"));
        }
        let notebooks = res.data["notebooks"].clone();
        Ok(notebooks)
    }

    async fn get_filepath_by_id(&self, idx: &str) -> Result<String> {
        let _permit = self._sem.acquire().await?;
        let client = reqwest::Client::new();
        let url = format!("{}/api/filetree/getPathByID", self.base_url);
        let payload = json!({"id": idx});
        let response = client
            .post(&url)
            .json(&payload)
            .headers(self.headers.clone())
            .json(&payload)
            .send()
            .await?;
        let res: ResponseData<String> = response.json().await?;
        if res.code != 0 {
            Err(anyhow!("Error getting filepath by ID"))
        } else {
            Ok(res.data)
        }
    }

    async fn update_block(&self, data: &str, idx: &str) -> Result<()> {
        let _permit = self._sem.acquire().await?;
        let client = reqwest::Client::new();
        let url = format!("{}/api/block/updateBlock", self.base_url);
        let payload = json!({"data": data, "dataType": "markdown", "id": idx});
        let response = client
            .post(&url)
            .json(&payload)
            .headers(self.headers.clone())
            .send()
            .await?;
        let res: ResponseData<()> = response.json().await?;
        if res.code != 0 {
            Err(anyhow!("Error updating block"))
        } else {
            Ok(())
        }
    }

    async fn get_block_kramdown(&self, idx: &str) -> Result<String> {
        let _permit = self._sem.acquire().await?;
        let client = reqwest::Client::new();
        let url = format!("{}/api/block/getKramdown", self.base_url);
        let payload = json!({"id": idx});
        let response = client
            .post(&url)
            .json(&payload)
            .headers(self.headers.clone())
            .send()
            .await?;
        let res: ResponseData<HashMap<String, String>> = response.json().await?;
        if res.code != 0 {
            Err(anyhow!("Error getting block Kramdown"))
        } else {
            let kramdown_data = res.data.get("kramdown").unwrap().clone();
            Ok(kramdown_data)
        }
    }

    async fn insert_block(
        &self,
        data: &str,
        next_id: Option<&str>,
        previous_id: Option<&str>,
        parent_id: Option<&str>,
    ) -> Result<String> {
        let _permit = self._sem.acquire().await?;
        let client = reqwest::Client::new();
        let url = format!("{}/api/insertBlock", self.base_url);
        let next_id = next_id.unwrap_or("");
        let previous_id = previous_id.unwrap_or("");
        let parent_id = parent_id.unwrap_or("");
        let payload = json!({"data": data, "nextID": next_id, "previousID": previous_id, "parentID": parent_id, "dataType": "markdown"});
        let response = client
            .post(&url)
            .json(&payload)
            .headers(self.headers.clone())
            .send()
            .await?;
        let res: ResponseData<Value> = response.json().await?;
        if res.code != 0 {
            Err(anyhow!("Error updating block"))
        } else {
            let new_idx = res.data["doOperations"]["id"].to_string();
            Ok(new_idx)
        }
    }

    async fn delete_block(&self, idx: &str) -> Result<()> {
        let _permit = self._sem.acquire().await?;
        let client = reqwest::Client::new();
        let url = format!("{}/api/deleteBlock", self.base_url);
        let payload = json!({"id": idx});
        let response = client
            .post(&url)
            .json(&payload)
            .headers(self.headers.clone())
            .send()
            .await?;
        let res: ResponseData<()> = response.json().await?;
        if res.code != 0 {
            Err(anyhow!("Error deleting block"))
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_list_notebooks() -> Result<()> {
        let api = Api::default().await;
        let res = api.list_notebooks().await?;
        println!("{:#?}", res);
        Ok(())
    }
}
