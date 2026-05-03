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

mod rules;
mod update;

use std::collections::HashMap;

pub use rules::Rules;
use tracing::{debug, info, instrument};
use update::UpdateFromJson;

use serde_json::{Map, Value};

use crate::error::Result;

#[derive(Debug, Clone)]
pub struct Configuration {
    pub rules: Rules,
    pub git_integration: bool,
    pub git_host_overrides: HashMap<String, String>,
    pub endpoint_url: Option<String>,
    pub http_secret: Option<String>,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            rules: Rules::default(),
            git_integration: true,
            git_host_overrides: HashMap::default(),
            endpoint_url: None,
            http_secret: None,
        }
    }
}

impl UpdateFromJson for Configuration {
    fn update_from_json(&mut self, json: &Value) -> Result<()> {
        if let Some(rules) = json.get("rules") {
            self.rules.update_from_json(rules)?;
        }

        if let Some(git_integration) = json.get("git_integration") {
            self.git_integration = git_integration.as_bool().unwrap_or(true);
        }

        if let Some(git_host_overrides) = json.get("git_host_overrides") {
            for (key, value) in git_host_overrides.as_object().unwrap_or(&Map::default()) {
                if let Some(v) = value.as_str() {
                    self.git_host_overrides.insert(key.to_owned(), v.to_owned());
                }
            }
        }

        if let Some(endpoint_url) = json.get("endpoint_url").and_then(Value::as_str) {
            self.endpoint_url = Some(endpoint_url.to_string());
        }

        if let Some(http_secret) = json.get("http_secret").and_then(Value::as_str) {
            self.http_secret = Some(http_secret.to_string());
        }

        Ok(())
    }
}

impl Configuration {
    #[instrument(skip(self, options))]
    pub fn update(&mut self, options: Option<Value>) -> Result<()> {
        if let Some(options) = options {
            debug!("Updating configuration from provided options");
            self.update_from_json(&options)?;
            info!("Configuration updated successfully");
        } else {
            debug!("No configuration options provided, using defaults");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_configuration() {
        let config = Configuration::default();
        assert!(config.git_integration);
        assert!(config.endpoint_url.is_none());
        assert!(config.http_secret.is_none());
    }

    #[test]
    fn test_update_configuration() {
        let mut config = Configuration::default();
        let json = serde_json::json!({
            "git_integration": false
        });

        config.update(Some(json)).unwrap();
        assert!(!config.git_integration);
    }
}
