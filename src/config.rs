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
        sparse: Option<Vec<String>>,
        branch: Option<String>,
        dest: String,
    },
    Url {
        name: String,
        url: String,
        dest: String,
    },
    Path {
        name: String,
        path: String,
        dest: String,
    },
}

pub fn load_config(path: &str) -> Result<ContextConfig, Box<dyn std::error::Error>> {
    let f = std::fs::File::open(path)?;
    let config: ContextConfig = serde_yaml::from_reader(f)?;
    Ok(config)
}
