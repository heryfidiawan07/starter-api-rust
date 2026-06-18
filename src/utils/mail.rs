use anyhow::Result;
use lettre::{
    message::header::ContentType,
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};

use crate::config::Config;

pub async fn send_verification_email(cfg: &Config, to_email: &str, to_name: &str, token: &str) -> Result<()> {
    let link = format!("{}/api/v1/auth/verify-email?token={}", cfg.app_url, token);
    let body = format!(
        "Hi {},\n\nPlease verify your email by clicking:\n{}\n\nThis link expires in 24 hours.",
        to_name, link
    );
    send_email(cfg, to_email, "Verify Your Email", &body).await
}

pub async fn send_password_reset_email(cfg: &Config, to_email: &str, to_name: &str, token: &str) -> Result<()> {
    let link = format!("{}/api/v1/auth/reset-password?token={}", cfg.app_url, token);
    let body = format!(
        "Hi {},\n\nReset your password by clicking:\n{}\n\nThis link expires in 1 hour.",
        to_name, link
    );
    send_email(cfg, to_email, "Reset Your Password", &body).await
}

async fn send_email(cfg: &Config, to_email: &str, subject: &str, body: &str) -> Result<()> {
    if cfg.mail_host.is_empty() {
        return Ok(());
    }
    let from = format!("{} <{}>", cfg.mail_from_name, cfg.mail_from);
    let email = Message::builder()
        .from(from.parse()?)
        .to(to_email.parse()?)
        .subject(subject)
        .header(ContentType::TEXT_PLAIN)
        .body(body.to_string())?;

    let creds = Credentials::new(cfg.mail_user.clone(), cfg.mail_pass.clone());
    let mailer: AsyncSmtpTransport<Tokio1Executor> =
        AsyncSmtpTransport::<Tokio1Executor>::relay(&cfg.mail_host)?
            .port(cfg.mail_port)
            .credentials(creds)
            .build();

    mailer.send(email).await?;
    Ok(())
}
