use anyhow::{Context, Result};
use clap::Subcommand;
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;

use crate::output::{print_output, OutputFormat};

// Bundled skills shipped with the binary.
const BUNDLED: &[(&str, &str)] = &[
    ("check-status", include_str!("../../skills/check-status/SKILL.md")),
    ("check-targeting", include_str!("../../skills/check-targeting/SKILL.md")),
    ("commit-edit", include_str!("../../skills/commit-edit/SKILL.md")),
    ("create-edit", include_str!("../../skills/create-edit/SKILL.md")),
    ("delete-edit", include_str!("../../skills/delete-edit/SKILL.md")),
    ("full-release", include_str!("../../skills/full-release/SKILL.md")),
    ("get-report", include_str!("../../skills/get-report/SKILL.md")),
    ("manage-screenshots", include_str!("../../skills/manage-screenshots/SKILL.md")),
    ("publish-app", include_str!("../../skills/publish-app/SKILL.md")),
    ("rollback-edit", include_str!("../../skills/rollback-edit/SKILL.md")),
    ("troubleshoot-validation", include_str!("../../skills/troubleshoot-validation/SKILL.md")),
    ("update-listing", include_str!("../../skills/update-listing/SKILL.md")),
    ("upload-apk", include_str!("../../skills/upload-apk/SKILL.md")),
    ("validate-edit", include_str!("../../skills/validate-edit/SKILL.md")),
];

#[derive(Subcommand)]
pub enum SkillsCommands {
    /// List available skills
    List,
    /// Show skill contents
    Show {
        name: String,
    },
    /// Find skills matching a string
    Find {
        query: String,
    },
    /// Install skills to ~/.xingu/skills (and detected agent dirs)
    Add {
        /// Specific skill name; if omitted installs all bundled skills
        #[arg(long)]
        skill: Option<String>,
    },
}

fn skills_dir() -> Result<PathBuf> {
    let dir = dirs::home_dir()
        .context("no home dir")?
        .join(".xingu")
        .join("skills");
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn agent_skill_dirs() -> Vec<(&'static str, PathBuf)> {
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return vec![],
    };
    [
        ("claude", home.join(".claude/skills")),
        ("gemini", home.join(".gemini/skills")),
        ("kiro", home.join(".kiro/skills")),
        ("cursor", home.join(".cursor/skills")),
        ("antigravity", home.join(".antigravity/skills")),
    ]
    .into_iter()
    .filter(|(_, p)| p.parent().is_some_and(|pp| pp.exists()))
    .collect()
}

fn description_of(md: &str) -> String {
    md.lines()
        .skip_while(|l| l.starts_with('#') || l.trim().is_empty())
        .find(|l| !l.trim().is_empty())
        .unwrap_or("")
        .trim()
        .trim_end_matches('.')
        .to_string()
}

pub fn run(cmd: &SkillsCommands, format: OutputFormat) -> Result<()> {
    match cmd {
        SkillsCommands::List => {
            let items: Vec<Value> = BUNDLED
                .iter()
                .map(|(n, y)| json!({ "name": n, "description": description_of(y) }))
                .collect();
            print_output(&Value::Array(items), format);
            Ok(())
        }
        SkillsCommands::Show { name } => {
            let y = BUNDLED
                .iter()
                .find(|(n, _)| *n == name)
                .map(|(_, y)| *y)
                .with_context(|| format!("skill not found: {name}"))?;
            println!("{y}");
            Ok(())
        }
        SkillsCommands::Find { query } => {
            let q = query.to_lowercase();
            let items: Vec<Value> = BUNDLED
                .iter()
                .filter(|(n, y)| n.contains(&q) || y.to_lowercase().contains(&q))
                .map(|(n, y)| json!({ "name": n, "description": description_of(y) }))
                .collect();
            print_output(&Value::Array(items), format);
            Ok(())
        }
        SkillsCommands::Add { skill } => {
            let xingu_dir = skills_dir()?;
            let agent_dirs = agent_skill_dirs();

            let skills_to_install: Vec<_> = BUNDLED
                .iter()
                .filter(|(n, _)| skill.as_deref().is_none_or(|s| s == *n))
                .collect();

            let mut installed: Vec<String> = Vec::new();
            let mut agents_installed: Vec<&str> = Vec::new();

            for (name, content) in &skills_to_install {
                let skill_dir = xingu_dir.join(name);
                fs::create_dir_all(&skill_dir)?;
                let path = skill_dir.join("SKILL.md");
                fs::write(&path, *content)?;
                installed.push(path.display().to_string());
            }

            for (agent_name, agent_dir) in &agent_dirs {
                fs::create_dir_all(agent_dir)?;
                for (name, content) in &skills_to_install {
                    let skill_dir = agent_dir.join(name);
                    fs::create_dir_all(&skill_dir)?;
                    fs::write(skill_dir.join("SKILL.md"), *content)?;
                }
                agents_installed.push(agent_name);
            }

            print_output(
                &json!({
                    "installed": installed,
                    "agents": agents_installed,
                }),
                format,
            );
            Ok(())
        }
    }
}
