use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use uuid::Uuid;

use crate::error::{PromptBankError, Result};

/// Categories of prompts supported by the system
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PromptCategory {
    System,
    Skill,
    Agent,
    Role,
    Task,
    Template,
    Custom(String),
}

impl fmt::Display for PromptCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PromptCategory::System => write!(f, "system"),
            PromptCategory::Skill => write!(f, "skill"),
            PromptCategory::Agent => write!(f, "agent"),
            PromptCategory::Role => write!(f, "role"),
            PromptCategory::Task => write!(f, "task"),
            PromptCategory::Template => write!(f, "template"),
            PromptCategory::Custom(name) => write!(f, "custom:{}", name),
        }
    }
}

impl FromStr for PromptCategory {
    type Err = PromptBankError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "system" => Ok(PromptCategory::System),
            "skill" => Ok(PromptCategory::Skill),
            "agent" => Ok(PromptCategory::Agent),
            "role" => Ok(PromptCategory::Role),
            "task" => Ok(PromptCategory::Task),
            "template" => Ok(PromptCategory::Template),
            other if other.starts_with("custom:") => {
                Ok(PromptCategory::Custom(other[7..].to_string()))
            }
            other => Err(PromptBankError::InvalidCategory(other.to_string())),
        }
    }
}

/// A single prompt entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prompt {
    pub id: String,
    pub name: String,
    pub category: PromptCategory,
    pub description: String,
    pub content: String,
    pub tags: Vec<String>,
    pub variables: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Prompt {
    pub fn new(
        name: String,
        category: PromptCategory,
        description: String,
        content: String,
        tags: Vec<String>,
    ) -> Self {
        let now = Utc::now();
        let variables = Self::extract_variables(&content);

        Self {
            id: Uuid::new_v4().to_string()[..8].to_string(),
            name,
            category,
            description,
            content,
            tags,
            variables,
            created_at: now,
            updated_at: now,
        }
    }

    /// Extract variables from content (format: {{variable_name}})
    fn extract_variables(content: &str) -> Vec<String> {
        let mut variables = Vec::new();
        let mut i = 0;
        let chars: Vec<char> = content.chars().collect();

        while i < chars.len() {
            if i + 1 < chars.len() && chars[i] == '{' && chars[i + 1] == '{' {
                i += 2;
                let mut var_name = String::new();
                while i < chars.len() {
                    if i + 1 < chars.len() && chars[i] == '}' && chars[i + 1] == '}' {
                        i += 2;
                        break;
                    }
                    var_name.push(chars[i]);
                    i += 1;
                }
                if !var_name.is_empty() && !variables.contains(&var_name) {
                    variables.push(var_name);
                }
            } else {
                i += 1;
            }
        }

        variables
    }

    /// Apply variable substitutions to the prompt content
    pub fn render(&self, substitutions: &[(String, String)]) -> String {
        let mut result = self.content.clone();
        for (key, value) in substitutions {
            let pattern = format!("{{{{{}}}}}", key);
            result = result.replace(&pattern, value);
        }
        result
    }

    pub fn update_content(&mut self, content: String) {
        self.content = content.clone();
        self.variables = Self::extract_variables(&content);
        self.updated_at = Utc::now();
    }
}

/// The prompt bank containing all prompts
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct PromptBank {
    pub prompts: Vec<Prompt>,
    pub version: String,
}

impl PromptBank {
    pub fn new() -> Self {
        Self {
            prompts: Vec::new(),
            version: "1.0".to_string(),
        }
    }

    pub fn add(&mut self, prompt: Prompt) {
        self.prompts.push(prompt);
    }

    pub fn get(&self, id: &str) -> Option<&Prompt> {
        self.prompts.iter().find(|p| p.id == id || p.name == id)
    }

    pub fn get_mut(&mut self, id: &str) -> Option<&mut Prompt> {
        self.prompts.iter_mut().find(|p| p.id == id || p.name == id)
    }

    pub fn delete(&mut self, id: &str) -> bool {
        let len_before = self.prompts.len();
        self.prompts.retain(|p| p.id != id && p.name != id);
        self.prompts.len() != len_before
    }

    pub fn list_by_category(&self, category: &PromptCategory) -> Vec<&Prompt> {
        self.prompts.iter().filter(|p| &p.category == category).collect()
    }

    pub fn search(&self, query: &str) -> Vec<&Prompt> {
        let query_lower = query.to_lowercase();
        self.prompts
            .iter()
            .filter(|p| {
                p.name.to_lowercase().contains(&query_lower)
                    || p.description.to_lowercase().contains(&query_lower)
                    || p.tags.iter().any(|t| t.to_lowercase().contains(&query_lower))
                    || p.content.to_lowercase().contains(&query_lower)
            })
            .collect()
    }
}
