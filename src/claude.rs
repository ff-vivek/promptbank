use std::fs;
use std::path::PathBuf;

use crate::error::{PromptBankError, Result};
use crate::prompt::Prompt;

const CLAUDE_DIR: &str = ".claude";

pub struct ClaudeIntegration {
    claude_dir: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InstallType {
    Skill,
    Command,
}

impl ClaudeIntegration {
    pub fn new() -> Result<Self> {
        let home = std::env::var("HOME")
            .map_err(|_| PromptBankError::Storage("Could not find HOME directory".to_string()))?;
        let claude_dir = PathBuf::from(home).join(CLAUDE_DIR);

        if !claude_dir.exists() {
            return Err(PromptBankError::Storage(
                "Claude directory not found. Is Claude Code installed?".to_string(),
            ));
        }

        Ok(Self { claude_dir })
    }

    /// Install a prompt as a Claude skill or command
    pub fn install(&self, prompt: &Prompt, install_type: InstallType) -> Result<PathBuf> {
        match install_type {
            InstallType::Skill => self.install_as_skill(prompt),
            InstallType::Command => self.install_as_command(prompt),
        }
    }

    /// Install as a skill (creates ~/.claude/skills/<name>/SKILL.md)
    fn install_as_skill(&self, prompt: &Prompt) -> Result<PathBuf> {
        let skill_dir = self.claude_dir.join("skills").join(&prompt.name);
        fs::create_dir_all(&skill_dir)?;

        let skill_file = skill_dir.join("SKILL.md");
        let content = self.generate_skill_content(prompt);
        fs::write(&skill_file, content)?;

        Ok(skill_file)
    }

    /// Install as a command (creates ~/.claude/commands/<name>.md)
    fn install_as_command(&self, prompt: &Prompt) -> Result<PathBuf> {
        let commands_dir = self.claude_dir.join("commands");
        fs::create_dir_all(&commands_dir)?;

        let command_file = commands_dir.join(format!("{}.md", prompt.name));
        fs::write(&command_file, &prompt.content)?;

        Ok(command_file)
    }

    /// Generate SKILL.md content with frontmatter
    fn generate_skill_content(&self, prompt: &Prompt) -> String {
        let allowed_tools = "Read, Write, Edit, Bash, Glob, Grep, Task";
        let arg_hint = if prompt.variables.is_empty() {
            String::new()
        } else {
            format!("<{}>", prompt.variables.join("> <"))
        };

        let mut content = String::new();
        content.push_str("---\n");
        content.push_str(&format!("name: {}\n", prompt.name));
        content.push_str(&format!("description: {}\n", prompt.description));
        if !arg_hint.is_empty() {
            content.push_str(&format!("argument-hint: \"{}\"\n", arg_hint));
        }
        content.push_str(&format!("allowed-tools: {}\n", allowed_tools));
        content.push_str("---\n\n");
        content.push_str(&prompt.content);

        content
    }

    /// List installed skills and commands from promptbank
    pub fn list_installed(&self) -> Result<(Vec<String>, Vec<String>)> {
        let mut skills = Vec::new();
        let mut commands = Vec::new();

        // List skills
        let skills_dir = self.claude_dir.join("skills");
        if skills_dir.exists() {
            for entry in fs::read_dir(&skills_dir)? {
                let entry = entry?;
                if entry.file_type()?.is_dir() {
                    if let Some(name) = entry.file_name().to_str() {
                        skills.push(name.to_string());
                    }
                }
            }
        }

        // List commands
        let commands_dir = self.claude_dir.join("commands");
        if commands_dir.exists() {
            for entry in fs::read_dir(&commands_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().map_or(false, |e| e == "md") {
                    if let Some(name) = path.file_stem().and_then(|n| n.to_str()) {
                        commands.push(name.to_string());
                    }
                }
            }
        }

        skills.sort();
        commands.sort();

        Ok((skills, commands))
    }

    /// Remove an installed skill or command
    pub fn remove(&self, name: &str) -> Result<bool> {
        let mut removed = false;

        // Try to remove skill
        let skill_dir = self.claude_dir.join("skills").join(name);
        if skill_dir.exists() {
            fs::remove_dir_all(&skill_dir)?;
            removed = true;
        }

        // Try to remove command
        let command_file = self.claude_dir.join("commands").join(format!("{}.md", name));
        if command_file.exists() {
            fs::remove_file(&command_file)?;
            removed = true;
        }

        Ok(removed)
    }

    /// Get the Claude directory path
    pub fn claude_dir(&self) -> &PathBuf {
        &self.claude_dir
    }
}
