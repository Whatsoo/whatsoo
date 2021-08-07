extern crate lettre;
extern crate lettre_email;
extern crate mime;

use lettre_email::Email;
use lettre::smtp::authentication::Credentials;
use lettre::{SmtpClient, Transport};

use anyhow::Result;
use argon2::{password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString}, Argon2, Version};
use rand_core::OsRng;

pub fn send_email() {
    let email_receiver = "YOUR_TARGET_EMAIL";
    let mine_email = "YOUR_GMAIL_ADDRESS";
    let smtp_server = "smtp.gmail.com";
    let password = "YOUR_GMAIL_APPLICATION_PASSWORD"; //需要生成应用专用密码

    let email = Email::builder()
        .to(email_receiver)
        .from(mine_email)
        .subject("subject")
        .html("<h1>Hi there</h1>")
        .text("Message send by lettre Rust")
        .build()
        .unwrap();

    let creds = Credentials::new(
        mine_email.to_string(),
        password.to_string(),
    );

    // Open connection to Gmail
    let mut mailer = SmtpClient::new_simple(smtp_server)
        .unwrap()
        .credentials(creds)
        .transport();

    // Send the email
    let result = mailer.send(email.into());

    if result.is_ok() {
        println!("Email sent");
    } else {
        println!("Could not send email: {:?}", result);
    }

    print!("{:?}", result);
    mailer.close();
}

pub fn encode_pwd(pwd: &str, salt: SaltString) -> String {
    let argon2 = Argon2::new(None, 3, 1024, 1, Version::V0x13).unwrap_or(Argon2::default());
    // TODO 错误处理，加密失败处理方式
    argon2.hash_password_simple(pwd.as_bytes(), salt.as_ref()).unwrap().to_string()
}