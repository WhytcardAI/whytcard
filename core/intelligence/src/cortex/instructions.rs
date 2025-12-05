//! Instructions Loader - Load and manage instructions
//!
//! Two sources of instructions:
//! 1. File-based: .instructions.md files from .github/instructions/
//! 2. User-defined: Stored in database, personalized per user/session
//!
//! User instructions take priority over file instructions.

use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use glob::Pattern;
use chrono::{DateTime, Utc};

/// A single instruction (from file or user-defined)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instruction {
    /// File name (without path) or user-defined key
    pub name: String,

    /// Description from frontmatter
    pub description: String,

    /// Glob pattern for applyTo
    pub apply_to: String,

    /// Compiled glob pattern
    #[serde(skip)]
    pub pattern: Option<Pattern>,

    /// Full content (after frontmatter)
    pub content: String,

    /// Source file path (None for user-defined)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_path: Option<PathBuf>,

    /// Source type
    pub source: InstructionSource,
}

/// Source of an instruction
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InstructionSource {
    /// From .instructions.md file
    File,
    /// User-defined, stored in database
    User,
    /// System default
    System,
}

/// User instruction stored in database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInstruction {
    /// Unique ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// User or session identifier
    pub user_id: String,

    /// Instruction key (e.g., "language", "style", "workflow")
    pub key: String,

    /// Instruction value/content
    pub value: String,

    /// Category for grouping
    pub category: InstructionCategory,

    /// Priority (higher = applied first)
    #[serde(default)]
    pub priority: i32,

    /// Whether this instruction is active
    #[serde(default = "default_true")]
    pub active: bool,

    /// Created timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,

    /// Updated timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
}

fn default_true() -> bool { true }

/// Category of user instruction
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum InstructionCategory {
    /// Communication style (language, tone, format)
    Communication,
    /// Workflow preferences (ACID phases, checkpoints)
    Workflow,
    /// Domain knowledge (tech stack, frameworks)
    Domain,
    /// Coding rules (naming, patterns)
    Coding,
    /// Custom category
    Custom(String),
}

impl std::fmt::Display for InstructionCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Communication => write!(f, "communication"),
            Self::Workflow => write!(f, "workflow"),
            Self::Domain => write!(f, "domain"),
            Self::Coding => write!(f, "coding"),
            Self::Custom(s) => write!(f, "{}", s),
        }
    }
}

impl UserInstruction {
    /// Create a new user instruction
    pub fn new(user_id: impl Into<String>, key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            id: None,
            user_id: user_id.into(),
            key: key.into(),
            value: value.into(),
            category: InstructionCategory::Custom("general".to_string()),
            priority: 0,
            active: true,
            created_at: Some(Utc::now()),
            updated_at: None,
        }
    }

    /// Set category
    pub fn with_category(mut self, category: InstructionCategory) -> Self {
        self.category = category;
        self
    }

    /// Set priority
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Convert to Instruction for unified handling
    pub fn to_instruction(&self) -> Instruction {
        Instruction {
            name: self.key.clone(),
            description: format!("User instruction: {}", self.category),
            apply_to: "**".to_string(),
            pattern: Some(Pattern::new("**").unwrap()),
            content: self.value.clone(),
            source_path: None,
            source: InstructionSource::User,
        }
    }
}

impl Instruction {
    /// Check if this instruction applies to a given file path
    pub fn applies_to(&self, file_path: &str) -> bool {
        if self.apply_to == "**" {
            return true;
        }

        if let Some(ref pattern) = self.pattern {
            pattern.matches(file_path)
        } else {
            // Fallback to simple contains check
            file_path.contains(&self.apply_to.replace("**", ""))
        }
    }
}

/// Instructions manager - handles both file and user instructions
#[derive(Debug, Clone, Default)]
pub struct InstructionsManager {
    /// File-based instructions
    file_instructions: Vec<Instruction>,

    /// User-defined instructions (from DB)
    user_instructions: Vec<UserInstruction>,

    /// Combined instructions (user takes priority)
    instructions: Vec<Instruction>,

    /// Instructions directory
    instructions_dir: Option<PathBuf>,

    /// Current user/session ID for filtering
    current_user_id: Option<String>,
}

impl InstructionsManager {
    /// Create a new instructions manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the current user/session ID
    pub fn set_user(&mut self, user_id: impl Into<String>) {
        self.current_user_id = Some(user_id.into());
        self.rebuild_combined();
    }

    /// Add user instructions (typically loaded from DB)
    pub fn add_user_instructions(&mut self, instructions: Vec<UserInstruction>) {
        // Filter by current user if set
        let filtered: Vec<UserInstruction> = if let Some(ref uid) = self.current_user_id {
            instructions.into_iter()
                .filter(|i| i.active && i.user_id == *uid)
                .collect()
        } else {
            instructions.into_iter().filter(|i| i.active).collect()
        };

        self.user_instructions.extend(filtered);
        self.rebuild_combined();
    }

