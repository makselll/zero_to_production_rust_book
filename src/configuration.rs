use secrecy::{ExposeSecret, SecretString};
use config::Config;

#[derive(serde::Deserialize, Debug)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application: ApplicationSettings,
    pub jaeger: JaegerSettings,
}

#[derive(serde::Deserialize, Debug)]
pub struct JaegerSettings {
    pub address: String,
    pub port: u16,
}

#[derive(serde::Deserialize, Debug)]
pub struct ApplicationSettings {
    pub address: String,
    pub port: u16,
}

#[derive(serde::Deserialize, Debug)]
pub struct DatabaseSettings { pub username: String,
    pub password: SecretString,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}


impl DatabaseSettings {
    pub fn connection_string(&self) -> SecretString { 
        SecretString::from(format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password.expose_secret(), self.host, self.port, self.database_name
        ))
    }

    pub fn connection_string_without_db(&self) -> SecretString {
        SecretString::from(format!(
            "postgres://{}:{}@{}:{}",
            self.username, self.password.expose_secret(), self.host, self.port
        ))
    }
}


pub fn get_configuration() -> Result<Settings, config::ConfigError> { // Initialise our configuration reader

    let base_path = std::env::current_dir().expect("Failed to determine the current directory");
    let configuration_directory = base_path.join("configuration");
    
    // Detect the running environment.
    // Default to `local` if unspecified.
    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "local".into())
        .try_into()
        .expect("Failed to parse APP_ENVIRONMENT.");
    
    let settings = Config::builder()
        .add_source(config::File::from(configuration_directory.join("base")).required(true))
        .add_source(config::File::from(configuration_directory.join(environment.as_str())).required(true))
        .build()?;
    
    settings.try_deserialize()
}

/// The possible runtime environment for our application.
pub enum Environment {
    Local,
    Production,
}
impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local", Environment::Production => "production",
        } }
}
impl TryFrom<String> for Environment {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> { 
        match s.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            other => Err(format!("{} is not a supported environment. Use either `local` or `production`.", other )),
    } }
}