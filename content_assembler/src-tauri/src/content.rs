use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

const LIMIT: u64 = 5_000_000;
const GROUPS: &str = ".graph-groups.json";
const SUFFIX: &str = ".node-graph.json";

fn filename(value: &Value) -> Result<&str, String> {
    let name = value.as_str().ok_or("파일 이름이 필요합니다.")?;
    let stem = name.split('.').next().unwrap_or("").to_ascii_lowercase();
    let reserved = ["con", "prn", "aux", "nul"].contains(&stem.as_str())
        || ((stem.starts_with("com") || stem.starts_with("lpt"))
            && stem.len() == 4
            && stem.as_bytes()[3].is_ascii_digit());
    if !name.ends_with(SUFFIX)
        || name.len() <= SUFFIX.len()
        || name.len() > 180
        || name.starts_with('.')
        || name.chars().any(|c| c < ' ' || "<>:\"/\\|?*".contains(c))
        || reserved
    {
        return Err(
            "파일 이름을 확인하세요. 폴더 경로 없이 .node-graph.json 이름을 사용하세요.".into(),
        );
    }
    Ok(name)
}

fn revision(text: &str) -> String {
    format!("{:x}", Sha256::digest(text.as_bytes()))
}

fn read_file(directory: &Path, name: &str) -> Result<Value, String> {
    let path = directory.join(name);
    let metadata = fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
    if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
        return Err("일반 콘텐츠 파일만 열 수 있습니다.".into());
    }
    if metadata.len() > LIMIT {
        return Err("파일 크기 제한은 5MB입니다.".into());
    }
    let mut text = String::new();
    File::open(path)
        .map_err(|e| e.to_string())?
        .take(LIMIT + 1)
        .read_to_string(&mut text)
        .map_err(|e| e.to_string())?;
    if text.len() as u64 > LIMIT {
        return Err("파일 크기 제한은 5MB입니다.".into());
    }
    Ok(json!({"name": name, "revision": revision(&text), "json": text}))
}

fn empty_groups() -> Value {
    json!({"format":"graph-groups","version":1,"groups":[]})
}

fn validate_groups(value: &Value) -> Result<(), String> {
    let invalid = "그룹 정보가 올바르지 않습니다. 이름 중복·빈 이름·파일 중복을 확인하세요.";
    if value["format"] != "graph-groups" || value["version"] != 1 {
        return Err(invalid.into());
    }
    let groups = value["groups"].as_array().ok_or(invalid)?;
    if groups.len() > 200 {
        return Err(invalid.into());
    }
    let mut ids = std::collections::HashSet::new();
    let mut names = std::collections::HashSet::new();
    let mut files = std::collections::HashSet::new();
    for group in groups {
        let id = group["id"].as_str().ok_or(invalid)?;
        let name = group["name"].as_str().ok_or(invalid)?;
        if id.is_empty()
            || id.len() > 80
            || !id
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b'-')
            || !ids.insert(id)
            || name.trim().is_empty()
            || name.trim() != name
            || name.chars().count() > 80
            || name.chars().any(|c| c < ' ')
            || !names.insert(name)
        {
            return Err(invalid.into());
        }
        for file in group["files"].as_array().ok_or(invalid)? {
            let name = filename(file)?;
            if !files.insert(name) {
                return Err(invalid.into());
            }
        }
    }
    Ok(())
}

fn read_groups(directory: &Path) -> Result<Value, String> {
    match fs::symlink_metadata(directory.join(GROUPS)) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            Ok(json!({"document":empty_groups(),"revision":null}))
        }
        Err(error) => Err(error.to_string()),
        Ok(_) => {
            let file = read_file(directory, GROUPS)?;
            let document: Value =
                serde_json::from_str(file["json"].as_str().unwrap()).map_err(|e| e.to_string())?;
            validate_groups(&document)?;
            Ok(json!({"document":document,"revision":file["revision"]}))
        }
    }
}

