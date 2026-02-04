use clap::{Parser, Subcommand};
use colored::*;
use dialoguer::{Editor, Input, Select};
use std::path::PathBuf;

use crate::error::{PromptBankError, Result};
use crate::prompt::{Prompt, PromptBank, PromptCategory};
use crate::storage::Storage;

#[derive(Parser)]
#[command(name = "promptbank")]
#[command(author, version, about = "Manage and apply prompts for Claude AI")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Add a new prompt
    Add {
        /// Name of the prompt
        #[arg(short, long)]
        name: Option<String>,

        /// Category (system, skill, agent, role, task, template)
        #[arg(short, long)]
        category: Option<String>,

        /// Description of the prompt
        #[arg(short, long)]
        description: Option<String>,

        /// Tags (comma-separated)
        #[arg(short, long)]
        tags: Option<String>,

        /// Content of the prompt (opens editor if not provided)
        #[arg(long)]
        content: Option<String>,

        /// Read content from a file
        #[arg(short, long)]
        file: Option<PathBuf>,
    },

    /// List all prompts
    List {
        /// Filter by category
        #[arg(short, long)]
        category: Option<String>,

        /// Show full content
        #[arg(long)]
        full: bool,
    },

    /// Get a specific prompt by ID or name
    Get {
        /// ID or name of the prompt
        id: String,

        /// Copy to clipboard
        #[arg(short, long)]
        copy: bool,

        /// Only output the content (for piping)
        #[arg(short, long)]
        raw: bool,
    },

    /// Apply a prompt (render with variables)
    Apply {
        /// ID or name of the prompt
        id: String,

        /// Variable substitutions (format: key=value)
        #[arg(short, long)]
        var: Vec<String>,

        /// Copy to clipboard
        #[arg(short, long)]
        copy: bool,

        /// Interactive mode for variables
        #[arg(short, long)]
        interactive: bool,
    },

    /// Edit an existing prompt
    Edit {
        /// ID or name of the prompt
        id: String,
    },

    /// Delete a prompt
    Delete {
        /// ID or name of the prompt
        id: String,

        /// Skip confirmation
        #[arg(short, long)]
        force: bool,
    },

    /// Search prompts
    Search {
        /// Search query
        query: String,
    },

    /// Export prompts to a file
    Export {
        /// Output file path
        output: PathBuf,
    },

    /// Import prompts from a file
    Import {
        /// Input file path
        input: PathBuf,

        /// Merge with existing prompts
        #[arg(short, long)]
        merge: bool,
    },

    /// Show storage info
    Info,
}

pub struct App {
    storage: Storage,
    bank: PromptBank,
}

impl App {
    pub fn new() -> Result<Self> {
        let storage = Storage::new()?;
        let bank = storage.load()?;
        Ok(Self { storage, bank })
    }

    pub fn run(&mut self, cli: Cli) -> Result<()> {
        match cli.command {
            Commands::Add {
                name,
                category,
                description,
                tags,
                content,
                file,
            } => self.add_prompt(name, category, description, tags, content, file),

            Commands::List { category, full } => self.list_prompts(category, full),

            Commands::Get { id, copy, raw } => self.get_prompt(&id, copy, raw),

            Commands::Apply {
                id,
                var,
                copy,
                interactive,
            } => self.apply_prompt(&id, var, copy, interactive),

            Commands::Edit { id } => self.edit_prompt(&id),

            Commands::Delete { id, force } => self.delete_prompt(&id, force),

            Commands::Search { query } => self.search_prompts(&query),

            Commands::Export { output } => self.export_prompts(&output),

            Commands::Import { input, merge } => self.import_prompts(&input, merge),

            Commands::Info => self.show_info(),
        }
    }

    fn add_prompt(
        &mut self,
        name: Option<String>,
        category: Option<String>,
        description: Option<String>,
        tags: Option<String>,
        content: Option<String>,
        file: Option<PathBuf>,
    ) -> Result<()> {
        // Get name interactively if not provided
        let name = match name {
            Some(n) => n,
            None => Input::new()
                .with_prompt("Prompt name")
                .interact_text()
                .map_err(|e| PromptBankError::InvalidInput(e.to_string()))?,
        };

        // Get category interactively if not provided
        let category = match category {
            Some(c) => c.parse()?,
            None => {
                let categories = vec!["system", "skill", "agent", "role", "task", "template"];
                let selection = Select::new()
                    .with_prompt("Select category")
                    .items(&categories)
                    .default(0)
                    .interact()
                    .map_err(|e| PromptBankError::InvalidInput(e.to_string()))?;
                categories[selection].parse()?
            }
        };

        // Get description
        let description = match description {
            Some(d) => d,
            None => Input::new()
                .with_prompt("Description")
                .interact_text()
                .map_err(|e| PromptBankError::InvalidInput(e.to_string()))?,
        };

        // Get tags
        let tags: Vec<String> = match tags {
            Some(t) => t.split(',').map(|s| s.trim().to_string()).collect(),
            None => {
                let tags_str: String = Input::new()
                    .with_prompt("Tags (comma-separated, optional)")
                    .allow_empty(true)
                    .interact_text()
                    .map_err(|e| PromptBankError::InvalidInput(e.to_string()))?;
                if tags_str.is_empty() {
                    Vec::new()
                } else {
                    tags_str.split(',').map(|s| s.trim().to_string()).collect()
                }
            }
        };

        // Get content
        let content = if let Some(path) = file {
            std::fs::read_to_string(&path)?
        } else if let Some(c) = content {
            c
        } else {
            Editor::new()
                .edit("# Enter your prompt content here\n")
                .map_err(|e| PromptBankError::InvalidInput(e.to_string()))?
                .ok_or_else(|| PromptBankError::InvalidInput("No content provided".to_string()))?
        };

        let prompt = Prompt::new(name.clone(), category, description, content, tags);
        let id = prompt.id.clone();
        self.bank.add(prompt);
        self.storage.save(&self.bank)?;

        println!("{} Prompt '{}' added with ID: {}", "✓".green(), name, id.cyan());
        Ok(())
    }

