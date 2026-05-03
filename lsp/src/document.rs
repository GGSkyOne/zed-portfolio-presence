/*
 * This file is part of discord-presence. Extension for Zed that adds support for Discord Rich Presence using LSP.
 *
 * Copyright (c) 2024 Steinhübl
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>
 */

use crate::error::{PresenceError, Result};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use tower_lsp::lsp_types::Url;

#[derive(Debug, Clone)]
pub struct Document {
    path: PathBuf,
    workspace_root: PathBuf,
    line_number: Option<u32>,
}

impl Document {
    pub fn new(url: &Url, workspace_root: &Path, line_number: Option<u32>) -> Self {
        Self {
            path: url
                .to_file_path()
                .unwrap_or_else(|()| PathBuf::from(url.path())),
            workspace_root: workspace_root.to_owned(),
            line_number,
        }
    }

    pub fn get_line_number(&self) -> Option<u32> {
        self.line_number
    }

    pub fn get_filename(&self) -> Result<String> {
        let filename = self
            .path
            .file_name()
            .ok_or_else(|| PresenceError::Config("No filename found".to_string()))?
            .to_str()
            .ok_or_else(|| PresenceError::Config("Invalid filename encoding".to_string()))?;

        Ok(filename.to_string())
    }

    pub fn get_extension(&self) -> &str {
        self.path
            .extension()
            .unwrap_or(OsStr::new(""))
            .to_str()
            .unwrap_or("")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[cfg(windows)]
    fn workspace_root() -> PathBuf {
        PathBuf::from(r"C:\Users\tester\project")
    }

    #[cfg(not(windows))]
    fn workspace_root() -> PathBuf {
        PathBuf::from("/home/user/project")
    }

    #[test]
    fn test_document_creation() {
        let workspace_root = workspace_root();
        let file_path = workspace_root.join("src").join("test.rs");
        let url = Url::from_file_path(&file_path).unwrap();
        let doc = Document::new(&url, &workspace_root, None);

        assert_eq!(doc.get_filename().unwrap(), "test.rs");
        assert_eq!(doc.get_extension(), "rs");
    }

    #[test]
    fn test_document_with_encoded_filename() {
        let workspace_root = workspace_root();
        let file_path = workspace_root.join("test file.rs");
        let url = Url::from_file_path(&file_path).unwrap();
        let doc = Document::new(&url, &workspace_root, None);

        assert_eq!(doc.get_filename().unwrap(), "test file.rs");
    }
}
