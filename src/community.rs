use serde::{Deserialize, Serialize};

use crate::error::{PromptBankError, Result};
use crate::prompt::{Prompt, PromptCategory};

const COMMUNITY_REPO: &str = "ff-vivek/promptbank-community";
const RAW_BASE_URL: &str = "https://raw.githubusercontent.com/ff-vivek/promptbank-community/main";

#[derive(Debug, Deserialize, Serialize)]
pub struct CommunityIndex {
    pub version: String,
    pub prompts: Vec<CommunityPromptEntry>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CommunityPromptEntry {
    pub name: String,
    pub category: String,
    pub description: String,
    pub author: String,
    pub path: String,
    pub tags: Vec<String>,
    pub downloads: u64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CommunityPrompt {
    pub name: String,
    pub category: String,
    pub description: String,
    pub content: String,
    pub tags: Vec<String>,
    #[serde(default)]
    pub variables: Vec<String>,
    pub author: String,
    pub version: String,
}

pub struct Community;

impl Community {
    /// Fetch the community index
    pub fn fetch_index() -> Result<CommunityIndex> {
        let url = format!("{}/index.json", RAW_BASE_URL);
        let response = ureq::get(&url)
            .call()
            .map_err(|e| PromptBankError::Storage(format!("Failed to fetch index: {}", e)))?;

        let index: CommunityIndex = response
            .into_json()
            .map_err(|e| PromptBankError::Storage(format!("Failed to parse index: {}", e)))?;

        Ok(index)
    }

    /// Fetch a specific prompt from the community
    pub fn fetch_prompt(path: &str) -> Result<CommunityPrompt> {
        let url = format!("{}/{}", RAW_BASE_URL, path);
        let response = ureq::get(&url)
            .call()
            .map_err(|e| PromptBankError::Storage(format!("Failed to fetch prompt: {}", e)))?;

        let prompt: CommunityPrompt = response
            .into_json()
            .map_err(|e| PromptBankError::Storage(format!("Failed to parse prompt: {}", e)))?;

        Ok(prompt)
    }

    /// Convert a community prompt to a local prompt
    pub fn to_local_prompt(community_prompt: CommunityPrompt) -> Result<Prompt> {
        let category: PromptCategory = community_prompt.category.parse()?;
        Ok(Prompt::new(
            community_prompt.name,
            category,
            community_prompt.description,
            community_prompt.content,
            community_prompt.tags,
        ))
    }

    /// Search community prompts
    pub fn search<'a>(index: &'a CommunityIndex, query: &str) -> Vec<&'a CommunityPromptEntry> {
        let query_lower = query.to_lowercase();
        index
            .prompts
            .iter()
            .filter(|p| {
                p.name.to_lowercase().contains(&query_lower)
                    || p.description.to_lowercase().contains(&query_lower)
                    || p.tags.iter().any(|t| t.to_lowercase().contains(&query_lower))
                    || p.category.to_lowercase().contains(&query_lower)
            })
            .collect()
    }

    /// Get the GitHub repo URL for contributions
    pub fn repo_url() -> String {
        format!("https://github.com/{}", COMMUNITY_REPO)
    }
}