    fn list_prompts(&self, category: Option<String>, full: bool) -> Result<()> {
        let prompts: Vec<&Prompt> = if let Some(cat) = category {
            let cat: PromptCategory = cat.parse()?;
            self.bank.list_by_category(&cat)
        } else {
            self.bank.prompts.iter().collect()
        };

        if prompts.is_empty() {
            println!("{}", "No prompts found.".yellow());
            return Ok(());
        }

        println!(
            "\n{} {} prompt(s) found:\n",
            "→".blue(),
            prompts.len().to_string().cyan()
        );

        for prompt in prompts {
            self.print_prompt_summary(prompt, full);
        }

        Ok(())
    }

    fn get_prompt(&self, id: &str, copy: bool, raw: bool) -> Result<()> {
        let prompt = self
            .bank
            .get(id)
            .ok_or_else(|| PromptBankError::PromptNotFound(id.to_string()))?;

        if raw {
            println!("{}", prompt.content);
        } else {
            self.print_prompt_full(prompt);
        }

        if copy {
            self.copy_to_clipboard(&prompt.content)?;
            if !raw {
                println!("\n{} Copied to clipboard!", "✓".green());
            }
        }

        Ok(())
    }

    fn apply_prompt(
        &mut self,
        id: &str,
        vars: Vec<String>,
        copy: bool,
        interactive: bool,
    ) -> Result<()> {
        let prompt = self
            .bank
            .get(id)
            .ok_or_else(|| PromptBankError::PromptNotFound(id.to_string()))?;

        let mut substitutions: Vec<(String, String)> = Vec::new();

        // Parse provided variables
        for var in vars {
            let parts: Vec<&str> = var.splitn(2, '=').collect();
            if parts.len() == 2 {
                substitutions.push((parts[0].to_string(), parts[1].to_string()));
            }
        }

        // Interactive mode for remaining variables
        if interactive && !prompt.variables.is_empty() {
            println!(
                "\n{} This prompt has {} variable(s):\n",
                "→".blue(),
                prompt.variables.len()
            );

            for var in &prompt.variables {
                let existing = substitutions.iter().find(|(k, _)| k == var);
                if existing.is_none() {
                    let value: String = Input::new()
                        .with_prompt(format!("  {}", var))
                        .interact_text()
                        .map_err(|e| PromptBankError::InvalidInput(e.to_string()))?;
                    substitutions.push((var.clone(), value));
                }
            }
        }

        let rendered = prompt.render(&substitutions);

        println!("\n{}", "═".repeat(60).dimmed());
        println!("{}", rendered);
        println!("{}", "═".repeat(60).dimmed());

        if copy {
            self.copy_to_clipboard(&rendered)?;
            println!("\n{} Copied to clipboard!", "✓".green());
        }

        Ok(())
    }

    fn edit_prompt(&mut self, id: &str) -> Result<()> {
        let prompt = self
            .bank
            .get(id)
            .ok_or_else(|| PromptBankError::PromptNotFound(id.to_string()))?;

        let current_content = prompt.content.clone();

        let new_content = Editor::new()
            .edit(&current_content)
            .map_err(|e| PromptBankError::InvalidInput(e.to_string()))?
            .ok_or_else(|| PromptBankError::InvalidInput("No content provided".to_string()))?;

        if new_content == current_content {
            println!("{}", "No changes made.".yellow());
            return Ok(());
        }

        let prompt = self
            .bank
            .get_mut(id)
            .ok_or_else(|| PromptBankError::PromptNotFound(id.to_string()))?;
        prompt.update_content(new_content);
        self.storage.save(&self.bank)?;

        println!("{} Prompt '{}' updated.", "✓".green(), id);
        Ok(())
    }

