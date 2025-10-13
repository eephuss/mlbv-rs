use directories::ProjectDirs;
use serde::Deserialize;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    // pub debug: Option<bool>,
    pub credentials: Credentials,
    // pub favorites: Option<Favorites>,
    // pub display: Option<Display>,
    // pub stream: Option<Stream>,
    // pub streamlink: Option<Streamlink>,
}

// #[derive(Debug, Deserialize)]
// pub struct Favorites {
//     pub teams: Option<Vec<String>>,
//     pub color: Option<String>,
//     pub critical_color: Option<String>,
// }

// #[derive(Debug, Deserialize)]
// pub struct Display {
//     pub scores: Option<bool>,
//     pub linescore: Option<bool>,
//     pub timeformat: Option<String>,
//     pub stats_limit: Option<u32>,
// }

// #[derive(Debug, Deserialize)]
// pub struct Stream {
//     pub resolution: Option<String>,
//     pub video_player: Option<String>,
//     // add other stream fields as needed
// }

// #[derive(Debug, Deserialize)]
// pub struct Streamlink {
//     // include streamlink-related keys as Option<T> here
//     // e.g. pub streamlink_highlights: Option<bool>,
// }

#[derive(Debug, Deserialize)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

impl AppConfig {
    fn prompt_credential(label: &str) -> io::Result<String> {
        print!("Enter mlb.tv {}: ", label);
        io::stdout().flush()?; // ensure prompt appears before waiting for input

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        Ok(input.trim().to_string())
    }

    fn generate_config(config_dir: &PathBuf, config_file: &PathBuf) -> anyhow::Result<()> {
        fs::create_dir_all(config_dir)?;

        let template_path = PathBuf::from("config_template.toml");
        if !template_path.exists() {
            anyhow::bail!("Template file not found at {}", template_path.display());
        }

        // Read the template into a string and prompt user for credentials
        let contents = fs::read_to_string(&template_path)?;
        let username = Self::prompt_credential("username")?;
        let password = Self::prompt_credential("password")?;

        // Parse template and update credentials while preserving comments/format
        let mut doc: toml_edit::DocumentMut = contents.parse()?;
        doc["credentials"]["username"] = toml_edit::value(username);
        doc["credentials"]["password"] = toml_edit::value(password);

        fs::write(config_file, doc.to_string())?;

        Ok(())
    }

    pub fn load() -> anyhow::Result<Self> {
        let proj_dirs =
            ProjectDirs::from("", "", "mlbv-rs").expect("Could not resolve config directory");
        let config_dir = proj_dirs.config_dir().to_path_buf();
        let config_file = config_dir.join("config.toml");

        // ensure the config exists (creates from template if needed)
        if !config_file.exists() {
            println!(
                "Config file not found, creating from template at {}",
                config_file.display()
            );
            Self::generate_config(&config_dir, &config_file)?;
        }

        println!("Loading config from: {}", config_file.display());

        let contents = fs::read_to_string(&config_file)?;
        let parsed: AppConfig = toml::from_str(&contents)?;

        Ok(parsed)
    }
}
