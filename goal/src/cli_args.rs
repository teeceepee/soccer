use std::path::PathBuf;
use argh::FromArgs;

fn default_config_path () -> PathBuf {
    PathBuf::from("./goal_config.toml")
}

#[derive(Debug, FromArgs)]
#[argh(description = "TODO cmd desc")]
pub struct CliArgs {
    #[argh(
        option,
        description = "config file path, default: './goal_config.toml'",
        default = "default_config_path()"
    )]
    pub config: PathBuf,
}
