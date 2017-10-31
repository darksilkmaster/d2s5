#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub general: GeneralConfig,
}

#[derive(Deserialize, Debug, Clone)]
pub struct GeneralConfig {
    pub listen_addr: String,
}
