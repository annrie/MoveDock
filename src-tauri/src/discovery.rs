use crate::{cli, model::Settings};
use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};

pub fn is_unconfigured(settings: &Settings) -> bool {
    settings.mode == "bundler"
        && [
            &settings.executable,
            &settings.gemfile,
            &settings.extra_path,
            &settings.ruby_version,
        ]
        .iter()
        .all(|value| value.trim().is_empty())
}

fn executable(name: &str, paths: &OsStr) -> Option<PathBuf> {
    which::which_in(name, Some(paths), "/").ok()
}
fn text(path: &Path) -> Option<String> {
    let value = fs::read_to_string(path).ok()?;
    let value = value.trim();
    (!value.is_empty()
        && value.len() < 100
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || ".-_".contains(c)))
    .then(|| value.to_owned())
}
fn project_candidates(home: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    for parent in ["work", "Projects", "projects", "Developer", "Code", "src"] {
        let dir = home.join(parent).join("wordmove");
        if dir.join("wordmove.gemspec").is_file() && dir.join("Gemfile").is_file() {
            if let Ok(file) = dir.join("Gemfile").canonicalize() {
                if !found.contains(&file) {
                    found.push(file);
                }
            }
        }
    }
    found
}
fn version_key(name: &str) -> Vec<u64> {
    name.split(|c: char| !c.is_ascii_digit())
        .filter_map(|s| s.parse().ok())
        .collect()
}
fn local_bin(home: &Path, service: &str, binary: &str) -> Option<PathBuf> {
    let root = home.join("Library/Application Support/Local/lightning-services");
    let mut entries: Vec<_> = fs::read_dir(root)
        .ok()?
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .starts_with(&format!("{service}-"))
        })
        .collect();
    entries.sort_by_key(|entry| version_key(&entry.file_name().to_string_lossy()));
    for entry in entries.into_iter().rev() {
        let platform = if cfg!(target_arch = "aarch64") {
            "darwin-arm64"
        } else {
            "darwin-x64"
        };
        for folder in [format!("bin/{platform}/bin"), "bin/darwin/bin".into()] {
            let dir = entry.path().join(folder);
            if executable(binary, dir.as_os_str()).is_some() {
                return Some(dir);
            }
        }
    }
    None
}

// Filesystem discovery only: never evaluate Gemfile, ERB or shell startup files here.
pub fn fill(settings: &mut Settings, home: &Path, search_path: &OsStr) -> Vec<String> {
    let untouched = is_unconfigured(settings);
    let mut notes = Vec::new();
    if settings.mode == "bundler" && settings.gemfile.trim().is_empty() {
        let projects = project_candidates(home);
        match projects.as_slice() {
            [file] => settings.gemfile = file.to_string_lossy().into_owned(),
            [] if untouched && executable("wordmove", search_path).is_some() => settings.mode = "direct".into(),
            [] => notes.push("Wordmove の Gemfile が見つかりません。Bundler を使う場合は「選択」で指定してください。".into()),
            _ => notes.push("Wordmove の Gemfile が複数見つかりました。「選択」で使用するものを指定してください。".into()),
        }
    }
    if settings.ruby_version.trim().is_empty() {
        let project_version = (!settings.gemfile.is_empty())
            .then(|| cli::expand(&settings.gemfile))
            .and_then(|file| file.parent().and_then(|p| text(&p.join(".ruby-version"))));
        let global_version = [home.join(".rbenv"), home.join(".anyenv/envs/rbenv")]
            .iter()
            .find_map(|root| text(&root.join("version")));
        settings.ruby_version = project_version.or(global_version).unwrap_or_default();
    }
    if settings.extra_path.trim().is_empty() {
        let mut additional = Vec::new();
        for (binary, service) in [("php", "php"), ("mysql", "mysql")] {
            if executable(binary, search_path).is_some() {
                continue;
            }
            let homebrew = ["/opt/homebrew", "/usr/local"]
                .iter()
                .flat_map(|prefix| {
                    let packages = if binary == "php" {
                        vec!["php"]
                    } else {
                        vec!["mysql-client", "mysql", "mariadb"]
                    };
                    packages
                        .into_iter()
                        .map(move |package| PathBuf::from(format!("{prefix}/opt/{package}/bin")))
                })
                .find(|dir| executable(binary, dir.as_os_str()).is_some());
            if let Some(dir) = homebrew
                .or_else(|| local_bin(home, service, binary))
                .or_else(|| {
                    (binary == "mysql")
                        .then(|| local_bin(home, "mariadb", binary))
                        .flatten()
                })
            {
                if !additional.contains(&dir) {
                    additional.push(dir);
                }
            }
        }
        settings.extra_path = additional
            .iter()
            .map(|dir| dir.to_string_lossy())
            .collect::<Vec<_>>()
            .join("\n");
    }
    if settings.executable.trim().is_empty() {
        let paths = std::env::join_paths(
            settings
                .extra_path
                .lines()
                .map(PathBuf::from)
                .chain(std::env::split_paths(search_path)),
        )
        .unwrap_or_default();
        let name = if settings.mode == "bundler" {
            "bundle"
        } else {
            "wordmove"
        };
        if let Some(path) = executable(name, &paths) {
            settings.executable = path.to_string_lossy().into_owned();
        } else {
            notes.push(format!(
                "{name} が見つかりません。導入後に「空欄を再検出」を実行してください。"
            ));
        }
    }
    notes.insert(
        0,
        "検出できた項目を自動入力しました。実際の起動状況は「実行環境を確認」で確認できます。"
            .into(),
    );
    notes
}

