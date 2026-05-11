use anyhow::Result;
use serde_json::json;
use std::fs;
use std::path::PathBuf;

use crate::output::{print_output, OutputFormat};

const AGENTS_MD: &str = r#"# xingu

This project uses [xingu](https://github.com/giolaq/xingu) to manage Amazon
Appstore submissions. Agents should prefer `xingu` over shelling out to curl
against developer.amazon.com.

## Quick reference

- `xingu +status <app-id>` — app info + active edit
- `xingu +publish <app-id> --file app.apk` — edit → upload → commit
- `xingu +update-listing <app-id> --locale en-US --title ...`
- `xingu --dry-run <any-command>` — preview without executing

Output is JSON by default. Exit codes: 0 ok, 1 api, 2 auth, 3 validation, 4 network.

Run `xingu skills list` to see installable agent workflows.
"#;

/// Known agent skill directories, relative to $HOME.
fn agent_dirs() -> Vec<(&'static str, PathBuf)> {
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return vec![],
    };
    [
        ("gemini", home.join(".gemini/skills")),
        ("claude", home.join(".claude/skills")),
        ("kiro", home.join(".kiro/skills")),
        ("cursor", home.join(".cursor/skills")),
        ("antigravity", home.join(".antigravity/skills")),
    ]
    .into_iter()
    .filter(|(_, p)| p.parent().is_some_and(|pp| pp.exists()))
    .collect()
}

pub fn run(format: OutputFormat) -> Result<()> {
    let agents_md = PathBuf::from("AGENTS.md");
    let mut wrote_agents_md = false;
    if !agents_md.exists() {
        fs::write(&agents_md, AGENTS_MD)?;
        wrote_agents_md = true;
    }

    let detected: Vec<&str> = agent_dirs().iter().map(|(n, _)| *n).collect();

    print_output(
        &json!({
            "agents_md": if wrote_agents_md { "created" } else { "existed" },
            "detected_agents": detected,
            "next": "Run `xingu skills add` to install bundled skills.",
        }),
        format,
    );
    Ok(())
}
