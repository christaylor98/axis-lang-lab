use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct LanglabConfig {
    pub active_profile: String,
    pub registry_paths: Vec<PathBuf>,
}

impl LanglabConfig {
    pub fn default() -> Self {
        LanglabConfig {
            active_profile: "default".into(),
            registry_paths: Vec::new(),
        }
    }

    pub fn load() -> Result<Self, std::io::Error> {
        let path = PathBuf::from("config/langlab.toml");
        let s = fs::read_to_string(&path)?;
        Ok(Self::from_toml_str(&s))
    }

    fn from_toml_str(s: &str) -> Self {
        // Minimal parser supporting only the exact shapes required
        let mut active_profile = "default".to_string();
        let mut registry_paths: Vec<PathBuf> = Vec::new();

        let mut in_registries = false;
        for line in s.lines() {
            let l = line.trim();
            if l.is_empty() || l.starts_with('#') {
                continue;
            }
            if l.starts_with("active_profile") {
                if let Some(eq) = l.find('=') {
                    let v = l[eq + 1..].trim().trim_matches('"').to_string();
                    active_profile = v;
                }
            } else if l.starts_with("registries") {
                if l.contains('[') && l.contains(']') {
                    // single-line array
                    if let Some(start) = l.find('[') {
                        if let Some(end) = l.find(']') {
                            let inner = &l[start + 1..end];
                            for item in inner.split(',') {
                                let it = item.trim().trim_matches('"').to_string();
                                if !it.is_empty() {
                                    registry_paths.push(PathBuf::from(it));
                                }
                            }
                        }
                    }
                } else if l.contains('[') {
                    in_registries = true;
                }
            } else if in_registries {
                if l.contains(']') {
                    in_registries = false;
                    continue;
                }
                // expect lines like "  \"path\"," or "\"path\",""
                let item = l.trim().trim_end_matches(',').trim_matches('"');
                if !item.is_empty() {
                    registry_paths.push(PathBuf::from(item));
                }
            }
        }

        LanglabConfig {
            active_profile,
            registry_paths,
        }
    }
}
