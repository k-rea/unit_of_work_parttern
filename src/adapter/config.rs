pub struct AppConfig {
    db_url: String,
}

impl AppConfig {
    pub fn load() -> Self {
        Self {
            db_url: "postgres://postgres:postgres@localhost:5452/app".to_string(),
        }
    }

    pub fn db_url(self) -> String {
        self.db_url
    }
}
