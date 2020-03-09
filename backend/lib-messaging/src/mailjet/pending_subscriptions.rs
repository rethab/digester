pub fn send_activation_email(email: &str, token: &str) -> Result<(), String> {
    println!("Token for email {}: {}", email, token);
    Ok(())
}