    fn delete_prompt(&mut self, id: &str, force: bool) -> Result<()> {
        let prompt = self
            .bank
            .get(id)
            .ok_or_else(|| PromptBankError::PromptNotFound(id.to_string()))?;

        let name = prompt.name.clone();

        if !force {
            let confirm = Select::new()
                .with_prompt(format!("Delete prompt '{}'?", name))
                .items(&["No", "Yes"])
                .default(0)
                .interact()
                .map_err(|e| PromptBankError::InvalidInput(e.to_string()))?;

            if confirm == 0 {
                println!("{}", "Cancelled.".yellow());
                return Ok(());
            }
        }

        self.bank.delete(id);
        self.storage.save(&self.bank)?;

        println!("{} Prompt '{}' deleted.", "✓".green(), name);
        Ok(())
    }

    fn search_prompts(&self, query: &str) -> Result<()> {
        let prompts = self.bank.search(query);

        if prompts.is_empty() {
            println!("{} No prompts matching '{}'", "→".yellow(), query);
            return Ok(());
        }

        println!(
            "\n{} {} result(s) for '{}':\n",
            "→".blue(),
            prompts.len().to_string().cyan(),
            query
        );

        for prompt in prompts {
            self.print_prompt_summary(prompt, false);
        }

        Ok(())
    }

    fn export_prompts(&self, output: &PathBuf) -> Result<()> {
        self.storage.export(&self.bank, output)?;
        println!(
            "{} Exported {} prompts to {:?}",
            "✓".green(),
            self.bank.prompts.len(),
            output
        );
        Ok(())
    }

    fn import_prompts(&mut self, input: &PathBuf, merge: bool) -> Result<()> {
        let imported = self.storage.import(input)?;
        let count = imported.prompts.len();

        if merge {
            for prompt in imported.prompts {
                if self.bank.get(&prompt.id).is_none() {
                    self.bank.add(prompt);
                }
            }
        } else {
            self.bank = imported;
        }

        self.storage.save(&self.bank)?;
        println!(
            "{} Imported {} prompts from {:?}",
            "✓".green(),
            count,
            input
        );
        Ok(())
    }

    fn show_info(&self) -> Result<()> {
        println!("\n{}", "Promptbank Info".bold().underline());
        println!("  Data file: {:?}", self.storage.data_file_path());
        println!("  Total prompts: {}", self.bank.prompts.len());

        // Count by category
        let mut counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        for prompt in &self.bank.prompts {
            *counts.entry(prompt.category.to_string()).or_insert(0) += 1;
        }

        if !counts.is_empty() {
            println!("\n  {}:", "By category".dimmed());
            for (cat, count) in counts {
                println!("    {}: {}", cat, count);
            }
        }

        println!();
        Ok(())
    }

    fn print_prompt_summary(&self, prompt: &Prompt, full: bool) {
        println!(
            "  {} {} [{}]",
            prompt.id.cyan(),
            prompt.name.bold(),
            prompt.category.to_string().yellow()
        );
        println!("    {}", prompt.description.dimmed());

        if !prompt.tags.is_empty() {
            println!("    Tags: {}", prompt.tags.join(", ").blue());
        }

        if !prompt.variables.is_empty() {
            println!(
                "    Variables: {}",
                prompt.variables.join(", ").magenta()
            );
        }

        if full {
            println!("\n{}", "─".repeat(50).dimmed());
            println!("{}", prompt.content);
            println!("{}\n", "─".repeat(50).dimmed());
        } else {
            println!();
        }
    }

    fn print_prompt_full(&self, prompt: &Prompt) {
        println!("\n{}", "═".repeat(60).dimmed());
        println!(
            "{}: {} ({})",
            "ID".bold(),
            prompt.id.cyan(),
            prompt.category.to_string().yellow()
        );
        println!("{}: {}", "Name".bold(), prompt.name);
        println!("{}: {}", "Description".bold(), prompt.description);

        if !prompt.tags.is_empty() {
            println!("{}: {}", "Tags".bold(), prompt.tags.join(", ").blue());
        }

        if !prompt.variables.is_empty() {
            println!(
                "{}: {}",
                "Variables".bold(),
                prompt.variables.join(", ").magenta()
            );
        }

        println!("{}: {}", "Created".bold(), prompt.created_at.format("%Y-%m-%d %H:%M"));
        println!("{}: {}", "Updated".bold(), prompt.updated_at.format("%Y-%m-%d %H:%M"));

        println!("\n{}", "Content:".bold().underline());
        println!("{}", "─".repeat(60).dimmed());
        println!("{}", prompt.content);
        println!("{}", "═".repeat(60).dimmed());
    }

    fn copy_to_clipboard(&self, text: &str) -> Result<()> {
        use arboard::Clipboard;

        let mut clipboard =
            Clipboard::new().map_err(|e| PromptBankError::Clipboard(e.to_string()))?;
        clipboard
            .set_text(text)
            .map_err(|e| PromptBankError::Clipboard(e.to_string()))?;
        Ok(())
    }
}