pub fn detect(settings: &mut Settings) -> Vec<String> {
    let Some(home) = dirs::home_dir() else {
        return vec!["ホームフォルダを取得できませんでした。".into()];
    };
    let paths = cli::path_env(settings);
    fill(settings, &home, &paths)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fake_binary(dir: &Path, name: &str) {
        use std::os::unix::fs::PermissionsExt;
        fs::create_dir_all(dir).unwrap();
        let file = dir.join(name);
        fs::write(&file, "#!/bin/sh\nexit 91\n").unwrap();
        fs::set_permissions(file, fs::Permissions::from_mode(0o755)).unwrap();
    }
    #[test]
    fn discovers_fork_and_ruby_without_executing_project_code() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path();
        let project = home.join("work/wordmove");
        fs::create_dir_all(&project).unwrap();
        fs::write(project.join("Gemfile"), "raise 'must not execute'\n").unwrap();
        fs::write(
            project.join("wordmove.gemspec"),
            "raise 'must not execute'\n",
        )
        .unwrap();
        fs::write(project.join(".ruby-version"), "3.3.12\n").unwrap();
        let bin = home.join("bin");
        fake_binary(&bin, "bundle");
        let mut settings = Settings::default();
        fill(&mut settings, home, bin.as_os_str());
        assert_eq!(settings.mode, "bundler");
        assert_eq!(settings.ruby_version, "3.3.12");
        assert_eq!(
            PathBuf::from(settings.gemfile),
            project.join("Gemfile").canonicalize().unwrap()
        );
        assert_eq!(PathBuf::from(settings.executable), bin.join("bundle"));
    }
    #[test]
    fn falls_back_to_direct_and_keeps_custom_values() {
        let dir = tempfile::tempdir().unwrap();
        let bin = dir.path().join("bin");
        fake_binary(&bin, "wordmove");
        let mut settings = Settings::default();
        fill(&mut settings, dir.path(), bin.as_os_str());
        assert_eq!(settings.mode, "direct");
        assert_eq!(PathBuf::from(&settings.executable), bin.join("wordmove"));
        settings.ruby_version = "custom".into();
        settings.extra_path = "/custom/bin".into();
        let before = settings.clone();
        fill(&mut settings, dir.path(), bin.as_os_str());
        assert_eq!(settings, before);
    }
    #[test]
    fn does_not_choose_between_multiple_projects_and_sorts_local_versions() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path();
        for parent in ["work", "Projects"] {
            let project = home.join(parent).join("wordmove");
            fs::create_dir_all(&project).unwrap();
            fs::write(project.join("Gemfile"), "").unwrap();
            fs::write(project.join("wordmove.gemspec"), "").unwrap();
        }
        let mut settings = Settings::default();
        let notes = fill(&mut settings, home, OsStr::new("/nonexistent"));
        assert!(settings.gemfile.is_empty());
        assert!(notes.iter().any(|n| n.contains("複数")));
        for version in ["8.3.9+0", "8.3.23+0"] {
            fake_binary(&home.join(format!("Library/Application Support/Local/lightning-services/php-{version}/bin/darwin/bin")), "php");
        }
        assert!(local_bin(home, "php", "php")
            .unwrap()
            .to_string_lossy()
            .contains("8.3.23"));
    }
}
