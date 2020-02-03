use reqwest::blocking::Client;
use reqwest::header::CONTENT_TYPE;
use serde::Serialize;

use super::Env;

pub struct MailjetCredentials {
    pub username: String,
    pub password: String,
}

pub fn send_email(cred: &MailjetCredentials, message: MailjetMessage) -> Result<(), String> {
    let messages = MailjetMessages::new(message);
    let result = Client::new()
        .post("https://api.mailjet.com/v3.1/send")
        .basic_auth(cred.username.clone(), Some(cred.password.clone()))
        .header(CONTENT_TYPE, "application/json")
        .json(&messages)
        .send();
    match result {
        Ok(resp) if resp.status().is_success() => Ok(()),
        Ok(resp) => Err(format!("Mailjet returned error: {:?}", resp)),
        Err(err) => Err(format!("Failed to send email: {:?}", err)),
    }
}

pub fn create_subject(env: &Env, subs: &[MailjetSubscription]) -> String {
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
struct MailjetMessages {
    #[serde(rename = "Messages")]
    messages: Vec<MailjetMessage>,
}

impl MailjetMessages {
    fn new(message: MailjetMessage) -> MailjetMessages {
        MailjetMessages {
            messages: vec![message],
        }
    }
}

#[derive(Serialize)]
pub struct MailjetMessage {
    #[serde(rename = "To")]
    to: Vec<MailjetTo>,
    #[serde(rename = "Subject")]
    subject: String,
    #[serde(rename = "TemplateID")]
    template_id: i32,
    #[serde(rename = "TemplateLanguage")]
    template_language: bool,
    #[serde(rename = "TemplateErrorReporting")]
    template_error_reporting: MailjetTemplateErrorReporting,
    #[serde(rename = "TemplateErrorDeliver")]
    template_error_deliver: bool,
    #[serde(rename = "Variables")]
    variables: MailjetVariables,
}

impl MailjetMessage {
    pub fn new(
        email: String,
        subject: String,
        subscriptions: Vec<MailjetSubscription>,
    ) -> MailjetMessage {
        MailjetMessage {
            to: vec![MailjetTo {
                email: email.clone(),
                name: email,
            }],
            subject,
            template_id: 1_153_883, // todo make flexible
            template_language: true,
            template_error_reporting: MailjetTemplateErrorReporting {
                email: "rethab@pm.me".into(),
                name: "Ret".into(),
            },
            template_error_deliver: true,
            // todo make flexible
            variables: MailjetVariables {
                update_subscriptions_url: "https://digester.app/subs".into(),
                add_subscription_url: "https://digester.app/subs".into(),
                subscriptions,
            },
        }
    }
}

#[derive(Serialize)]
struct MailjetTo {
    #[serde(rename = "Email")]
    email: String,
    #[serde(rename = "Name")]
    name: String,
}

#[derive(Serialize)]
struct MailjetTemplateErrorReporting {
    #[serde(rename = "Email")]
    email: String,
    #[serde(rename = "Name")]
    name: String,
}

#[derive(Serialize)]
struct MailjetVariables {
    update_subscriptions_url: String,
    add_subscription_url: String,
    subscriptions: Vec<MailjetSubscription>,
}

#[derive(Serialize)]
pub struct MailjetSubscription {
    title: String,
    updates: Vec<MailjetUpdate>,
}

impl MailjetSubscription {
    pub fn new(title: &str, updates: Vec<MailjetUpdate>) -> MailjetSubscription {
        MailjetSubscription {
            title: title.into(),
            updates,
        }
    }
}

#[derive(Serialize)]
pub struct MailjetUpdate {
    pub title: String,
    pub url: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn limit_subject_length_by_not_adding_very_long() {
        let sub1 = MailjetSubscription::new("kubernetes/kubernetes", Vec::new());
        let sub2 = MailjetSubscription::new("golang/tools", Vec::new());
        let sub3 = MailjetSubscription::new(
            "ohmylongorganisationname/ohmylongrepositoryname",
            Vec::new(),
        );
        let sub4 = MailjetSubscription::new("node/node", Vec::new());

        let actual = create_subject(&Env::Prod, &[sub1, sub2, sub3, sub4]);
        let expected = "Digests from kubernetes/kubernetes, golang/tools and more".to_owned();
        assert_eq!(expected, actual)
    }

    #[test]
    fn show_long_subject_if_only_one() {
        let sub1 = MailjetSubscription::new(
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
        let sub1 = MailjetSubscription::new("kubernetes/kubernetes", Vec::new());
        let sub2 = MailjetSubscription::new("golang/tools", Vec::new());
        let actual = create_subject(&Env::Prod, &[sub1, sub2]);
        let expected = "Digests from kubernetes/kubernetes, golang/tools".to_owned();
        assert_eq!(expected, actual)
    }

    #[test]
    fn prepend_env_to_subject_in_dev_and_stg() {
        // dev
        let sub1 = MailjetSubscription::new("kubernetes/kubernetes", Vec::new());
        let actual = create_subject(&Env::Dev, &[sub1]);
        let expected = "[Dev] Digests from kubernetes/kubernetes".to_owned();
        assert_eq!(expected, actual);

        let sub1 = MailjetSubscription::new("kubernetes/kubernetes", Vec::new());
        let actual = create_subject(&Env::Stg, &[sub1]);
        let expected = "[Stg] Digests from kubernetes/kubernetes".to_owned();
        assert_eq!(expected, actual)
    }
}
