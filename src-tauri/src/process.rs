use crate::model::LogEvent;
use regex::Regex;
use std::{process::Stdio, sync::OnceLock, time::Duration};
use tokio::{
    io::{AsyncRead, AsyncReadExt},
    process::Command,
    sync::mpsc,
};

pub struct Outcome {
    pub code: Option<i32>,
    pub cancelled: bool,
    pub timed_out: bool,
}
struct Group(u32);
impl Drop for Group {
    fn drop(&mut self) {
        // Each child is placed in its own group; never signal the GUI's process group.
        #[cfg(unix)]
        unsafe {
            libc::kill(-(self.0 as i32), libc::SIGKILL);
        }
    }
}
pub fn clean(raw: &str) -> String {
    static ANSI: OnceLock<Regex> = OnceLock::new();
    static SECRET: OnceLock<Regex> = OnceLock::new();
    let ansi = ANSI.get_or_init(|| {
        Regex::new(r"\x1b(?:\[[0-?]*[ -/]*[@-~]|\][^\x07]*(?:\x07|\x1b\\))").unwrap()
    });
    let secrets = SECRET.get_or_init(|| Regex::new(r#"(?i)((?:--password|password|passwd|pwd|token|secret)\s*[=:]\s*)(?:'[^']*'|"[^"]*"|[^\s]+)"#).unwrap());
    let text = ansi.replace_all(raw, "");
    secrets
        .replace_all(&text, "${1}[redacted]")
        .chars()
        .filter(|c| !c.is_control() || *c == '\t')
        .collect()
}
async fn read_stream(mut reader: impl AsyncRead + Unpin, stream: &str, tx: mpsc::Sender<LogEvent>) {
    let mut buf = [0; 4096];
    let mut pending = Vec::new();
    loop {
        match reader.read(&mut buf).await {
            Ok(0) | Err(_) => break,
            Ok(n) => {
                for byte in &buf[..n] {
                    if *byte == b'\n' || *byte == b'\r' || pending.len() >= 16384 {
                        if !pending.is_empty() {
                            let line = clean(&String::from_utf8_lossy(&pending));
                            if tx
                                .send(LogEvent {
                                    stream: stream.into(),
                                    line,
                                })
                                .await
                                .is_err()
                            {
                                return;
                            }
                            pending.clear();
                        }
                    }
                    if *byte != b'\n' && *byte != b'\r' {
                        pending.push(*byte);
                    }
                }
            }
        }
    }
    if !pending.is_empty() {
        let _ = tx
            .send(LogEvent {
                stream: stream.into(),
                line: clean(&String::from_utf8_lossy(&pending)),
            })
            .await;
    }
}
pub async fn execute(
    mut command: Command,
    mut cancel: mpsc::Receiver<()>,
    limit: Duration,
    mut output: impl FnMut(LogEvent),
) -> Result<Outcome, String> {
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    #[cfg(unix)]
    command.process_group(0);
    let mut child = command
        .spawn()
        .map_err(|e| format!("コマンドを起動できません: {e}"))?;
    let group = Group(child.id().ok_or("プロセス ID を取得できません")?);
    let (tx, mut rx) = mpsc::channel(128);
    let out = tokio::spawn(read_stream(
        child.stdout.take().unwrap(),
        "stdout",
        tx.clone(),
    ));
    let err = tokio::spawn(read_stream(child.stderr.take().unwrap(), "stderr", tx));
    let timer = tokio::time::sleep(limit);
    tokio::pin!(timer);
    let mut cancelled = false;
    let mut timed_out = false;
    let mut cancel_open = true;
    let mut streams_open = true;
    let status = loop {
        tokio::select! {
            event = rx.recv(), if streams_open => match event { Some(e) => output(e), None => streams_open = false },
            result = child.wait() => break result.map_err(|e| e.to_string())?,
            message = cancel.recv(), if cancel_open => {
                if message.is_some() { cancelled = true; break stop(&mut child, group.0).await?; }
                cancel_open = false;
            },
            _ = &mut timer => { timed_out = true; break stop(&mut child, group.0).await?; }
        }
    };
    // Kill descendants that retained pipes even if the direct child has exited.
    drop(group);
    while let Some(event) = rx.recv().await {
        output(event);
    }
    let _ = tokio::join!(out, err);
    Ok(Outcome {
        code: status.code(),
        cancelled,
        timed_out,
    })
}
async fn stop(
    child: &mut tokio::process::Child,
    pid: u32,
) -> Result<std::process::ExitStatus, String> {
    #[cfg(unix)]
    unsafe {
        libc::kill(-(pid as i32), libc::SIGTERM);
    }
    match tokio::time::timeout(Duration::from_secs(2), child.wait()).await {
        Ok(result) => result.map_err(|e| e.to_string()),
        Err(_) => {
            #[cfg(unix)]
            unsafe {
                libc::kill(-(pid as i32), libc::SIGKILL);
            }
            let _ = child.start_kill();
            child.wait().await.map_err(|e| e.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn captures_both_streams_and_failure() {
        let mut cmd = Command::new("/bin/sh");
        cmd.args(["-c", "printf 'out\\n'; printf 'err\\n' >&2; exit 7"]);
        let (_tx, rx) = mpsc::channel(1);
        let mut lines = Vec::new();
        let result = execute(cmd, rx, Duration::from_secs(5), |e| {
            lines.push((e.stream, e.line))
        })
        .await
        .unwrap();
        assert_eq!(result.code, Some(7));
        assert!(lines.contains(&("stdout".into(), "out".into())));
        assert!(lines.contains(&("stderr".into(), "err".into())));
    }
    #[tokio::test]
    async fn cancellation_and_timeout_finish_promptly() {
        for cancelled in [true, false] {
            let mut cmd = Command::new("/bin/sh");
            cmd.args(["-c", "sleep 30 & wait"]);
            let (tx, rx) = mpsc::channel(1);
            if cancelled {
                tx.send(()).await.unwrap();
            }
            let result = tokio::time::timeout(
                Duration::from_secs(5),
                execute(cmd, rx, Duration::from_millis(50), |_| {}),
            )
            .await
            .unwrap()
            .unwrap();
            assert_eq!(result.cancelled, cancelled);
            assert_eq!(result.timed_out, !cancelled);
        }
    }
    #[test]
    fn strips_colors_and_secrets() {
        assert_eq!(
            clean("\x1b[31mhello\x1b[0m --password='a b' token=secret"),
            "hello --password=[redacted] token=[redacted]"
        );
    }
}