struct WriteLock {
    path: PathBuf,
    file: Option<File>,
    temporary: Option<PathBuf>,
}
impl Drop for WriteLock {
    fn drop(&mut self) {
        drop(self.file.take());
        if let Some(path) = &self.temporary {
            let _ = fs::remove_file(path);
        }
        let _ = fs::remove_file(&self.path);
    }
}

pub fn operation(directory: &Path, input: &Value) -> Result<Value, String> {
    fs::create_dir_all(directory).map_err(|e| e.to_string())?;
    let metadata = fs::symlink_metadata(directory).map_err(|e| e.to_string())?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err("콘텐츠 폴더는 실제 디렉터리여야 합니다.".into());
    }
    if input["operation"] == "list" {
        let mut names = Vec::new();
        for entry in fs::read_dir(directory).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let kind = entry.file_type().map_err(|e| e.to_string())?;
            if kind.is_file()
                && let Some(name) = entry.file_name().to_str()
                && filename(&json!(name)).is_ok()
            {
                names.push(name.to_string());
            }
        }
        names.sort();
        return Ok(
            json!({"directory": directory.to_string_lossy(), "files": names, "organization": read_groups(directory)?}),
        );
    }
    let grouping = input["operation"] == "save-groups";
    let name = if grouping {
        GROUPS
    } else {
        filename(&input["name"])?
    };
    if input["operation"] == "read" {
        return read_file(directory, name);
    }
    if input["operation"] != "save" && !grouping {
        return Err("잘못된 콘텐츠 요청입니다.".into());
    }
    let text = input["json"].as_str().ok_or("저장할 JSON이 필요합니다.")?;
    if text.len() as u64 > LIMIT {
        return Err("파일 크기 제한은 5MB입니다.".into());
    }
    let document: Value = serde_json::from_str(text).map_err(|e| e.to_string())?;
    if grouping {
        validate_groups(&document)?;
    } else if document["format"] != "node-graph" || document["version"] != 1 {
        return Err("지원하는 노드 그래프 파일이 아닙니다.".into());
    }
    let expected = input.get("expected").ok_or("저장 기준이 필요합니다.")?;
    if !expected.is_null() && !expected.is_string() {
        return Err("저장 기준이 올바르지 않습니다.".into());
    }
    let lock_path = directory.join(".content-write.lock");
    let lock = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&lock_path)
        .map_err(|error| {
            if error.kind() == std::io::ErrorKind::AlreadyExists {
                "콘텐츠 폴더에서 다른 저장이 진행 중입니다. 잠시 후 다시 저장하세요.".to_string()
            } else {
                error.to_string()
            }
        })?;
    let mut guard = WriteLock {
        path: lock_path,
        file: Some(lock),
        temporary: None,
    };
    let path = directory.join(name);
    let previous = match fs::symlink_metadata(&path) {
        Ok(_) => read_file(directory, name)?["revision"].clone(),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Value::Null,
        Err(error) => return Err(error.to_string()),
    };
    if previous != *expected {
        if grouping {
            return Err(
                "그룹이 외부에서 변경되었습니다. 편집을 취소하고 새로고침한 뒤 다시 적용하세요."
                    .into(),
            );
        }
        return Err("같은 이름의 파일이 있거나 파일이 외부에서 변경되었습니다. 다시 열거나 다른 이름으로 저장하세요.".into());
    }
    let temporary = directory.join(".content-write.tmp");
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(|e| e.to_string())?;
    guard.temporary = Some(temporary.clone());
    file.write_all(text.as_bytes()).map_err(|e| e.to_string())?;
    file.sync_all().map_err(|e| e.to_string())?;
    drop(file);
    fs::rename(&temporary, path).map_err(|e| e.to_string())?;
    if grouping {
        Ok(json!({"document": document,"revision":revision(text)}))
    } else {
        Ok(json!({"name": name, "json": text, "revision": revision(text)}))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn folder() -> PathBuf {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("content-files-{}-{unique}", std::process::id()))
    }
    #[test]
    fn roundtrip_conflict_and_invalid_paths_preserve_file() {
        let root = folder();
        let name = "한글 콘텐츠.node-graph.json";
        let text = r#"{"format":"node-graph","version":1}"#;
        let first = operation(
            &root,
            &json!({"operation":"save","name":name,"json":text,"expected":null}),
        )
        .unwrap();
        assert_eq!(
            operation(&root, &json!({"operation":"read","name":name})).unwrap(),
            first
        );
        assert_eq!(
            operation(&root, &json!({"operation":"list"})).unwrap()["files"],
            json!([name])
        );
        assert!(
            operation(
                &root,
                &json!({"operation":"save","name":name,"json":text,"expected":null})
            )
            .is_err()
        );
        assert!(
            operation(
                &root,
                &json!({"operation":"read","name":"../escape.node-graph.json"})
            )
            .is_err()
        );
        assert!(
            operation(
                &root,
                &json!({"operation":"save","name":name,"json":"bad","expected":first["revision"]})
            )
            .is_err()
        );
        let updated = format!("{text}\n");
        operation(
            &root,
            &json!({"operation":"save","name":name,"json":updated,"expected":first["revision"]}),
        )
        .unwrap();
        assert_eq!(fs::read_to_string(root.join(name)).unwrap(), updated);
        assert!(!root.join(".content-write.lock").exists());
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn group_roundtrip_conflicts_validation_and_graph_preservation() {
        let root = folder();
        fs::create_dir_all(&root).unwrap();
        let name = "예제.node-graph.json";
        fs::write(root.join(name), "preserve").unwrap();
        assert_eq!(
            operation(&root, &json!({"operation":"list"})).unwrap()["organization"]["revision"],
            Value::Null
        );
        let document = json!({"format":"graph-groups","version":1,"groups":[{"id":"first","name":"공명 예제","files":[name]}]});
        let first = operation(
            &root,
            &json!({"operation":"save-groups","json":document.to_string(),"expected":null}),
        )
        .unwrap();
        assert_eq!(
            operation(&root, &json!({"operation":"list"})).unwrap()["organization"],
            first
        );
        assert!(operation(&root, &json!({"operation":"save-groups","json":empty_groups().to_string(),"expected":null})).is_err());
        let mut invalid = document.clone();
        invalid["groups"][0]["files"] = json!([name, name]);
        assert!(operation(&root, &json!({"operation":"save-groups","json":invalid.to_string(),"expected":first["revision"]})).is_err());
        operation(&root, &json!({"operation":"save-groups","json":empty_groups().to_string(),"expected":first["revision"]})).unwrap();
        assert_eq!(fs::read_to_string(root.join(name)).unwrap(), "preserve");
        fs::write(root.join(GROUPS), "broken").unwrap();
        assert!(operation(&root, &json!({"operation":"list"})).is_err());
        assert!(operation(&root, &json!({"operation":"save-groups","json":empty_groups().to_string(),"expected":null})).is_err());
        assert_eq!(fs::read_to_string(root.join(GROUPS)).unwrap(), "broken");
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn symlinks_are_not_listed_or_read_or_overwritten() {
        let root = folder();
        fs::create_dir_all(&root).unwrap();
        let outside = root.with_extension("json");
        fs::write(&outside, "preserve").unwrap();
        let name = "link.node-graph.json";
        std::os::unix::fs::symlink(&outside, root.join(name)).unwrap();
        assert_eq!(
            operation(&root, &json!({"operation":"list"})).unwrap()["files"],
            json!([])
        );
        assert!(operation(&root, &json!({"operation":"read","name":name})).is_err());
        assert!(operation(&root, &json!({"operation":"save","name":name,"json":r#"{"format":"node-graph","version":1}"#,"expected":null})).is_err());
        std::os::unix::fs::symlink(&outside, root.join(GROUPS)).unwrap();
        assert!(operation(&root, &json!({"operation":"list"})).is_err());
        assert!(operation(&root, &json!({"operation":"save-groups","json":empty_groups().to_string(),"expected":null})).is_err());
        assert_eq!(fs::read_to_string(&outside).unwrap(), "preserve");
        fs::remove_dir_all(root).unwrap();
        fs::remove_file(outside).unwrap();
    }
}
