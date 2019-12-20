use channels::github_release::GithubRelease;
use chrono::Duration;
use lib_channels as channels;
use lib_db as db;
use lib_db::{Channel, ChannelType, NewUpdate};

pub struct App<'a> {
    channel_github_release: GithubRelease,
    db: &'a db::Connection,
}

impl App<'_> {
    pub fn new(db_conn: &db::Connection, github: GithubRelease) -> App {
        App {
            channel_github_release: github,
            db: db_conn,
        }
    }

    pub fn run(&self) -> Result<(), String> {
        let fetch_frequency = Duration::hours(6);

        let channels = self.find_due_channels(fetch_frequency)?;

        if channels.is_empty() {
            println!("Found no channels to update")
        }

        for channel in channels {
            let res = self.fetch_articles(&channel);
            self.update_last_sync(&channel, res)?;
        }

        Ok(())
    }

    fn find_due_channels(&self, fetch_frequency: Duration) -> Result<Vec<Channel>, String> {
        db::channels_find_by_last_fetched(&self.db, fetch_frequency)
    }

    fn fetch_articles(&self, channel: &Channel) -> Result<(), String> {
        let c = self.get_channel(channel);

        let updates = c.fetch_updates(&channel.name, channel.last_fetched)?;

        for update in updates {
            let new_update = NewUpdate {
                channel_id: channel.id,
                title: update.title,
                url: update.url,
                published: update.published,
            };
            match db::updates_insert_new(&self.db, &new_update) {
                Ok(_) => {}
                Err(db::InsertError::Unknown) => {
                    return Err("Error during updates insert".to_owned())
                }
                Err(db::InsertError::Duplicate) => {
                    println!("Ignoring duplicate update: {}", new_update.title)
                }
            }
        }

        Ok(())
    }

    fn get_channel(&self, channel: &Channel) -> &dyn channels::Channel {
        match channel.channel_type {
            ChannelType::GithubRelease => &self.channel_github_release,
        }
    }

    fn update_last_sync(
        &self,
        channel: &Channel,
        sync_result: Result<(), String>,
    ) -> Result<(), String> {
        match sync_result {
            Err(err) => {
                eprintln!("update_last_sync: failures are not handled yet: {:?}", err);
                Ok(())
            }
            Ok(()) => {
                db::channels_update_last_fetched(&self.db, channel)?;
                println!("Updated last_fetched of channel {}", channel.id);
                Ok(())
            }
        }
    }
}