    /// Add a single user instruction
    pub fn add_user_instruction(&mut self, instruction: UserInstruction) {
        if instruction.active {
            self.user_instructions.push(instruction);
            self.rebuild_combined();
        }
    }

    /// Get user instructions for export/save to DB
    pub fn get_user_instructions(&self) -> &[UserInstruction] {
        &self.user_instructions
    }

    /// Rebuild combined instructions (user + file, user takes priority)
    fn rebuild_combined(&mut self) {
        self.instructions.clear();

        // Add user instructions first (higher priority)
        let mut user_sorted = self.user_instructions.clone();
        user_sorted.sort_by(|a, b| b.priority.cmp(&a.priority));

        for ui in user_sorted {
            self.instructions.push(ui.to_instruction());
        }

        // Add file instructions
        self.instructions.extend(self.file_instructions.clone());
    }

    /// Load instructions from a workspace directory
    pub fn load_from_workspace(&mut self, workspace: &Path) -> Result<usize> {
        // Try common locations for instructions
        let possible_dirs = [
            workspace.join(".github").join("instructions"),
            workspace.join(".instructions"),
            workspace.join("instructions"),
        ];

        for dir in possible_dirs {
            if dir.exists() && dir.is_dir() {
                return self.load_from_directory(&dir);
            }
        }

        tracing::debug!("No instructions directory found in workspace");
        Ok(0)
    }

    /// Load instructions from a specific directory
    pub fn load_from_directory(&mut self, dir: &Path) -> Result<usize> {
        if !dir.exists() {
            return Ok(0);
        }

        self.instructions_dir = Some(dir.to_path_buf());
        self.file_instructions.clear();

        let mut count = 0;

        // Find all .instructions.md files
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.ends_with(".instructions.md") {
                        if let Ok(instruction) = self.parse_instruction_file(&path) {
                            tracing::debug!("Loaded instruction: {} -> {}", instruction.name, instruction.apply_to);
                            self.file_instructions.push(instruction);
                            count += 1;
                        }
                    }
                }
            }
        }

        // Rebuild combined list
        self.rebuild_combined();

        tracing::info!("Loaded {} instruction files from {:?}", count, dir);
        Ok(count)
    }

    /// Parse a single instruction file
    fn parse_instruction_file(&self, path: &Path) -> Result<Instruction> {
        let content = std::fs::read_to_string(path)?;

        // Extract name: remove .instructions.md suffix
        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        let name = if filename.ends_with(".instructions.md") {
            filename.trim_end_matches(".instructions.md").to_string()
        } else {
            filename.to_string()
        };

        // Parse frontmatter
        let (frontmatter, body) = self.parse_frontmatter(&content);

        let description = frontmatter
            .get("description")
            .cloned()
            .unwrap_or_default();

        let apply_to = frontmatter
            .get("applyTo")
            .cloned()
            .unwrap_or_else(|| "**".to_string());

        // Compile glob pattern
        let pattern = Pattern::new(&apply_to).ok();

        Ok(Instruction {
            name,
            description,
            apply_to,
            pattern,
            content: body,
            source_path: Some(path.to_path_buf()),
            source: InstructionSource::File,
        })
    }

    /// Parse YAML frontmatter from content
    fn parse_frontmatter(&self, content: &str) -> (HashMap<String, String>, String) {
        let mut frontmatter = HashMap::new();

        // Check for YAML frontmatter (starts with ---)
        if !content.starts_with("---") {
            return (frontmatter, content.to_string());
        }

        // Find end of frontmatter
        if let Some(end_idx) = content[3..].find("---") {
            let yaml_content = &content[3..end_idx + 3];
            let body = &content[end_idx + 6..];

            // Simple YAML parsing (key: value)
            for line in yaml_content.lines() {
                let line = line.trim();
                if let Some(colon_idx) = line.find(':') {
                    let key = line[..colon_idx].trim().to_string();
                    let value = line[colon_idx + 1..].trim();
                    // Remove quotes
                    let value = value.trim_matches('"').trim_matches('\'').to_string();
                    frontmatter.insert(key, value);
                }
            }

            (frontmatter, body.trim().to_string())
        } else {
            (frontmatter, content.to_string())
        }
    }

    /// Get all instructions
    pub fn all(&self) -> &[Instruction] {
        &self.instructions
    }

    /// Get instructions that apply to a specific file
    pub fn for_file(&self, file_path: &str) -> Vec<&Instruction> {
        self.instructions
            .iter()
            .filter(|i| i.applies_to(file_path))
            .collect()
    }

    /// Get instructions that apply globally (applyTo: **)
    pub fn global(&self) -> Vec<&Instruction> {
        self.instructions
            .iter()
            .filter(|i| i.apply_to == "**")
            .collect()
    }

    /// Generate prompt context from instructions
    pub fn to_prompt_context(&self, file_path: Option<&str>) -> String {
        let applicable: Vec<&Instruction> = if let Some(fp) = file_path {
            self.for_file(fp)
        } else {
            self.global()
        };

        if applicable.is_empty() {
            return String::new();
        }

        let mut parts = vec![];

        // User instructions first (highest priority)
        let user_instr: Vec<_> = applicable.iter()
            .filter(|i| i.source == InstructionSource::User)
            .collect();

        if !user_instr.is_empty() {
            parts.push("## User Preferences\n".to_string());
            for instruction in user_instr {
                parts.push(format!("**{}**: {}\n", instruction.name, instruction.content));
            }
            parts.push("\n".to_string());
        }

        // File instructions
        let file_instr: Vec<_> = applicable.iter()
            .filter(|i| i.source == InstructionSource::File)
            .collect();

        if !file_instr.is_empty() {
            parts.push("## Instructions\n".to_string());
            for instruction in file_instr {
                parts.push(format!("### {} ({})\n", instruction.name, instruction.description));
                // Truncate long instructions
                let content = if instruction.content.len() > 2000 {
                    format!("{}...\n[truncated]", &instruction.content[..2000])
                } else {
                    instruction.content.clone()
                };
                parts.push(content);
                parts.push("\n".to_string());
            }
        }

        parts.join("\n")
    }

    /// Get instruction content by name
    pub fn get_content(&self, name: &str) -> Option<&str> {
        self.instructions
            .iter()
            .find(|i| i.name == name || i.name.contains(name))
            .map(|i| i.content.as_str())
    }

    /// Check if instructions are loaded
    pub fn is_loaded(&self) -> bool {
        !self.instructions.is_empty()
    }

    /// Get count of loaded instructions (total)
    pub fn count(&self) -> usize {
        self.instructions.len()
    }

    /// Get statistics
    pub fn stats(&self) -> InstructionsStats {
        InstructionsStats {
            total: self.instructions.len(),
            from_files: self.file_instructions.len(),
            from_user: self.user_instructions.len(),
            current_user: self.current_user_id.clone(),
        }
    }

    /// Reload file instructions from the configured directory (preserves user instructions)
    pub fn reload(&mut self) -> Result<usize> {
        if let Some(dir) = self.instructions_dir.clone() {
            self.load_from_directory(&dir)
        } else {
            Ok(0)
        }
    }
}

