use reqwest::blocking::Client;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde::Serialize;

pub mod digests;
pub mod pending_subscriptions;
pub mod welcome;

pub struct NEVec<T> {
    head: T,
    tail: Vec<T>,
}

impl<T> NEVec<T> {
    pub fn from_vec(mut vs: Vec<T>) -> Option<NEVec<T>> {
        if vs.is_empty() {
            None
        } else {
            Some(NEVec {
                head: vs.remove(0),
                tail: vs,
            })
        }
    }
}

impl<T> Into<Vec<T>> for NEVec<T> {
    fn into(mut self) -> Vec<T> {
        let mut tmp = Vec::with_capacity(self.tail.len() + 1);
        tmp.push(self.head);
        tmp.append(&mut self.tail);
        tmp
    }
}

pub struct SendgridCredentials {
    pub api_key: String,
}

pub fn send_email(cred: &SendgridCredentials, request: SendgridRequest) -> Result<(), String> {
    let result = Client::new()
        .post("https://api.sendgrid.com/v3/mail/send")
        .header(AUTHORIZATION, format!("Bearer {}", cred.api_key))
        .header(CONTENT_TYPE, "application/json")
        .json(&request)
        .send();
    match result {
        Ok(resp) if resp.status().is_success() => Ok(()),
        Ok(resp) => Err(format!(
            "Sendgrid returned status {}: {:?}",
            resp.status(),
            resp.text().unwrap_or_else(|_| "".to_owned())
        )),
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
    pub fn new_digests_request(messages: NEVec<SendgridMessage>) -> SendgridRequest {
        SendgridRequest {
            from: SendgridFrom {
                email: "info@digester.app".into(),
                name: "Digester".into(),
            },
            template_id: "d-f83856fe31b94f05bff5b81679e56ef0".into(),
            personalizations: messages.into(),
        }
    }

    pub fn new_welcome_request(message: SendgridMessage) -> SendgridRequest {
        SendgridRequest {
            from: SendgridFrom {
                email: "info@digester.app".into(),
                name: "Digester".into(),
            },
            template_id: "d-4ecfbf0d40ca4116b5d9486369df9f92".into(),
            personalizations: vec![message],
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
    dynamic_template_data: Option<SendgridTemplateData>,
}

impl SendgridMessage {
    pub fn new_digests_message(
        recipient_email: String,
        subject: String,
        subscriptions: Vec<SendgridSubscription>,
    ) -> SendgridMessage {
        SendgridMessage {
            to: vec![SendgridTo {
                email: recipient_email.clone(),
                name: recipient_email,
            }],
            dynamic_template_data: Some(SendgridTemplateData {
                subject,
                subscriptions,
            }),
        }
    }

    pub fn new_welcome_message(recipient_email: &str) -> SendgridMessage {
        SendgridMessage {
            to: vec![SendgridTo {
                email: recipient_email.to_owned(),
                name: recipient_email.to_owned(),
            }],
            dynamic_template_data: None,
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
