use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::fs;

/// OpenClaw skill metadata from YAML frontmatter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenClawMetadata {
    pub name: Option<String>,
    pub description: Option<String>,
    pub category: Option<String>,
    pub tags: Option<Vec<String>>,
    pub author: Option<String>,
    pub version: Option<String>,
}

/// Parse YAML frontmatter from OpenClaw skill.md
/// Format:
/// ---
/// name: "Skill Name"
/// description: "Description"
/// ---
/// # Instructions...
pub fn parse_frontmatter(content: &str) -> Result<(OpenClawMetadata, String), String> {
    // Check if content starts with frontmatter delimiter
    if !content.trim_start().starts_with("---") {
        return Ok((
            OpenClawMetadata {
                name: None,
                description: None,
                category: None,
                tags: None,
                author: None,
                version: None,
            },
            content.to_string(),
        ));
    }

    let parts: Vec<&str> = content.splitn(3, "---").collect();
    if parts.len() < 3 {
        return Err("Invalid frontmatter format".to_string());
    }

    let yaml_str = parts[1].trim();
    let body = parts[2].trim();

    // Parse YAML manually (basic key-value parser)
    let metadata = parse_yaml_metadata(yaml_str)?;

    Ok((metadata, body.to_string()))
}

/// Basic YAML parser for frontmatter (supports string and array values)
fn parse_yaml_metadata(yaml: &str) -> Result<OpenClawMetadata, String> {
    let mut map: HashMap<String, String> = HashMap::new();
    let mut tags: Vec<String> = Vec::new();

    let mut in_array = false;
    let mut current_key = String::new();

    for line in yaml.lines() {
        let trimmed = line.trim();

        // Skip comments and empty lines
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Handle array items
        if trimmed.starts_with('-') && in_array {
            let value = trimmed[1..].trim().trim_matches('"').trim_matches('\'');
            tags.push(value.to_string());
            continue;
        }

        // Handle key-value pairs
        if let Some(colon_pos) = trimmed.find(':') {
            let key = trimmed[..colon_pos].trim();
            let value = trimmed[colon_pos + 1..].trim();

            // Check if this is an array start
            if value.is_empty() || value == "[]" {
                in_array = key == "tags";
                current_key = key.to_string();
                if value == "[]" {
                    tags.clear();
                }
            } else {
                in_array = false;
                // Remove quotes if present
                let clean_value = value.trim_matches('"').trim_matches('\'');
                map.insert(key.to_string(), clean_value.to_string());
            }
        }
    }

    Ok(OpenClawMetadata {
        name: map.get("name").cloned(),
        description: map.get("description").cloned(),
        category: map.get("category").cloned(),
        tags: if tags.is_empty() { None } else { Some(tags) },
        author: map.get("author").cloned(),
        version: map.get("version").cloned(),
    })
}

/// Convert OpenClaw skill.md to SkillHub format (SKILL.md)
/// Preserves YAML frontmatter but ensures SKILL.md naming convention
pub fn to_skillhub_format(
    openclaw_content: &str,
    metadata: &OpenClawMetadata,
) -> Result<String, String> {
    // For now, just ensure frontmatter exists and preserve content
    // In the future, could add more transformations
    if openclaw_content.trim_start().starts_with("---") {
        Ok(openclaw_content.to_string())
    } else {
        // Add minimal frontmatter if missing
        let frontmatter = format!(
            "---\nname: \"{}\"\ndescription: \"{}\"\n---\n\n{}",
            metadata.name.as_deref().unwrap_or("Untitled Skill"),
            metadata.description.as_deref().unwrap_or(""),
            openclaw_content
        );
        Ok(frontmatter)
    }
}

/// Convert SkillHub format to OpenClaw skill.md
/// Ensures proper YAML frontmatter formatting
pub fn to_openclaw_format(skillhub_content: &str) -> Result<String, String> {
    // OpenClaw uses skill.md (lowercase), content format is similar
    // Just ensure proper frontmatter structure
    Ok(skillhub_content.to_string())
}

/// Read OpenClaw skill from workspace directory
pub async fn read_openclaw_skill(skill_path: &str) -> Result<(OpenClawMetadata, String), String> {
    let skill_md_path = format!("{}/skill.md", skill_path);

    let content = fs::read_to_string(&skill_md_path)
        .await
        .map_err(|e| format!("Failed to read skill.md: {}", e))?;

    parse_frontmatter(&content)
}

/// Write OpenClaw skill to workspace directory
pub async fn write_openclaw_skill(
    skill_path: &str,
    content: &str,
    ensure_lowercase: bool,
) -> Result<(), String> {
    let skill_md_name = if ensure_lowercase { "skill.md" } else { "SKILL.md" };
    let skill_md_path = format!("{}/{}", skill_path, skill_md_name);

    // Create directory if needed
    fs::create_dir_all(skill_path)
        .await
        .map_err(|e| format!("Failed to create skill directory: {}", e))?;

    fs::write(&skill_md_path, content)
        .await
        .map_err(|e| format!("Failed to write skill.md: {}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_frontmatter() {
        let content = r#"---
name: "Test Skill"
description: "A test skill"
category: "testing"
tags:
  - test
  - example
---

# Instructions
Do something cool"#;

        let (metadata, body) = parse_frontmatter(content).unwrap();
        assert_eq!(metadata.name, Some("Test Skill".to_string()));
        assert_eq!(metadata.description, Some("A test skill".to_string()));
        assert_eq!(metadata.category, Some("testing".to_string()));
        assert_eq!(metadata.tags, Some(vec!["test".to_string(), "example".to_string()]));
        assert!(body.contains("# Instructions"));
    }

    #[test]
    fn test_parse_no_frontmatter() {
        let content = "# Just a markdown file";
        let (metadata, body) = parse_frontmatter(content).unwrap();
        assert_eq!(metadata.name, None);
        assert_eq!(body, content);
    }

    #[test]
    fn test_to_skillhub_format() {
        let content = r#"---
name: "Test"
---
Body"#;
        let (metadata, _) = parse_frontmatter(content).unwrap();
        let result = to_skillhub_format(content, &metadata).unwrap();
        assert!(result.contains("---"));
    }
}