/// Statistics about loaded instructions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstructionsStats {
    /// Total combined instructions
    pub total: usize,
    /// Instructions from files
    pub from_files: usize,
    /// Instructions from user
    pub from_user: usize,
    /// Current user ID
    pub current_user: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_instruction(dir: &Path, name: &str, apply_to: &str, content: &str) {
        let path = dir.join(format!("{}.instructions.md", name));
        let full_content = format!(
            r#"---
description: "Test instruction for {}"
applyTo: "{}"
---

{}
"#,
            name, apply_to, content
        );
        std::fs::write(path, full_content).unwrap();
    }

    #[test]
    fn test_load_instructions() {
        let temp = TempDir::new().unwrap();
        let instructions_dir = temp.path().join(".github").join("instructions");
        std::fs::create_dir_all(&instructions_dir).unwrap();

        create_test_instruction(&instructions_dir, "test", "**", "Test content");
        create_test_instruction(&instructions_dir, "rust", "**/*.rs", "Rust rules");

        let mut manager = InstructionsManager::new();
        let count = manager.load_from_directory(&instructions_dir).unwrap();

        assert_eq!(count, 2);
        assert_eq!(manager.count(), 2);
    }

    #[test]
    fn test_applies_to() {
        let temp = TempDir::new().unwrap();
        let instructions_dir = temp.path();

        create_test_instruction(instructions_dir, "global", "**", "Global");
        create_test_instruction(instructions_dir, "rust", "**/*.rs", "Rust");

        let mut manager = InstructionsManager::new();
        manager.load_from_directory(instructions_dir).unwrap();

        // Global should apply to everything
        let global = manager.for_file("any/file.txt");
        assert!(global.iter().any(|i| i.name == "global"));

        // Rust should apply to .rs files
        let rust = manager.for_file("src/main.rs");
        assert!(rust.iter().any(|i| i.name == "rust"));
    }

    #[test]
    fn test_to_prompt_context() {
        let temp = TempDir::new().unwrap();
        let instructions_dir = temp.path();

        create_test_instruction(instructions_dir, "workflow", "**", "Follow ACID workflow");

        let mut manager = InstructionsManager::new();
        manager.load_from_directory(instructions_dir).unwrap();

        let context = manager.to_prompt_context(None);
        assert!(context.contains("Instructions"));
        assert!(context.contains("workflow"));
        assert!(context.contains("Follow ACID workflow"));
    }

    #[test]
    fn test_parse_frontmatter() {
        let manager = InstructionsManager::new();

        let content = r#"---
description: "Test"
applyTo: "**/*.rs"
---

# Content here
"#;

        let (fm, body) = manager.parse_frontmatter(content);
        assert_eq!(fm.get("description"), Some(&"Test".to_string()));
        assert_eq!(fm.get("applyTo"), Some(&"**/*.rs".to_string()));
        assert!(body.contains("Content here"));
    }
}
