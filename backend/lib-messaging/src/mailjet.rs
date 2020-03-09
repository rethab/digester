use reqwest::blocking::Client;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde::Serialize;

use super::Env;

pub struct SendgridCredentials {
    pub api_key: String,
}

pub fn send_email(cred: &SendgridCredentials, message: SendgridMessage) -> Result<(), String> {
    let result = Client::new()
        .post("https://api.sendgrid.com/v3/mail/send")
        .header(AUTHORIZATION, format!("Bearer {}", cred.api_key))
        .header(CONTENT_TYPE, "application/json")
        .json(&message)
        .send();
    match result {
        Ok(resp) if resp.status().is_success() => Ok(()),
        Ok(resp) => Err(format!("Sendgrid returned error: {:?}", resp)),
        Err(err) => Err(format!("Failed to send email: {:?}", err)),
    }
}

pub fn create_subject_for_list(env: &Env, list_name: &str) -> String {
    let mut subject = String::new();

    if *env != Env::Prod {
        subject.push_str(&format!("[{:?}] ", env));
    }

    subject.push_str("Digests from ");
    subject.push_str(list_name);
    subject
}

pub fn create_subject(env: &Env, subs: &[SendgridSubscription]) -> String {
    let mut subject = String::new();

    if *env != Env::Prod {
        subject.push_str(&format!("[{:?}] ", env));
    }

    subject.push_str("Digests from ");

    let max_len = 50; // mailjet says too long subjects could be suspicious/spammy
    let mut there_would_be_more = false;
    let mut added_one = false;

    for sub in subs {
        if subject.len() + sub.title.len() > max_len {
            there_would_be_more = true;
        } else {
            if added_one {
                subject.push_str(", ")
            }
            subject.push_str(&sub.title);
            added_one = true;
        }
    }

    if !added_one {
        if let Some(sub) = subs.iter().next() {
            subject.push_str(&sub.title);
        }
    } else if there_would_be_more {
        subject.push_str(" and more");
    }

    subject
}

#[derive(Serialize)]
pub struct SendgridMessage {
    from: SendgridFrom,
    template_id: String,
    personalizations: Vec<SendgridPersonalization>,
}

impl SendgridMessage {
    pub fn new(
        email: String,
        subject: String,
        subscriptions: Vec<SendgridSubscription>,
    ) -> SendgridMessage {
        SendgridMessage {
            from: SendgridFrom {
                email: "info@digester.app".into(),
                name: "Digester".into(),
            },
            template_id: "d-f83856fe31b94f05bff5b81679e56ef0".into(),
            personalizations: vec![SendgridPersonalization {
                to: vec![SendgridTo {
                    email: email.clone(),
                    name: email,
                }],
                dynamic_template_data: SendgridTemplateData {
                    subject,
                    subscriptions,
                },
            }],
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
struct SendgridPersonalization {
    to: Vec<SendgridTo>,
    dynamic_template_data: SendgridTemplateData,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn limit_subject_length_by_not_adding_very_long() {
        let sub1 = SendgridSubscription::new("kubernetes/kubernetes", Vec::new());
        let sub2 = SendgridSubscription::new("golang/tools", Vec::new());
        let sub3 = SendgridSubscription::new(
            "ohmylongorganisationname/ohmylongrepositoryname",
            Vec::new(),
        );
        let sub4 = SendgridSubscription::new("node/node", Vec::new());

        let actual = create_subject(&Env::Prod, &[sub1, sub2, sub3, sub4]);
        let expected = "Digests from kubernetes/kubernetes, golang/tools and more".to_owned();
        assert_eq!(expected, actual)
    }

    #[test]
    fn show_long_subject_if_only_one() {
        let sub1 = SendgridSubscription::new(
            "ohmyverylongorganisationname/ohmyverylongrepositoryname",
            Vec::new(),
        );

        let actual = create_subject(&Env::Prod, &[sub1]);
        let expected =
            "Digests from ohmyverylongorganisationname/ohmyverylongrepositoryname".to_owned();
        assert_eq!(expected, actual)
    }

    #[test]
    fn dont_show_and_more() {
        let sub1 = SendgridSubscription::new("kubernetes/kubernetes", Vec::new());
        let sub2 = SendgridSubscription::new("golang/tools", Vec::new());
        let actual = create_subject(&Env::Prod, &[sub1, sub2]);
        let expected = "Digests from kubernetes/kubernetes, golang/tools".to_owned();
        assert_eq!(expected, actual)
    }

    #[test]
    fn prepend_env_to_subject_in_dev_and_stg() {
        // dev
        let sub1 = SendgridSubscription::new("kubernetes/kubernetes", Vec::new());
        let actual = create_subject(&Env::Dev, &[sub1]);
        let expected = "[Dev] Digests from kubernetes/kubernetes".to_owned();
        assert_eq!(expected, actual);

        let sub1 = SendgridSubscription::new("kubernetes/kubernetes", Vec::new());
        let actual = create_subject(&Env::Stg, &[sub1]);
        let expected = "[Stg] Digests from kubernetes/kubernetes".to_owned();
        assert_eq!(expected, actual)
    }
}
