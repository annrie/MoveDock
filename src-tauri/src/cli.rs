use crate::{model::*, process};
use std::{
    ffi::OsString,
    path::{Path, PathBuf},
    time::Duration,
};
use tokio::{process::Command, sync::mpsc};

pub fn expand(value: &str) -> PathBuf {
    if let Some(rest) = value.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(rest);
        }
    }
    PathBuf::from(value)
}
// Prefer installations with their own rbenv manager over abandoned shims.
fn rbenv_paths(home: &Path) -> Vec<PathBuf> {
    let roots = [home.join(".anyenv/envs/rbenv"), home.join(".rbenv")];
    let mut paths = Vec::new();
    for root in &roots {
        if root.join("bin/rbenv").is_file() || root.join("libexec/rbenv").is_file() {
            paths.push(root.join("shims"));
        }
    }
    // Homebrew keeps the manager outside ~/.rbenv.
    if ["/opt/homebrew/bin/rbenv", "/usr/local/bin/rbenv"]
        .iter()
        .any(|manager| Path::new(manager).is_file())
        && !paths.contains(&home.join(".rbenv/shims"))
    {
        paths.push(home.join(".rbenv/shims"));
    }
    paths
}
pub fn path_env(settings: &Settings) -> OsString {
    let mut paths: Vec<PathBuf> = settings
        .extra_path
        .lines()
        .filter(|p| !p.trim().is_empty())
        .map(|p| expand(p.trim()))
        .collect();
    if let Some(home) = dirs::home_dir() {
        paths.extend(rbenv_paths(&home));
        paths.push(home.join(".local/bin"));
    }
    paths.extend(std::env::split_paths(
        &std::env::var_os("PATH").unwrap_or_default(),
    ));
    paths.extend(
        [
            "/opt/homebrew/bin",
            "/usr/local/bin",
            "/usr/bin",
            "/bin",
            "/usr/sbin",
            "/sbin",
        ]
        .map(PathBuf::from),
    );
    std::env::join_paths(paths).unwrap_or_default()
}
pub fn resolve(name: &str, settings: &Settings) -> Result<PathBuf, String> {
    which::which_in(expand(name), Some(path_env(settings)), "/").map_err(|_| {
        crate::messages::with_params(
            "backend.executableMissing",
            serde_json::json!({"name": name}),
        )
    })
}
pub fn base(settings: &Settings, cwd: &Path) -> Result<Command, String> {
    if !["direct", "bundler"].contains(&settings.mode.as_str()) {
        return Err(crate::messages::message("backend.invalidMode"));
    }
    let default = if settings.mode == "bundler" {
        "bundle"
    } else {
        "wordmove"
    };
    let executable = if settings.executable.trim().is_empty() {
        default
    } else {
        settings.executable.trim()
    };
    let mut command = Command::new(resolve(executable, settings)?);
    command
        .current_dir(cwd)
        .env("PATH", path_env(settings))
        .env("NO_COLOR", "1")
        .env("TERM", "dumb");
    // A GUI launched from `bundle exec` must not inherit the caller's bundle.
    for key in [
        "RUBYOPT",
        "RUBYLIB",
        "BUNDLE_BIN_PATH",
        "BUNDLE_GEMFILE",
        "BUNDLE_PATH",
        "BUNDLE_APP_CONFIG",
        "RBENV_DIR",
        "RBENV_GEMSET_ALREADY",
        "RBENV_GEMSET_FILE",
    ] {
        command.env_remove(key);
    }
    if !settings.ruby_version.trim().is_empty() {
        command.env("RBENV_VERSION", settings.ruby_version.trim());
    }
    if settings.mode == "bundler" {
        let gemfile = expand(settings.gemfile.trim())
            .canonicalize()
            .map_err(|_| crate::messages::message("backend.gemfileRequired"))?;
        if !gemfile.is_file() {
            return Err(crate::messages::message("backend.gemfileNotFile"));
        }
        // Reuse bundle config from the selected fork, not the WordPress site's config.
        command.env(
            "BUNDLE_APP_CONFIG",
            gemfile.parent().unwrap().join(".bundle"),
        );
        command
            .env("RBENV_DIR", gemfile.parent().unwrap())
            .env("BUNDLE_GEMFILE", &gemfile)
            .args(["exec", "wordmove"]);
    }
    Ok(command)
}
pub fn config_name(path: &str) -> Result<String, String> {
    let name = Path::new(path)
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or(crate::messages::message("backend.invalidFilename"))?;
    if name.contains(['*', '?', '[', ']', '{', '}', '\\', '\n', '\r']) {
        return Err(crate::messages::message("backend.unsafeFilename"));
    }
    // Wordmove joins --config onto its working directory. Use a relative filename.
    Ok(format!("./{name}"))
}
pub fn arguments(request: &RunRequest, site: &Site) -> Result<Vec<String>, String> {
    if !["push", "pull"].contains(&request.direction.as_str()) {
        return Err(crate::messages::message("backend.invalidDirection"));
    }
    if request.environment.is_empty()
        || request.environment.starts_with('-')
        || request.environment.contains(['\n', '\r', '\0'])
    {
        return Err(crate::messages::message("backend.invalidEnvironment"));
    }
    let allowed = [
        "wordpress",
        "uploads",
        "themes",
        "plugins",
        "mu_plugins",
        "languages",
        "db",
    ];
    if request.targets.is_empty()
        || request
            .targets
            .iter()
            .any(|t| !allowed.contains(&t.as_str()))
    {
        return Err(crate::messages::message("backend.targetsRequired"));
    }
    let mut args = vec![
        request.direction.clone(),
        "--config".into(),
        config_name(&site.path)?,
        "--environment".into(),
        request.environment.clone(),
    ];
    for target in allowed {
        if request.targets.iter().any(|t| t == target) {
            args.push(format!("--{target}"));
        }
    }
    if request.simulate {
        args.push("--simulate".into());
    }
    Ok(args)
}
pub fn display(command: &Command) -> String {
    let quote = |s: &std::ffi::OsStr| format!("'{}'", s.to_string_lossy().replace('\'', "'\\''"));
    std::iter::once(command.as_std().get_program())
        .chain(command.as_std().get_args())
        .map(quote)
        .collect::<Vec<_>>()
        .join(" ")
}
pub fn parse_environments(text: &str) -> Result<Inspection, String> {
    let mut section = "";
    let mut local = String::new();
    let mut environments = Vec::new();
    for line in text.lines() {
        if line.contains("Listing Local") {
            section = "local";
            continue;
        }
        if line.contains("Listing Remotes") {
            section = "remote";
            continue;
        }
        if let Some((name, vhost)) = line.trim().split_once(": ") {
            if name.is_empty() || vhost.trim().is_empty() {
                continue;
            }
            match section {
                "local" if name == "local" => local = vhost.into(),
                "remote" => environments.push(Environment {
                    name: name.into(),
                    vhost: vhost.into(),
                }),
                _ => {}
            }
        }
    }
    if environments.is_empty() {
        return Err(crate::messages::message("backend.noEnvironments"));
    }
    Ok(Inspection {
        local,
        environments,
    })
}
pub async fn capture(command: Command, limit: Duration) -> Result<String, String> {
    let (_tx, rx) = mpsc::channel(1);
    let mut text = String::new();
    let result = process::execute(command, rx, limit, |event| {
        if text.len() < 128 * 1024 {
            text.push_str(&event.line);
            text.push('\n');
        }
    })
    .await?;
    if result.timed_out {
        return Err(crate::messages::with_detail("backend.timeout", &text));
    }
    if result.code != Some(0) {
        return Err(crate::messages::encode(
            "backend.commandFailed",
            serde_json::json!({"code": result.code.map(|code| code.to_string()).unwrap_or_else(|| "—".into())}),
            &text,
        ));
    }
    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn arguments_are_separate_and_config_is_relative() {
        let site = Site {
            id: "1".into(),
            name: "test".into(),
            path: "/tmp/site with spaces/Movefile.yml".into(),
        };
        let mut request = RunRequest {
            site_id: "1".into(),
            environment: "staging".into(),
            direction: "pull".into(),
            targets: vec!["themes".into()],
            simulate: true,
        };
        assert_eq!(
            arguments(&request, &site).unwrap(),
            [
                "pull",
                "--config",
                "./Movefile.yml",
                "--environment",
                "staging",
                "--themes",
                "--simulate"
            ]
        );
        request.targets.clear();
        assert!(arguments(&request, &site).is_err());
        request.targets.push("db;touch /tmp/unsafe".into());
        assert!(arguments(&request, &site).is_err());
        request.targets = vec!["db".into()];
        request.environment = "--all".into();
        assert!(arguments(&request, &site).is_err());
    }
    #[test]
    fn parses_wordmove_list_with_multiple_remotes() {
        let result = parse_environments("▬▬ Listing Local ▬▬\nlocal: http://site.local\n▬▬ Listing Remotes ▬▬\nstaging: https://stage.example.com\nproduction: https://example.com\n").unwrap();
        assert_eq!(result.environments.len(), 2);
        assert_eq!(result.local, "http://site.local");
        assert!(parse_environments("an error").is_err());
    }
}

#[cfg(test)]
mod discovery_tests {
    use super::*;
    #[test]
    fn prioritizes_installed_rbenv_over_abandoned_shims() {
        let home = tempfile::tempdir().unwrap();
        let current = home.path().join(".anyenv/envs/rbenv");
        std::fs::create_dir_all(current.join("bin")).unwrap();
        std::fs::write(current.join("bin/rbenv"), "manager").unwrap();
        std::fs::create_dir_all(home.path().join(".rbenv/shims")).unwrap();
        let paths = rbenv_paths(home.path());
        assert_eq!(paths.first(), Some(&current.join("shims")));
    }
}
