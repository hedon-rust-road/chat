use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use sha1::{Digest, Sha1};

use crate::AppError;

use super::ChatFile;

impl ChatFile {
    pub fn new(ws_id: u64, filename: &str, data: &[u8]) -> Self {
        let hash = Sha1::digest(data);
        Self {
            ws_id,
            ext: filename.split('.').last().unwrap_or("txt").to_string(),
            hash: hex::encode(hash),
        }
    }

    pub fn url(&self) -> String {
        format!("/files/{}", self.hash_to_path())
    }

    pub fn path(&self, base_dir: &Path) -> PathBuf {
        base_dir.join(self.hash_to_path())
    }

    fn hash_to_path(&self) -> String {
        let (part1, part2) = self.hash.split_at(3);
        let (part2, part3) = part2.split_at(3);
        format!("{}/{part1}/{part2}/{part3}.{}", self.ws_id, self.ext)
    }
}

impl FromStr for ChatFile {
    type Err = AppError;

    // convert /files/1/a04/90d/e8a83ec42176fed247fae142cb749b9aa1.sql to ChatFile
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // 1/a04/90d/e8a83ec42176fed247fae142cb749b9aa1.sql
        let Some(s) = s.strip_prefix("/files/") else {
            return Err(AppError::ChatFileError(
                "Invalid chat file path".to_string(),
            ));
        };

        // 1
        // a04
        // 90d
        // e8a83ec42176fed247fae142cb749b9aa1.sql
        let parts: Vec<&str> = s.split('/').collect();
        if parts.len() != 4 {
            return Err(AppError::ChatFileError(
                "File path dose not valid".to_string(),
            ));
        }

        let Ok(ws_id) = parts[0].parse::<u64>() else {
            return Err(AppError::ChatFileError(format!(
                "Invalid workspace id: {}",
                parts[0]
            )));
        };

        // e8a83ec42176fed247fae142cb749b9aa1
        // sql
        let Some((part3, ext)) = parts[3].split_once('.') else {
            return Err(AppError::ChatFileError(format!(
                "Invalid file name: {}",
                parts[3]
            )));
        };

        // a0490de8a83ec42176fed247fae142cb749b9aa1
        let hash = format!("{}{}{}", parts[1], parts[2], part3);
        Ok(Self {
            ws_id,
            ext: ext.to_string(),
            hash,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chat_file_new_should_work() {
        let file = ChatFile::new(1, "test.txt", b"hello world");
        assert_eq!(file.ws_id, 1);
        assert_eq!(file.ext, "txt");
        assert_eq!(file.hash, "2aae6c35c94fcfb415dbe95f408b9ce91ee846ed")
    }
}
