use chrono::{DateTime, Datelike, Duration, TimeZone, Timelike, Utc, Weekday};
use lettre::smtp::authentication::{Credentials, Mechanism};
use lettre::{ClientSecurity, SmtpClient, Transport};
use lettre_email::Email;
use lib_db as db;
use lib_db::{Day, Digest, Frequency, InsertDigest, Subscription};

pub struct App<'a> {
    db_conn: &'a db::Connection,
}

impl App<'_> {
    pub fn new(db_conn: &db::Connection) -> App {
        App { db_conn }
    }

    pub fn run(&self) -> Result<(), String> {
        let subscriptions = db::subscriptions_find_without_due_digest(&self.db_conn)?;
        println!(
            "Found {} subscriptions without due digest",
            subscriptions.len()
        );

        for subscription in subscriptions {
            self.insert_next_digest(&subscription)?;
        }

        let digests = db::digests_find_due(&self.db_conn)?;
        println!("Found {} digests to send", digests.len());

        for digest in digests {
            match self.send_digest(&digest) {
                Ok(_) => db::digests_set_sent(&self.db_conn, &digest)?,
                Err(err) => eprintln!("Failed to send digest: {:?}", err),
            }
        }

        Ok(())
    }

    fn insert_next_digest(&self, subscription: &Subscription) -> Result<(), String> {
        let digest = InsertDigest {
            subscription_id: subscription.id,
            due: next_due_date_for_subscription(subscription, Utc::now()),
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

    fn send_digest(&self, digest: &Digest) -> Result<(), String> {
        // we send new updates since the last digest
        let previous_digest_sent =
            db::digests_find_previous(&self.db_conn, &digest)?.and_then(|d| d.sent);
        let subscription = db::subscriptions_find_by_digest(&self.db_conn, &digest)?;
        let updates = db::updates_find_new(&self.db_conn, &subscription, previous_digest_sent)?;
        let formatted_updates = updates
            .iter()
            .map(|p| format!("- {}: {}", p.title, p.url))
            .collect::<Vec<String>>()
            .join("\n");
        let email_content = format!(
            "Hi, here's your digest:\n\n{}\n\n-Digester",
            formatted_updates
        );
        let recipient = subscription.email;

        send_email(recipient, email_content)
    }
}

fn next_due_date_for_subscription(
    subscription: &Subscription,
    now: DateTime<Utc>,
) -> DateTime<Utc> {
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

fn send_email(recipient: String, email_content: String) -> Result<(), String> {
    let email = Email::builder()
        .to(recipient)
        .from("digster@digester.io")
        .subject("your digest is ready")
        .text(email_content)
        .build()
        .unwrap();
    // todo use tls
    let mut mailer = SmtpClient::new(("smtp.mailtrap.io", 25), ClientSecurity::None)
        .unwrap()
        .credentials(Credentials::new(
            "5037062aefe9c9652".to_string(),
            "6e7e4d68510493".to_string(),
        ))
        .authentication_mechanism(Mechanism::Plain)
        .transport();
    mailer
        .send(email.into())
        .map(|_| ())
        .map_err(|err| format!("Failed to send email: {:?}", err))
}

#[allow(dead_code)]
mod tests {
    use super::*;
    use chrono::naive::NaiveTime;

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
        }
    }

    fn day(weekday: Weekday, hour: u32, minute: u32) -> DateTime<Utc> {
        // 2nd december is a monday, so if we are passed monday, then 'day' will be 2
        let day = weekday.number_from_monday() + 1;
        Utc.ymd(2019, 12, day).and_hms(hour, minute, 0)
    }

    fn today(hour: u32, minute: u32) -> DateTime<Utc> {
        Utc::now()
            .with_hour(hour)
            .unwrap()
            .with_minute(minute)
            .unwrap()
            .with_nanosecond(0)
            .unwrap()
    }

    fn tomorrow(hour: u32, minute: u32) -> DateTime<Utc> {
        today(hour, minute) + Duration::days(1)
    }
}
