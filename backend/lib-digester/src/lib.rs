use chrono::{DateTime, Datelike, Duration, TimeZone, Timelike, Utc, Weekday};
use chrono_tz::Tz;
use lib_db as db;
use lib_db::{Digest, Frequency, InsertDigest, Subscription, User};

mod messaging;

use messaging::*;

pub use messaging::MailjetCredentials;

pub struct App<'a> {
    db_conn: &'a db::Connection,
    mailjet: MailjetCredentials,
    env: Env,
}

#[derive(Debug, PartialEq)]
pub enum Env {
    Dev,
    Stg,
    Prod,
}

impl App<'_> {
    pub fn new(db_conn: &db::Connection, mailjet: MailjetCredentials, env: Env) -> App {
        App {
            db_conn,
            mailjet,
            env,
        }
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
                let channel = db::channels_find_by_id(&self.db_conn.0, subscription.channel_id)?;
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
            println!(
                "{} updates to send for user {}",
                mailjet_subscriptions.len(),
                user.id
            );
            let subject = messaging::create_subject(&self.env, &mailjet_subscriptions);
            let recipient = d_and_s[0].1.email.clone();
            let message = MailjetMessage::new(recipient, subject, mailjet_subscriptions);
            messaging::send_email(&self.mailjet, message)
        }
    }
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
    use lib_db::Day;

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
