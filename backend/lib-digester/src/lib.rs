use chrono::{DateTime, Datelike, Duration, TimeZone, Timelike, Utc, Weekday};
use chrono_tz::Tz;
use lib_db as db;
use lib_db::{Day, Digest, Frequency, InsertDigest, Subscription, User};
use reqwest::header::CONTENT_TYPE;
use reqwest::Client;
use serde::Serialize;

pub struct App<'a> {
    db_conn: &'a db::Connection,
    mailjet: MailjetCredentials,
}

pub struct MailjetCredentials {
    pub username: String,
    pub password: String,
}

impl App<'_> {
    pub fn new(db_conn: &db::Connection, mailjet: MailjetCredentials) -> App {
        App { db_conn, mailjet }
    }

    pub fn run(&self) -> Result<(), String> {
        let subscriptions = db::subscriptions_find_without_due_digest(&self.db_conn)?;
        println!(
            "{} subscriptions need a digest (unsent)",
            subscriptions.len()
        );

        for subscription in subscriptions {
            self.insert_next_digest(&subscription)?;
        }

        let users = db::digests_find_users_with_due(&self.db_conn)?;
        println!("Found {} users with due digests", users.len());

        for user in users {
            let d_and_s = db::digests_find_due_for_user(&self.db_conn, &user)?;
            match self.send_digest(&user, &d_and_s) {
                Ok(_) => {
                    for (digest, _) in d_and_s {
                        db::digests_set_sent(&self.db_conn, &digest)?;
                    }
                }
                Err(err) => eprintln!("Failed to send digest: {:?}", err),
            }
        }

        Ok(())
    }

    fn insert_next_digest(&self, subscription: &Subscription) -> Result<(), String> {
        let user = db::users_find_by_id0(&self.db_conn, subscription.user_id)?;
        let timezone = user.timezone.map(|tz| tz.0).unwrap_or(Tz::UTC);

        let now_in_tz: DateTime<Tz> = timezone.from_utc_datetime(&Utc::now().naive_utc());
        let due_in_tz = next_due_date_for_subscription(subscription, now_in_tz);
        let due_date = due_in_tz.with_timezone(&Utc);

        let digest = InsertDigest {
            subscription_id: subscription.id,
            due: due_date,
        };
        match db::digests_insert(&self.db_conn, &digest) {
            Ok(()) => Ok(()),
            Err(db::InsertError::Unknown) => {
                Err(format!("Failed to insert new digest: {:?}", digest))
            }
            Err(db::InsertError::Duplicate) => {
                println!(
                    "Digest seems to have been inserted in the meantime.. {:?}",
                    digest
                );
                Ok(())
            }
        }
    }

    fn send_digest(&self, user: &User, d_and_s: &[(Digest, Subscription)]) -> Result<(), String> {
        let mut mailjet_subscriptions = Vec::with_capacity(d_and_s.len());
        for (digest, subscription) in d_and_s {
            // we send new updates since the last digest or since
            // when the subscription was created if this is the first digest
            let updates_since = db::digests_find_previous(&self.db_conn, &digest)?
                .and_then(|d| d.sent)
                .unwrap_or(subscription.inserted);
            let updates = db::updates_find_new(&self.db_conn, &subscription, updates_since)?;

            if !updates.is_empty() {
                let channel = db::channels_find_by_id(&self.db_conn, subscription.channel_id)?;
                let mailjet_updates = updates
                    .into_iter()
                    .map(|u| MailjetUpdate {
                        title: u.title,
                        url: u.url,
                    })
                    .collect();
                mailjet_subscriptions
                    .push(MailjetSubscription::new(&channel.name, mailjet_updates));
            }
        }

        if mailjet_subscriptions.is_empty() {
            // happens if user has a due digest, but we have no updates..
            println!(
                "No updates to send for User {} in any of their digests",
                user.id
            );
            Ok(())
        } else {
            let recipient = d_and_s[0].1.email.clone();
            let message = MailjetMessage::new(recipient, mailjet_subscriptions);
            self.send_email(message)
        }
    }

    fn send_email(&self, message: MailjetMessage) -> Result<(), String> {
        let messages = MailjetMessages::new(message);
        let result = Client::new()
            .post("https://api.mailjet.com/v3.1/send")
            .basic_auth(
                self.mailjet.username.clone(),
                Some(self.mailjet.password.clone()),
            )
            .header(CONTENT_TYPE, "application/json")
            .json(&messages)
            .send();
        match result {
            Ok(resp) if resp.status().is_success() => Ok(()),
            Ok(resp) => Err(format!("Mailjet returned error: {:?}", resp)),
            Err(err) => Err(format!("Failed to send email: {:?}", err)),
        }
    }
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
struct MailjetMessage {
    #[serde(rename = "To")]
    to: Vec<MailjetTo>,
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
    fn new(email: String, subscriptions: Vec<MailjetSubscription>) -> MailjetMessage {
        MailjetMessage {
            to: vec![MailjetTo {
                email: email,
                name: "Anonymous Panter".into(),
            }],
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
struct MailjetSubscription {
    title: String,
    updates: Vec<MailjetUpdate>,
}

impl MailjetSubscription {
    fn new(title: &str, updates: Vec<MailjetUpdate>) -> MailjetSubscription {
        MailjetSubscription {
            title: title.into(),
            updates,
        }
    }
}

#[derive(Serialize)]
struct MailjetUpdate {
    title: String,
    url: String,
}

fn next_due_date_for_subscription(subscription: &Subscription, now: DateTime<Tz>) -> DateTime<Tz> {
    match subscription.frequency {
        Frequency::Daily => {
            let due_time = subscription.time;
            let is_due_today = due_time.hour() > now.hour()
                || (due_time.hour() == now.hour() && due_time.minute() > now.minute());
            let due = now
                .with_hour(due_time.hour())
                .unwrap()
                .with_minute(due_time.minute())
                .unwrap();
            if is_due_today {
                due
            } else {
                due + Duration::days(1)
            }
        }
        Frequency::Weekly => {
            let Subscription {
                day: due_day,
                time: due_time,
                ..
            } = subscription;

            // into doesn't seem to work on references? need to make this nicer..
            let due_weekday: Weekday = (*due_day.as_ref().unwrap()).clone().into();

            let is_due_today = due_weekday == now.weekday() && due_time.hour() > now.hour()
                || (due_time.hour() == now.hour() && due_time.minute() > now.minute());

            let due = now
                .with_hour(due_time.hour())
                .unwrap()
                .with_minute(due_time.minute())
                .unwrap();
            if is_due_today {
                due
            } else {
                let today_idx = now.weekday().num_days_from_monday() as i64;
                let due_idx = due_weekday.num_days_from_monday() as i64;
                let days_to_day = if due_idx > today_idx {
                    due_idx - today_idx
                } else {
                    7 - today_idx + due_idx
                };
                due + Duration::days(days_to_day)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::naive::NaiveTime;
    use chrono_tz::Europe::Zurich;

    #[test]
    fn digester_due_daily_tomorrow() {
        let subscription = mk_daily(9, 0);
        let now = today(10, 0);
        let due = next_due_date_for_subscription(&subscription, now);
        assert_eq!(tomorrow(9, 0), due)
    }

    #[test]
    fn digester_due_daily_tomorrow_minute() {
        let subscription = mk_daily(9, 0);
        let now = today(9, 15);
        let due = next_due_date_for_subscription(&subscription, now);
        assert_eq!(tomorrow(9, 0), due)
    }

    #[test]
    fn digester_due_daily_today() {
        let subscription = mk_daily(9, 0);
        let now = today(8, 0);
        let due = next_due_date_for_subscription(&subscription, now);
        assert_eq!(today(9, 0), due)
    }

    #[test]
    fn digester_due_daily_today_minute() {
        let subscription = mk_daily(9, 0);
        let now = today(8, 15);
        let due = next_due_date_for_subscription(&subscription, now);
        assert_eq!(today(9, 0), due)
    }

    #[test]
    fn digester_due_weekly_today_hour() {
        let subscription = mk_weekly(Day::Mon, 9, 0);
        let now = day(Weekday::Mon, 8, 0);
        let due = next_due_date_for_subscription(&subscription, now);
        assert_eq!(day(Weekday::Mon, 9, 0), due)
    }

    #[test]
    fn digester_due_weekly_today_minute() {
        let subscription = mk_weekly(Day::Mon, 9, 0);
        let now = day(Weekday::Mon, 8, 15);
        let due = next_due_date_for_subscription(&subscription, now);
        assert_eq!(day(Weekday::Mon, 9, 0), due)
    }

    #[test]
    fn digester_due_weekly_tomorrow_day() {
        let subscription = mk_weekly(Day::Tue, 9, 0);
        let now = day(Weekday::Mon, 9, 0);
        let due = next_due_date_for_subscription(&subscription, now);
        assert_eq!(day(Weekday::Tue, 9, 0), due)
    }

    #[test]
    fn digester_due_weekly_tomorrow_hour() {
        let subscription = mk_weekly(Day::Tue, 10, 0);
        let now = day(Weekday::Mon, 9, 0);
        let due = next_due_date_for_subscription(&subscription, now);
        assert_eq!(day(Weekday::Tue, 10, 0), due)
    }

    #[test]
    fn digester_due_weekly_tomorrow_earlier_hour() {
        let subscription = mk_weekly(Day::Tue, 10, 0);
        let now = day(Weekday::Mon, 11, 0);
        let due = next_due_date_for_subscription(&subscription, now);
        assert_eq!(day(Weekday::Tue, 10, 0), due)
    }

    #[test]
    fn digester_due_weekly_next_week() {
        let subscription = mk_weekly(Day::Wed, 10, 0);
        let now = day(Weekday::Thu, 9, 0);
        let due = next_due_date_for_subscription(&subscription, now);
        assert_eq!(day(Weekday::Wed, 10, 0) + Duration::weeks(1), due)
    }

    fn mk_daily(hour: u32, minute: u32) -> Subscription {
        Subscription {
            id: 1,
            email: "foo@bar.ch".into(),
            channel_id: 1,
            user_id: 1,
            frequency: Frequency::Daily,
            day: None,
            time: NaiveTime::from_hms(hour, minute, 0),
            inserted: Utc::now(),
        }
    }

    fn mk_weekly(day: Day, hour: u32, minute: u32) -> Subscription {
        Subscription {
            id: 1,
            email: "foo@bar.ch".into(),
            channel_id: 1,
            user_id: 1,
            frequency: Frequency::Weekly,
            day: Some(day),
            time: NaiveTime::from_hms(hour, minute, 0),
            inserted: Utc::now(),
        }
    }

    fn day(weekday: Weekday, hour: u32, minute: u32) -> DateTime<Tz> {
        // 2nd december is a monday, so if we are passed monday, then 'day' will be 2
        let day = weekday.number_from_monday() + 1;
        Zurich.ymd(2019, 12, day).and_hms(hour, minute, 0)
    }

    fn today(hour: u32, minute: u32) -> DateTime<Tz> {
        Zurich.ymd(1990, 5, 6).and_hms(hour, minute, 0)
    }

    fn tomorrow(hour: u32, minute: u32) -> DateTime<Tz> {
        today(hour, minute) + Duration::days(1)
    }
}
