use std::env;

#[derive(Clone)]
pub struct Config {
    pub app_url: String,
    pub app_port: u16,
    pub db_driver: String,
    pub db_host: String,
    pub db_port: u16,
    pub db_user: String,
    pub db_pass: String,
    pub db_name: String,
    pub jwt_secret: String,
    pub jwt_access_expire: i64,
    pub jwt_refresh_expire: i64,
    pub email_verification_required: bool,
    pub mail_host: String,
    pub mail_port: u16,
    pub mail_user: String,
    pub mail_pass: String,
    pub mail_from: String,
    pub mail_from_name: String,
    pub google_client_id: String,
    pub facebook_client_id: String,
    pub storage_path: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            app_url: env::var("APP_URL").unwrap_or_else(|_| "http://localhost:8000".into()),
            app_port: env::var("APP_PORT").unwrap_or_else(|_| "8000".into()).parse().unwrap_or(8000),
            db_driver: env::var("DB_DRIVER").unwrap_or_else(|_| "mysql".into()),
            db_host: env::var("DB_HOST").unwrap_or_else(|_| "127.0.0.1".into()),
            db_port: env::var("DB_PORT").unwrap_or_else(|_| "3306".into()).parse().unwrap_or(3306),
            db_user: env::var("DB_USER").unwrap_or_else(|_| "root".into()),
            db_pass: env::var("DB_PASS").unwrap_or_default(),
            db_name: env::var("DB_NAME").unwrap_or_else(|_| "starter_api".into()),
            jwt_secret: env::var("JWT_SECRET").unwrap_or_else(|_| "secret".into()),
            jwt_access_expire: env::var("JWT_ACCESS_EXPIRE").unwrap_or_else(|_| "15".into()).parse().unwrap_or(15),
            jwt_refresh_expire: env::var("JWT_REFRESH_EXPIRE").unwrap_or_else(|_| "10080".into()).parse().unwrap_or(10080),
            email_verification_required: env::var("EMAIL_VERIFICATION_REQUIRED")
                .unwrap_or_else(|_| "false".into())
                .to_lowercase() == "true",
            mail_host: env::var("MAIL_HOST").unwrap_or_default(),
            mail_port: env::var("MAIL_PORT").unwrap_or_else(|_| "587".into()).parse().unwrap_or(587),
            mail_user: env::var("MAIL_USER").unwrap_or_default(),
            mail_pass: env::var("MAIL_PASS").unwrap_or_default(),
            mail_from: env::var("MAIL_FROM").unwrap_or_else(|_| "no-reply@example.com".into()),
            mail_from_name: env::var("MAIL_FROM_NAME").unwrap_or_else(|_| "StarterAPI".into()),
            google_client_id: env::var("GOOGLE_CLIENT_ID").unwrap_or_default(),
            facebook_client_id: env::var("FACEBOOK_CLIENT_ID").unwrap_or_default(),
            storage_path: env::var("STORAGE_PATH").unwrap_or_else(|_| "./storage/photos".into()),
        }
    }

    pub fn database_url(&self) -> String {
        match self.db_driver.as_str() {
            "postgres" | "postgresql" => format!(
                "postgres://{}:{}@{}:{}/{}",
                self.db_user, self.db_pass, self.db_host, self.db_port, self.db_name
            ),
            "sqlite" => format!("sqlite://{}.db", self.db_name),
            _ => format!(
                "mysql://{}:{}@{}:{}/{}",
                self.db_user, self.db_pass, self.db_host, self.db_port, self.db_name
            ),
        }
    }
}
