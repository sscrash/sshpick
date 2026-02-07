use anyhow::{Context, Result};
use console::style;
use dialoguer::{theme::ColorfulTheme, Select};
use std::fs;
use std::process::Command;

/// An SSH host entry parsed from ~/.ssh/config
#[derive(Debug, Clone)]
struct SshHost {
    name: String,
    hostname: Option<String>,
    identity_file: Option<String>,
    user: Option<String>,
}

impl std::fmt::Display for SshHost {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut parts = vec![self.name.clone()];
        if let Some(ref host) = self.hostname {
            parts.push(format!("→ {}", host));
        }
        if let Some(ref key) = self.identity_file {
            parts.push(format!("[{}]", key));
        }
        if let Some(ref user) = self.user {
            parts.push(format!("({})", user));
        }
        write!(f, "{}", parts.join("  "))
    }
}

/// Parse ~/.ssh/config and extract Host entries
fn parse_ssh_config() -> Result<Vec<SshHost>> {
    let home = dirs::home_dir().context("Could not determine home directory")?;
    let config_path = home.join(".ssh").join("config");

    let content = fs::read_to_string(&config_path)
        .with_context(|| format!("Could not read {}", config_path.display()))?;

    let mut hosts: Vec<SshHost> = Vec::new();
    let mut current: Option<SshHost> = None;

    for line in content.lines() {
        let line = line.trim();

        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let (key, value) = match line.split_once(|c: char| c.is_whitespace() || c == '=') {
            Some((k, v)) => (k.trim().to_lowercase(), v.trim().to_string()),
            None => continue,
        };

        match key.as_str() {
            "host" => {
                if let Some(h) = current.take() {
                    hosts.push(h);
                }
                if value.contains('*') {
                    continue;
                }
                current = Some(SshHost {
                    name: value,
                    hostname: None,
                    identity_file: None,
                    user: None,
                });
            }
            "hostname" => {
                if let Some(ref mut h) = current {
                    h.hostname = Some(value);
                }
            }
            "identityfile" => {
                if let Some(ref mut h) = current {
                    let resolved = if value.starts_with("~/") {
                        let home = dirs::home_dir().unwrap();
                        home.join(&value[2..]).to_string_lossy().to_string()
                    } else {
                        value
                    };
                    h.identity_file = Some(resolved);
                }
            }
            "user" => {
                if let Some(ref mut h) = current {
                    h.user = Some(value);
                }
            }
            _ => {}
        }
    }

    if let Some(h) = current {
        hosts.push(h);
    }

    Ok(hosts)
}

fn pick_host(hosts: &[SshHost]) -> Result<&SshHost> {
    if hosts.is_empty() {
        anyhow::bail!("No Host entries found in ~/.ssh/config");
    }

    if hosts.len() == 1 {
        println!(
            "{} Only one host found, using: {}",
            style("→").cyan().bold(),
            style(&hosts[0]).green()
        );
        return Ok(&hosts[0]);
    }

    let display: Vec<String> = hosts.iter().map(|h| h.to_string()).collect();

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select SSH host")
        .items(&display)
        .default(0)
        .interact()
        .context("Selection cancelled")?;

    Ok(&hosts[selection])
}

fn main() -> Result<()> {
    let hosts = parse_ssh_config()?;
    let host = pick_host(&hosts)?;

    println!(
        "\n{} ssh -T git@{}\n",
        style("🔑").bold(),
        style(&host.name).green()
    );

    let status = Command::new("ssh")
        .args(["-T", &format!("git@{}", host.name)])
        .status()
        .context("Failed to execute ssh")?;

    // GitHub/GitLab return exit code 1 on successful auth with ssh -T
    if !status.success() && status.code() != Some(1) {
        anyhow::bail!("ssh exited with status {}", status);
    }

    Ok(())
}
