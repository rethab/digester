use chrono::Duration;
use lib_channels as channels;
use lib_db as db;
use lib_db::{Channel, ChannelType, NewUpdate};

fn main() -> Result<(), String> {
    let fetch_frequency = Duration::hours(6);

    let db_conn = db::connection_from_env()?;
    let channels = find_due_channels(fetch_frequency, &db_conn)?;

    if channels.is_empty() {
        println!("Found no channels to update")
    }

    for channel in channels {
        let res = fetch_articles(&channel, &db_conn);
        update_last_sync(&channel, res, &db_conn)?;
    }

    Ok(())
}

fn find_due_channels(
    fetch_frequency: Duration,
    conn: &db::Connection,
) -> Result<Vec<Channel>, String> {
    db::channels_find_by_last_fetched(conn, fetch_frequency)
}

fn fetch_articles(channel: &Channel, conn: &db::Connection) -> Result<(), String> {
    let c = get_channel(channel);

    let updates = c.fetch_updates(&channel.name, channel.last_fetched)?;

    for update in updates {
        let new_update = NewUpdate {
            channel_id: channel.id,
            title: update.title,
            url: update.url,
            published: update.published,
        };
        match db::updates_insert_new(&conn, &new_update) {
            Ok(_) => {}
            Err(db::InsertError::Unknown) => return Err("Error during updates insert".to_owned()),
            Err(db::InsertError::Duplicate) => {
                println!("Ignoring duplicate update: {}", new_update.title)
            }
        }
    }

    Ok(())
}

fn get_channel(channel: &Channel) -> Box<dyn channels::Channel> {
    match channel.type_ {
        ChannelType::GithubRelease => Box::new(channels::github_release::GithubRelease),
    }
}

fn update_last_sync(
    channel: &Channel,
    sync_result: Result<(), String>,
    conn: &db::Connection,
) -> Result<(), String> {
    match sync_result {
        Err(err) => {
            eprintln!("update_last_sync: failures are not handled yet: {:?}", err);
            Ok(())
        }
        Ok(()) => {
            db::channels_update_last_fetched(&conn, channel)?;
            println!("Updated last_fetched of channel {}", channel.id);
            Ok(())
        }
    }
}
