/*
 * Copyright © 2025 Mitja Leino
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated
 * documentation files (the “Software”), to deal in the Software without restriction, including without limitation
 * the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software,
 * and to permit persons to whom the Software is furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE
 * WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS
 * OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT,
 * TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
 */
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProfilesConfig {
    #[serde(default = "default_profiles")]
    pub profiles: Vec<Profile>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Profile {
    pub name: String,
    pub provider: String,
    pub models: Vec<Model>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Model {
    pub model: String,
    pub description: Option<String>,
    #[serde(default = "default_model_type")]
    pub model_type: ModelType,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ModelType {
    Fast,
    Balanced,
    Deep,
}

impl Display for ModelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModelType::Fast => write!(f, "fast"),
            ModelType::Balanced => write!(f, "balanced"),
            ModelType::Deep => write!(f, "deep"),
        }
    }
}

impl ProfilesConfig {
    /// Validate profiles
    /// 1. At least one profile must be defined
    /// 2. Each profile must have a unique name
    /// 3. Each profile must be valid
    pub fn validate(&self) -> Result<(), String> {
        if self.profiles.is_empty() {
            return Err("No profiles defined".to_string());
        }

        let mut names: Vec<String> = vec![];

        for profile in &self.profiles {
            if names.contains(&profile.name) {
                return Err(format!("Profile name {} is not unique", profile.name));
            }

            names.push(profile.name.clone());

            profile.validate(&profile.name)?;
        }
        Ok(())
    }
}

impl Profile {
    /// 1. The profile must have at least one model
    /// 2. Each model must have a unique model type
    pub fn validate(&self, profile_name: &String) -> Result<(), String> {
        if self.models.is_empty() {
            return Err(format!("Profile {} has no models", profile_name));
        }

        let mut model_types: Vec<ModelType> = vec![];

        for model in &self.models {
            if model_types.contains(&model.model_type) {
                return Err(format!("Profile {} has a duplicate model type: {}", profile_name, &model.model_type));
            }

            model_types.push(model.model_type.clone());
        }

        Ok(())
    }
}

fn default_model_type() -> ModelType {
    ModelType::Balanced
}

fn default_profiles() -> Vec<Profile> {
    let models: Vec<Model> = vec![
        Model {
            model: "qwen3:4b".to_string(),
            description: None,
            model_type: ModelType::Fast,
        }
    ];


    let profiles: Vec<Profile> = vec![
        Profile {
            name: "local".to_string(),
            provider: "ollama".to_string(),
            models,
        }
    ];

    profiles
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_profiles() {
        let config = ProfilesConfig {
            profiles: default_profiles()
        };

        assert_eq!(config.profiles.len(), 1);
        assert_eq!(config.profiles[0].name, "local");
        assert_eq!(config.profiles[0].provider, "ollama");
        assert_eq!(config.profiles[0].models.len(), 1);
        assert_eq!(config.profiles[0].models[0].model, "qwen3:4b");
        assert_eq!(config.profiles[0].models[0].model_type, ModelType::Fast);
        assert!(config.profiles[0].models[0].description.is_none());
    }

    #[test]
    fn test_parse_valid_config() {
        let config_str: &str = r#"
            [[profiles]]
            name = "test"
            provider = "ollama"

            [[profiles.models]]
            model = "qwen3:4b"
            model_type = "fast"
            description = "Fast model"

            [[profiles.models]]
            model = "gemma3:12b"
            model_type = "balanced"
            description = "Balanced model"

            [[profiles.models]]
            model = "mixtral:8x7b"
            model_type = "deep"
            description = "Deep model"
        "#;

        let config: ProfilesConfig = toml::from_str(config_str).unwrap();
        assert_eq!(config.profiles.len(), 1);
        assert_eq!(config.profiles[0].name, "test");
        assert_eq!(config.profiles[0].provider, "ollama");
        assert_eq!(config.profiles[0].models.len(), 3);
        assert_eq!(config.profiles[0].models[0].model, "qwen3:4b");
        assert_eq!(config.profiles[0].models[0].model_type, ModelType::Fast);
        assert_eq!(config.profiles[0].models[1].model, "gemma3:12b");
        assert_eq!(config.profiles[0].models[1].model_type, ModelType::Balanced);
        assert_eq!(config.profiles[0].models[2].model, "mixtral:8x7b");
        assert_eq!(config.profiles[0].models[2].model_type, ModelType::Deep);

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_empty_profiles() {
        let config_str = r#"
            profiles = []
        "#;

        let config: ProfilesConfig = toml::from_str(config_str).unwrap();
        assert_eq!(config.validate().unwrap_err(), "No profiles defined");
    }

    #[test]
    fn test_validate_duplicate_profile_names() {
        let config_str = r#"
            [[profiles]]
            name = "test"
            provider = "ollama"
            [[profiles.models]]
            model = "model1"
            model_type = "fast"

            [[profiles]]
            name = "test"
            provider = "ollama"
            [[profiles.models]]
            model = "model2"
            model_type = "fast"
        "#;

        let config: ProfilesConfig = toml::from_str(config_str).unwrap();
        assert_eq!(config.validate().unwrap_err(), "Profile name test is not unique");
    }

    #[test]
    fn test_validate_profile_no_models() {
        let config_str = r#"
            [[profiles]]
            name = "test"
            provider = "ollama"
            models = []
        "#;

        let config: ProfilesConfig = toml::from_str(config_str).unwrap();
        assert_eq!(config.validate().unwrap_err(), "Profile test has no models");
    }

    #[test]
    fn test_validate_default_model_type() {
        let config_str = r#"
            [[profiles]]
            name = "test"
            provider = "ollama"
            [[profiles.models]]
            model = "model1"
        "#;

        let config: ProfilesConfig = toml::from_str(config_str).unwrap();
        assert_eq!(config.profiles.len(), 1);
        assert_eq!(config.profiles[0].name, "test");
        assert_eq!(config.profiles[0].provider, "ollama");
        assert_eq!(config.profiles[0].models.len(), 1);
        assert_eq!(config.profiles[0].models[0].model, "model1");
        assert_eq!(config.profiles[0].models[0].model_type, ModelType::Balanced);
    }

    #[test]
    fn test_validate_duplicate_model_types() {
        let config_str = r#"
            [[profiles]]
            name = "test"
            provider = "ollama"

            [[profiles.models]]
            model = "model1"
            model_type = "fast"

            [[profiles.models]]
            model = "model2"
            model_type = "fast"
        "#;

        let config: ProfilesConfig = toml::from_str(config_str).unwrap();
        assert_eq!(config.validate().unwrap_err(), "Profile test has a duplicate model type: fast");
    }
}