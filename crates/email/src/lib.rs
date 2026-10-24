use lettre::{
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor, message::MultiPart,
    transport::smtp::authentication::Credentials,
};

use gilvave_settings::settings;

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

    pub async fn send_email_validate(
        &self,
        to_email: &str,
        token: String,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let confirmation_link =
            format!("http://127.0.0.1:3000/users/auth/register/confirm?token={token}");

        let html_body = format!(
            r#"<h1>Добро пожаловать в Gilvave!</h1>
           <p>Пожалуйста, подтвердите ваш email для завершения регистрации, перейдя по ссылке:</p>
           <p><a href="{confirmation_link}">Подтвердить email</a></p>"#
        );
        let text_body = format!(
            "Добро пожаловать в Gilvave!\n\nПодтвердите ваш email для завершения регистрации: {confirmation_link}"
        );

        let email = Message::builder()
            .from(format!("Gilvave <{}>", settings!().smtp_login).parse()?)
            .to(format!("New User <{to_email}>").parse()?)
            .subject("Подтверждение регистрации")
            .multipart(MultiPart::alternative_plain_html(text_body, html_body))?;

        self.mailer.send(email).await?;

        Ok(())
    }
}
