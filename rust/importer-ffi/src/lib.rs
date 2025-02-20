uniffi::include_scaffolding!("lib");

mod error;

pub use error::MyError;
use error::MyResult;
use importer_backend::Notebook;

pub struct NotebookFfi {
    core: Notebook,
}

impl NotebookFfi {
    pub fn new(data_home: String, base_url: String) -> MyResult<Self> {
        let notebook = Notebook::new(&data_home, &base_url)?;
        Ok(Self { core: notebook })
    }

    pub fn get_notebook_names(&self) -> MyResult<Vec<String>> {
        let names = self.core.get_notebook_names()?;
        Ok(names)
    }

    pub fn set_notebook_name(&self, name: String) -> MyResult<()> {
        self.core.set_notebook_name(&name)?;
        Ok(())
    }

    pub fn get_all_files(&self) -> MyResult<Vec<String>> {
        let files = self.core.get_all_files()?;
        Ok(files)
    }

    pub fn process_file(&self, path: String) -> MyResult<()> {
        self.core.process_file(&path)?;
        Ok(())
    }
}
