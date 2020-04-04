use chrono::{DateTime, Datelike, Duration, TimeZone, Timelike, Utc, Weekday};
use chrono_tz::Tz;
use lib_db as db;
use lib_db::{Digest, Frequency, InsertDigest, Subscription, User};
use lib_messaging as messaging;
use messaging::sendgrid::*;

pub use messaging::sendgrid::SendgridCredentials;

pub struct App<'a> {
    db_conn: &'a db::Connection,
    sendgrid: SendgridCredentials,
    env: Env,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Env {
    Dev,
    Stg,
    Prod,
}

impl Into<messaging::Env> for Env {
    fn into(self) -> messaging::Env {
        match self {
            Env::Dev => messaging::Env::Dev,
            Env::Stg => messaging::Env::Stg,
            Env::Prod => messaging::Env::Prod,
        }
    }
}

impl App<'_> {
    pub fn new(db_conn: &db::Connection, sendgrid: SendgridCredentials, env: Env) -> App {
        App {
            db_conn,
            sendgrid,
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
        let timezone = match subscription.timezone.as_ref() {
            Some(tz) => tz.0,
            None => match subscription.user_id {
                None => {
                    return Err(format!(
                        "Subscription has neither user nor timezone: {}",
                        subscription.id
                    ));
                }
                Some(user_id) => {
                    let user = db::users_find_by_id0(&self.db_conn, user_id)?;
                    user.timezone.map(|tz| tz.0).unwrap_or({
                        eprintln!("User {} has no timezone, using UTC", user.id);
                        Tz::UTC
                    })
                }
            },
        };

        let now_in_tz: DateTime<Tz> = timezone.from_utc_datetime(&Utc::now().naive_utc());
        let due_in_tz = next_due_date_for_subscription(subscription, now_in_tz);
        let due_date = due_in_tz.with_timezone(&Utc);

        let digest = InsertDigest {
            subscription_id: subscription.id,
            due: due_date,
        };
        match db::digests_insert(&self.db_conn, &digest) {
            Ok(()) => Ok(()),
            Err(db::InsertError::Unknown(err)) => Err(format!(
                "Failed to insert new digest {:?}: {:?}",
                digest, err
            )),
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
        let mut channel_digests = Vec::new();
        let mut list_digests = Vec::new();

        for (digest, subscription) in d_and_s {
            match (subscription.channel_id, subscription.list_id) {
                (Some(channel_id), None) => {
                    channel_digests.push((digest, subscription, channel_id))
                }
                (None, Some(list_id)) => list_digests.push((digest, subscription, list_id)),
                _ => {
                    return Err(format!(
                        "Subscription {} has both channel and list set or none",
                        subscription.id
                    ))
                }
            }
        }

        // channel messages are batched
        let mut messages = Vec::new();
        if let Some(message) = self.create_message_for_channels(&user, channel_digests)? {
            messages.push(message);
        }

        // list messages are sent per list
        for (digest, subscription, list_id) in list_digests {
            if let Some(message) =
                self.create_message_for_lists(&user, digest, subscription, list_id)?
            {
                messages.push(message)
            }
        }

        if let Some(ne_messages) = NEVec::from_vec(messages) {
            messaging::sendgrid::send_email(&self.sendgrid, ne_messages)
        } else {
            Ok(())
        }
    }

    fn create_message_for_channels(
        &self,
        user: &User,
        d_and_s: Vec<(&Digest, &Subscription, i32)>,
    ) -> Result<Option<SendgridMessage>, String> {
        let mut sendgrid_subscriptions = Vec::with_capacity(d_and_s.len());
        for (digest, subscription, channel_id) in &d_and_s {
            // we send new updates since the last digest or since
            // when the subscription was created if this is the first digest
            let updates_since = db::digests_find_previous(&self.db_conn, &digest)?
                .and_then(|d| d.sent)
                .unwrap_or(subscription.inserted);
            let updates = db::updates_find_new(&self.db_conn, *channel_id, updates_since)?;
            if !updates.is_empty() {
                let channel = db::channels_find_by_id(&self.db_conn.0, *channel_id)?;
                let sendgrid_updates = updates
                    .into_iter()
                    .map(|u| SendgridUpdate {
                        title: u.title,
                        url: u.url,
                    })
                    .collect();
                sendgrid_subscriptions
                    .push(SendgridSubscription::new(&channel.name, sendgrid_updates));
            }
        }

        if sendgrid_subscriptions.is_empty() {
            // happens if user has a due digest, but we have no updates..
            println!(
                "No updates to send for User {} in any of their digests",
                user.id
            );
            Ok(None)
        } else {
            println!(
                "{} updates to send for user {}",
                sendgrid_subscriptions.len(),
                user.id
            );
            let subject =
                digests::create_subject(&self.env.clone().into(), &sendgrid_subscriptions);
            let recipient = d_and_s[0].1.email.clone();
            Ok(Some(SendgridMessage::new(
                recipient,
                subject,
                sendgrid_subscriptions,
            )))
        }
    }

    fn create_message_for_lists(
        &self,
        user: &User,
        digest: &Digest,
        sub: &Subscription,
        list_id: i32,
    ) -> Result<Option<SendgridMessage>, String> {
        let list = match db::lists_find_by_id(&self.db_conn.0, list_id)? {
            None => return Err(format!("List with id {} not found", list_id)),
            Some((list, _)) => list,
        };

        // we send new updates since the last digest or since
        // when the subscription was created if this is the first digest
        let updates_since = db::digests_find_previous(&self.db_conn, digest)?
            .and_then(|d| d.sent)
            .unwrap_or(sub.inserted);

        let channels = db::channels_find_by_list_id(&self.db_conn.0, list_id)?;

        let mut sendgrid_subscriptions = Vec::with_capacity(channels.len());
        for channel in channels {
            let updates = db::updates_find_new(&self.db_conn, channel.id, updates_since)?;
            if !updates.is_empty() {
                let sendgrid_updates = updates
                    .into_iter()
                    .map(|u| SendgridUpdate {
                        title: u.title,
                        url: u.url,
                    })
                    .collect();
                sendgrid_subscriptions
                    .push(SendgridSubscription::new(&channel.name, sendgrid_updates));
            }
        }

        if sendgrid_subscriptions.is_empty() {
            // happens if user has a due digest, but we have no updates..
            println!(
                "No updates to send for User {} in any of their digests (via lists)",
                user.id
            );
            Ok(None)
        } else {
            println!(
                "{} updates to send for user {} (via lists)",
                sendgrid_subscriptions.len(),
                user.id
            );
            let subject = digests::create_subject_for_list(&self.env.clone().into(), &list.name);
            let recipient = sub.email.clone();
            Ok(Some(SendgridMessage::new(
                recipient,
                subject,
                sendgrid_subscriptions,
            )))
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
            timezone: None,
            channel_id: Some(1),
            list_id: None,
            user_id: Some(1),
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
            timezone: None,
            channel_id: Some(1),
            list_id: None,
            user_id: Some(1),
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
