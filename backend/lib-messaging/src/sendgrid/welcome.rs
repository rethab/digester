use super::*;

pub fn send_welcome_email(credentials: &SendgridCredentials, recipient: &str) {
    let message = SendgridMessage::new_welcome_message(recipient);
    let request = SendgridRequest::new_welcome_request(message);
    if let Err(errmsg) = send_email(credentials, request) {
        eprintln!("Failed to send welcome e-mail: {}", errmsg);
    }
}
