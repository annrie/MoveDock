use crate::model::*;
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    fs,
    io::Write,
    path::{Path, PathBuf},
};
use tokio::sync::{mpsc, Mutex};

pub struct Trusted {
    pub revision: String,
    pub settings: Settings,
    pub inspection: Inspection,
}
pub struct State {
    pub data: Mutex<AppData>,
    pub trusted: Mutex<HashMap<String, Trusted>>,
    pub active: Mutex<Option<mpsc::Sender<()>>>,
    pub file: PathBuf,
}
impl State {
    pub fn open(directory: PathBuf) -> Result<Self, String> {
        fs::create_dir_all(&directory).map_err(|e| e.to_string())?;
        let file = directory.join("settings.json");
        let data = match fs::read(&file) {
            Ok(bytes) => serde_json::from_slice(&bytes)
                .map_err(|e| format!("設定ファイルを読めません: {e}"))?,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => AppData::default(),
            Err(e) => return Err(e.to_string()),
        };
        Ok(Self {
            data: Mutex::new(data),
            trusted: Mutex::new(HashMap::new()),
            active: Mutex::new(None),
            file,
        })
    }
    pub fn save(&self, data: &AppData) -> Result<(), String> {
        atomic_write(
            &self.file,
            &serde_json::to_vec_pretty(data).map_err(|e| e.to_string())?,
        )
    }
}
pub fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let parent = path.parent().ok_or("保存先が不正です")?;
    let mut file = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
    if let Ok(meta) = fs::metadata(path) {
        file.as_file()
            .set_permissions(meta.permissions())
            .map_err(|e| e.to_string())?;
    }
    file.write_all(bytes).map_err(|e| e.to_string())?;
    file.as_file().sync_all().map_err(|e| e.to_string())?;
    file.persist(path).map_err(|e| e.to_string())?;
    Ok(())
}
pub fn read_document(path: &str) -> Result<Document, String> {
    let meta = fs::metadata(path).map_err(|e| e.to_string())?;
    if meta.len() > 1024 * 1024 {
        return Err("Movefile は 1 MB 以下にしてください".into());
    }
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let revision = format!("{:x}", Sha256::digest(content.as_bytes()));
    Ok(Document { content, revision })
}
pub fn site(data: &AppData, id: &str) -> Result<Site, String> {
    data.sites
        .iter()
        .find(|s| s.id == id)
        .cloned()
        .ok_or("サイトが見つかりません".into())
}
