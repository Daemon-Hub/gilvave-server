use lettre::{
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor, message::MultiPart,
    transport::smtp::authentication::Credentials,
};

use gilvave_settings::{constants::build_confirm_email_html, settings};

pub struct EmailService {
    mailer: AsyncSmtpTransport<Tokio1Executor>,
}

impl EmailService {
    pub fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let creds = Credentials::new(
            settings!().smtp_login.to_string(),
            settings!().smtp_secret.to_string(),
        );

        Ok(Self {
            mailer: AsyncSmtpTransport::<Tokio1Executor>::relay("smtp.mail.ru")?
                .credentials(creds)
                .build(),
        })
    }

    pub async fn send_email_validate_msg(
        &self,
        to_email: &str,
        token: String,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let confirmation_link =
            format!("http://127.0.0.1:3000/users/auth/register/confirm?token={token}");

        let html_body = build_confirm_email_html(&confirmation_link);
        let text_body = format!(
            "Добро пожаловать в Gilvave!\n\n\
            Спасибо за регистрацию. Для подтверждения адреса электронной почты перейдите по ссылке:\n\
            {confirmation_link}\n\n\
            Ссылка активна в течение 20 минут.\n\
            Если вы не создавали аккаунт в Gilvave, просто проигнорируйте это письмо."
        );

        let email = Message::builder()
            .from(format!("Gilvave Team <{}>", settings!().smtp_login).parse()?)
            .to(format!("New User <{to_email}>").parse()?)
            .subject("Подтверждение регистрации")
            .multipart(MultiPart::alternative_plain_html(text_body, html_body))?;

        self.mailer.send(email).await?;

        Ok(())
    }
}
