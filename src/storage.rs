use directories::ProjectDirs;
use std::fs;
use std::path::PathBuf;

use crate::error::{PromptBankError, Result};
use crate::prompt::PromptBank;

const APP_NAME: &str = "promptbank";
const ORG_NAME: &str = "claude";
const DATA_FILE: &str = "prompts.json";

pub struct Storage {
    data_path: PathBuf,
}

impl Storage {
    pub fn new() -> Result<Self> {
        let data_path = Self::get_data_path()?;

        // Ensure directory exists
        if let Some(parent) = data_path.parent() {
            fs::create_dir_all(parent)?;
        }

        Ok(Self { data_path })
    }

    /// Get the path to the data file
    fn get_data_path() -> Result<PathBuf> {
        if let Some(proj_dirs) = ProjectDirs::from("com", ORG_NAME, APP_NAME) {
            let data_dir = proj_dirs.data_dir();
            Ok(data_dir.join(DATA_FILE))
        } else {
            // Fallback to home directory
            let home = dirs_fallback()?;
            Ok(home.join(format!(".{}", APP_NAME)).join(DATA_FILE))
        }
    }

    /// Load the prompt bank from storage
    pub fn load(&self) -> Result<PromptBank> {
        if !self.data_path.exists() {
            return Ok(PromptBank::new());
        }

        let content = fs::read_to_string(&self.data_path)?;
        let bank: PromptBank = serde_json::from_str(&content)?;
        Ok(bank)
    }

    /// Save the prompt bank to storage
    pub fn save(&self, bank: &PromptBank) -> Result<()> {
        let content = serde_json::to_string_pretty(bank)?;
        fs::write(&self.data_path, content)?;
        Ok(())
    }

    /// Get the data file path for display
    pub fn data_file_path(&self) -> &PathBuf {
        &self.data_path
    }

    /// Export prompts to a file
    pub fn export(&self, bank: &PromptBank, path: &PathBuf) -> Result<()> {
        let content = serde_json::to_string_pretty(bank)?;
        fs::write(path, content)?;
        Ok(())
    }

    /// Import prompts from a file
    pub fn import(&self, path: &PathBuf) -> Result<PromptBank> {
        let content = fs::read_to_string(path)?;
        let bank: PromptBank = serde_json::from_str(&content)?;
        Ok(bank)
    }
}

fn dirs_fallback() -> Result<PathBuf> {
    std::env::var("HOME")
        .map(PathBuf::from)
        .map_err(|_| PromptBankError::Storage("Could not determine home directory".to_string()))
}
