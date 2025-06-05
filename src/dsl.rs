use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct DslConfig {
    pub name: String,
    pub target: String,
    pub method: String,
    pub concurrency: u64,
    pub duration: u64, 
}

impl DslConfig {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config = serde_yaml::from_str(&content)?;
        Ok(config)
    }
}
