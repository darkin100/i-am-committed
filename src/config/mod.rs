use std::{env, fs, path::PathBuf};

pub struct Config {
    config_dir: PathBuf,
}

impl Config {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config_dir = Self::get_config_dir()?;

        // Create config directory if it doesn't exist
        fs::create_dir_all(&config_dir)?;

        Ok(Config { config_dir })
    }

    /// Get the platform-appropriate config directory
    fn get_config_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
        // Check for XDG_CONFIG_HOME first (Linux standard)
        if let Ok(xdg_config) = env::var("XDG_CONFIG_HOME") {
            return Ok(PathBuf::from(xdg_config).join("iamcommitted"));
        }

        // Otherwise use platform-specific defaults
        let home = env::var("HOME")
            .or_else(|_| env::var("USERPROFILE")) // Windows fallback
            .map_err(|_| "Failed to get home directory")?;

        let config_dir = if cfg!(target_os = "macos") {
            PathBuf::from(&home).join("Library/Application Support/iamcommitted")
        } else if cfg!(target_os = "windows") {
            PathBuf::from(&home).join("AppData/Roaming/iamcommitted")
        } else {
            // Linux/Unix default
            PathBuf::from(&home).join(".config/iamcommitted")
        };

        Ok(config_dir)
    }

    /// Get the path to the prompts configuration file
    pub fn prompts_path(&self) -> PathBuf {
        self.config_dir.join("prompts.md")
    }

    /// Get the platform-appropriate log directory
    pub fn get_log_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
        // Check for XDG_DATA_HOME first (Linux standard)
        if let Ok(xdg_data) = env::var("XDG_DATA_HOME") {
            return Ok(PathBuf::from(xdg_data).join("iamcommitted/logs"));
        }

        let home = env::var("HOME")
            .or_else(|_| env::var("USERPROFILE")) // Windows fallback
            .map_err(|_| "Failed to get home directory")?;

        let log_dir = if cfg!(target_os = "macos") {
            PathBuf::from(&home).join("Library/Logs/iamcommitted")
        } else if cfg!(target_os = "windows") {
            PathBuf::from(&home).join("AppData/Local/iamcommitted/logs")
        } else {
            // Linux/Unix default - use XDG_STATE_HOME if available
            if let Ok(xdg_state) = env::var("XDG_STATE_HOME") {
                PathBuf::from(xdg_state).join("iamcommitted/logs")
            } else {
                PathBuf::from(&home).join(".local/state/iamcommitted/logs")
            }
        };

        Ok(log_dir)
    }

    /// Load prompts from config file, creating default if it doesn't exist
    pub fn load_prompts(&self) -> Result<String, Box<dyn std::error::Error>> {
        let prompts_path = self.prompts_path();

        if !prompts_path.exists() {
            // Copy default prompts to config directory
            self.create_default_prompts()?;
        }

        fs::read_to_string(&prompts_path)
            .map_err(|e| format!("Failed to read prompts from {:?}: {}", prompts_path, e).into())
    }

    /// Create default prompts file in config directory
    fn create_default_prompts(&self) -> Result<(), Box<dyn std::error::Error>> {
        let default_prompts = include_str!("prompts.md");
        let prompts_path = self.prompts_path();

        fs::write(&prompts_path, default_prompts).map_err(|e| -> Box<dyn std::error::Error> {
            format!(
                "Failed to write default prompts to {:?}: {}",
                prompts_path, e
            )
            .into()
        })?;

        println!(
            "Created default prompts configuration at: {:?}",
            prompts_path
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_config_dir_creation() {
        // Set a temporary XDG_CONFIG_HOME for testing
        let temp_dir = TempDir::new().unwrap();
        env::set_var("XDG_CONFIG_HOME", temp_dir.path());

        let config = Config::new().unwrap();
        assert!(config.config_dir.exists());

        // Clean up
        env::remove_var("XDG_CONFIG_HOME");
    }

    #[test]
    fn test_prompts_path() {
        let temp_dir = TempDir::new().unwrap();
        env::set_var("XDG_CONFIG_HOME", temp_dir.path());

        let config = Config::new().unwrap();
        let prompts_path = config.prompts_path();
        assert_eq!(prompts_path.file_name().unwrap(), "prompts.md");

        // Clean up
        env::remove_var("XDG_CONFIG_HOME");
    }
}
