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
pub fn path_env(settings: &Settings) -> OsString {
    let mut paths: Vec<PathBuf> = settings
        .extra_path
        .lines()
        .filter(|p| !p.trim().is_empty())
        .map(|p| expand(p.trim()))
        .collect();
    if let Some(home) = dirs::home_dir() {
        paths.extend([
            home.join(".rbenv/shims"),
            home.join(".anyenv/envs/rbenv/shims"),
            home.join(".local/bin"),
        ]);
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
        format!("{name} が見つかりません。設定で実行ファイル・追加 PATH を確認してください。")
    })
}
pub fn base(settings: &Settings, cwd: &Path) -> Result<Command, String> {
    if !["direct", "bundler"].contains(&settings.mode.as_str()) {
        return Err("実行方式が不正です".into());
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
    ] {
        command.env_remove(key);
    }
    if !settings.ruby_version.trim().is_empty() {
        command.env("RBENV_VERSION", settings.ruby_version.trim());
    }
    if settings.mode == "bundler" {
        let gemfile = expand(settings.gemfile.trim())
            .canonicalize()
            .map_err(|_| "設定で Wordmove フォークの Gemfile を指定してください")?;
        if !gemfile.is_file() {
            return Err("Gemfile がファイルではありません".into());
        }
        // Reuse bundle config from the selected fork, not the WordPress site's config.
        command.env(
            "BUNDLE_APP_CONFIG",
            gemfile.parent().unwrap().join(".bundle"),
        );
        command
            .env("BUNDLE_GEMFILE", &gemfile)
            .args(["exec", "wordmove"]);
    }
    Ok(command)
}
pub fn config_name(path: &str) -> Result<String, String> {
    let name = Path::new(path)
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or("Movefile のファイル名が不正です")?;
    if name.contains(['*', '?', '[', ']', '{', '}', '\\', '\n', '\r']) {
        return Err("Movefile の名前には glob 特殊文字や改行を使用できません".into());
    }
    // Wordmove joins --config onto its working directory. Use a relative filename.
    Ok(format!("./{name}"))
}
pub fn arguments(request: &RunRequest, site: &Site) -> Result<Vec<String>, String> {
    if !["push", "pull"].contains(&request.direction.as_str()) {
        return Err("同期方向が不正です".into());
    }
    if request.environment.is_empty()
        || request.environment.starts_with('-')
        || request.environment.contains(['\n', '\r', '\0'])
    {
        return Err("環境名が不正です".into());
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
        return Err("同期対象を選択してください".into());
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
        return Err(
            "環境一覧を取得できません。Movefile のリモート環境に vhost を設定してください。".into(),
        );
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
        return Err(format!("コマンドがタイムアウトしました。\n{text}"));
    }
    if result.code != Some(0) {
        return Err(format!(
            "コマンドが失敗しました（終了コード {:?}）。\n{text}",
            result.code
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
