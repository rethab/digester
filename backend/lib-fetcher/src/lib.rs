use lib_channels as channels;
use lib_db as db;

use channels::github_release::GithubRelease;
use channels::rss::Rss;
use channels::twitter::Twitter;
use channels::{Channel, Update};
use chrono::{DateTime, Duration, Utc};

pub struct App<'a> {
    channel_github_release: GithubRelease,
    channel_rss_feed: Rss,
    channel_twitter: Twitter,
    db: &'a db::Connection,
}

impl App<'_> {
    pub fn new(db_conn: &db::Connection, github: GithubRelease, twitter: Twitter) -> App {
        App {
            channel_github_release: github,
            channel_rss_feed: Rss {},
            channel_twitter: twitter,
            db: db_conn,
        }
    }

    pub fn run(&self) -> Result<(), String> {
        let fetch_frequency = Duration::hours(6);

        let channels = self.find_due_channels(fetch_frequency)?;

        if channels.is_empty() {
            println!("Found no channels to update");
        } else {
            println!("Found {} channels to update", channels.len());
        }

        for channel in channels {
            let res = self.fetch_articles(&channel);
            self.update_last_sync(&channel, res)?;
        }

        Ok(())
    }

    fn find_due_channels(&self, fetch_frequency: Duration) -> Result<Vec<db::Channel>, String> {
        db::channels_find_by_last_fetched(&self.db, fetch_frequency)
    }

    fn fetch_articles(&self, channel: &db::Channel) -> Result<(), String> {
        let c = self.get_channel(channel);
        let last_known_update = db::updates_find_newest_by_channel(self.db, channel.id)?;
        let all_updates = c.fetch_updates(&channel.ext_id)?;
        let updates = filter_new_updates(all_updates, last_known_update);

        println!(
            "Found {} updates in {:?} channel {}",
            updates.len(),
            channel.channel_type,
            channel.name
        );

        for update in updates {
            let new_update = db::NewUpdate {
                channel_id: channel.id,
                title: update.title,
                url: update.url,
                published: update.published,
            };
            match db::updates_insert_new(&self.db, &new_update) {
                Ok(_) => {}
                Err(db::InsertError::Unknown(err)) => {
                    return Err(format!("Error during updates insert: {:?}", err))
                }
                Err(db::InsertError::Duplicate) => {
                    println!("Ignoring duplicate update: {}", new_update.title)
                }
            }
        }

        Ok(())
    }

    fn get_channel(&self, channel: &db::Channel) -> &dyn Channel {
        match channel.channel_type {
            db::ChannelType::GithubRelease => &self.channel_github_release,
            db::ChannelType::RssFeed => &self.channel_rss_feed,
            db::ChannelType::Twitter => &self.channel_twitter,
        }
    }

    fn update_last_sync(
        &self,
        channel: &db::Channel,
        sync_result: Result<(), String>,
    ) -> Result<(), String> {
        match sync_result {
            Err(err) => {
                eprintln!(
                    "update_last_sync for {}: failures are not handled yet: {:?}",
                    channel.name, err
                );
                Ok(())
            }
            Ok(()) => {
                db::channels_update_last_fetched(&self.db, channel)?;
                Ok(())
            }
        }
    }
}

// Given all updates returned from a channel (online) and the last known update (db), return the
// ones that are new to us. This function is trickier than it might seem, because some channels
// (looking at you, RSS) don't set a publish date or do set one, but one from the past. One
// particular nasty example (this time looking at keycloak): They would pubish blog entries at
// around 9am, but set the publish date to 00:00am. Since we last tried to fetch their blog at
// 3am, we would ignore the update, because the published date is before the last_fetched.
//
// Strategy:
//  - if we fetch a channel for the first time (last_known == None), we look at the published date
//    and take everything one week back (to be on the safe side)
//  - otherise, take everything with a published date newer than the inserted date of the last_known
//
// Limitatioons:
//  - Blog X publishes entry A at 9am sets date to 00:00am. We fetch it at 9:15am. At 10am they
//    publish entry B and again set the date to 00:00am. We'd miss the second entry
//  - Blog Y publishes entry A at 1.1.1970
fn filter_new_updates(updates: Vec<Update>, last_known: Option<db::Update>) -> Vec<Update> {
    // do we also need to change the digester code? does that currently work on published or inserted?
    let one_week_ago: &DateTime<Utc> = &(Utc::now() - Duration::weeks(1));
    let filter_fn = |u: &Update| match last_known.as_ref() {
        Some(lk) => u.published > lk.inserted,
        None => u.published > *one_week_ago,
    };

    updates.into_iter().filter(filter_fn).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ignore_updates_older_than_a_week_in_first_fetch() {
        let new_update = mk_update(Utc::now());
        let old_update = mk_update(Utc::now() - Duration::days(9));
        assert_eq!(
            vec![new_update.clone()],
            filter_new_updates(vec![new_update, old_update], None)
        );
    }

    #[test]
    fn include_updates_from_the_future_in_first_fetch() {
        let new_update = mk_update(Utc::now() + Duration::days(1));
        assert_eq!(
            vec![new_update.clone()],
            filter_new_updates(vec![new_update], None)
        );
    }

    #[test]
    fn include_updates_from_earlier_today_if_after_last_update() {
        let update = mk_update(Utc::now() - Duration::hours(5));
        let last_known_update = Some(mk_db_update(Utc::now() - Duration::hours(6)));
        assert_eq!(
            vec![update.clone()],
            filter_new_updates(vec![update], last_known_update)
        );
    }

    #[test]
    fn include_two_updates_from_earlier_this_week_if_after_last_update() {
        let old_update = mk_update(Utc::now() - Duration::days(8));
        let new_update = mk_update(Utc::now() - Duration::days(1));
        let new_update_2 = mk_update(Utc::now() - Duration::days(2));
        let last_known_update = Some(mk_db_update(Utc::now() - Duration::days(3)));
        assert_eq!(
            vec![new_update.clone(), new_update_2.clone()],
            filter_new_updates(
                vec![old_update, new_update, new_update_2],
                last_known_update
            )
        );
    }

    #[test]
    fn ignore_updates_from_earlier_than_last_update() {
        let old_update = mk_update(Utc::now() - Duration::hours(5));
        let new_update = mk_update(Utc::now());
        let last_known_update = Some(mk_db_update(Utc::now() - Duration::hours(4)));
        assert_eq!(
            vec![new_update.clone()],
            filter_new_updates(vec![old_update, new_update], last_known_update)
        );
    }

    #[test]
    fn ignore_updates_from_the_same_time_as_the_last_update() {
        let timestamp = Utc::now() - Duration::hours(4);
        let old_update = mk_update(timestamp);
        let last_known_update = Some(mk_db_update(timestamp));
        assert_eq!(
            vec![] as Vec<Update>,
            filter_new_updates(vec![old_update], last_known_update)
        );
    }

    fn mk_update(published: DateTime<Utc>) -> Update {
        Update {
            title: "title".into(),
            url: "url".into(),
            published,
        }
    }

    fn mk_db_update(inserted: DateTime<Utc>) -> db::Update {
        db::Update {
            id: 1,
            channel_id: 2,
            title: "title".into(),
            url: "url".into(),
            published: Utc::now(),
            inserted,
        }
    }
}
