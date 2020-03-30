use reqwest::blocking::Client;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde::Serialize;

pub mod digests;
pub mod pending_subscriptions;

pub struct SendgridCredentials {
    pub api_key: String,
}

pub fn send_email(
    cred: &SendgridCredentials,
    messages: Vec<SendgridMessage>,
) -> Result<(), String> {
    let request = SendgridRequest::new(messages);
    let result = Client::new()
        .post("https://api.sendgrid.com/v3/mail/send")
        .header(AUTHORIZATION, format!("Bearer {}", cred.api_key))
        .header(CONTENT_TYPE, "application/json")
        .json(&request)
        .send();
    match result {
        Ok(resp) if resp.status().is_success() => Ok(()),
        Ok(resp) => Err(format!("Sendgrid returned error: {:?}", resp)),
        Err(err) => Err(format!("Failed to send email: {:?}", err)),
    }
}

#[derive(Serialize)]
pub struct SendgridRequest {
    from: SendgridFrom,
    template_id: String,
    personalizations: Vec<SendgridMessage>,
}

impl SendgridRequest {
    pub fn new(messages: Vec<SendgridMessage>) -> SendgridRequest {
        SendgridRequest {
            from: SendgridFrom {
                email: "info@digester.app".into(),
                name: "Digester".into(),
            },
            template_id: "d-f83856fe31b94f05bff5b81679e56ef0".into(),
            personalizations: messages,
        }
    }
}

#[derive(Serialize)]
struct SendgridFrom {
    email: String,
    name: String,
}

#[derive(Serialize)]
struct SendgridTo {
    email: String,
    name: String,
}

#[derive(Serialize)]
pub struct SendgridMessage {
    to: Vec<SendgridTo>,
    dynamic_template_data: SendgridTemplateData,
}

impl SendgridMessage {
    pub fn new(
        recipient_email: String,
        subject: String,
        subscriptions: Vec<SendgridSubscription>,
    ) -> SendgridMessage {
        SendgridMessage {
            to: vec![SendgridTo {
                email: recipient_email.clone(),
                name: recipient_email,
            }],
            dynamic_template_data: SendgridTemplateData {
                subject,
                subscriptions,
            },
        }
    }
}

#[derive(Serialize)]
struct SendgridTemplateData {
    subject: String,
    subscriptions: Vec<SendgridSubscription>,
}

#[derive(Serialize)]
pub struct SendgridSubscription {
    title: String,
    updates: Vec<SendgridUpdate>,
}

impl SendgridSubscription {
    pub fn new(title: &str, updates: Vec<SendgridUpdate>) -> SendgridSubscription {
        SendgridSubscription {
            title: title.into(),
            updates,
        }
    }
}

#[derive(Serialize)]
pub struct SendgridUpdate {
    pub title: String,
    pub url: String,
}