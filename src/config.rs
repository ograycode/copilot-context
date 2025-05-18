use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ContextConfig {
    pub version: u8,
    pub dest: Option<String>,
    pub sources: Vec<Source>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Source {
    Repo {
        name: String,
        repo: String,
        branch: Option<String>,
        dest: String,
        rm: Option<Vec<String>>,
    },
    Url {
        name: String,
        url: String,
        dest: String,
        rm: Option<Vec<String>>,
    },
    Path {
        name: String,
        path: String,
        dest: String,
        rm: Option<Vec<String>>,
    },
}

pub fn load_config(path: &str) -> Result<ContextConfig, Box<dyn std::error::Error>> {
    let f = std::fs::read_to_string(path)?;
    let config: ContextConfig = toml::from_str(&f)?;
    Ok(config)
}
