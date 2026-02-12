use tauri::command;
use std::fs;
use std::path::PathBuf;

#[command]
pub fn rename_recording(old_path: String, new_name: String) -> Result<String, String> {
    let old_pathbuf = PathBuf::from(&old_path);

    if !old_pathbuf.exists() {
        return Err("ファイルが見つかりません".to_string());
    }

    let parent = old_pathbuf.parent().ok_or("親ディレクトリが見つかりません")?;
    let extension = old_pathbuf.extension().and_then(|e| e.to_str()).unwrap_or("mp4");
    let new_pathbuf = parent.join(format!("{}.{}", new_name, extension));

    fs::rename(&old_pathbuf, &new_pathbuf)
        .map_err(|e| format!("名前変更に失敗: {}", e))?;

    Ok(new_pathbuf.to_string_lossy().to_string())
}

#[command]
pub fn list_recordings() -> Result<Vec<String>, String> {
    let recordings_dir = PathBuf::from("./recordings");

    if !recordings_dir.exists() {
        return Ok(vec![]);
    }

    let mut files = vec![];
    for entry in fs::read_dir(&recordings_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("mp4") {
            files.push(path.to_string_lossy().to_string());
        }
    }

    Ok(files)
}
