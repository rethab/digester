use chrono::{DateTime, Duration, Utc};
use lib_db as db;
use lib_db::{Channel, NewUpdate};
use rss::Channel as RssChannel;

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
    let rss_channel = RssChannel::from_url(&channel.name).map_err(|err| {
        format!(
            "failed to fetch channel from url '{}': {:?}",
            channel.url, err
        )
    })?;
    println!(
        "Found {} articles for channel {}",
        rss_channel.items().len(),
        rss_channel.url
    );
    for item in rss_channel.items() {
        let update = NewUpdate {
            channel_id: rss_channel.id,
            title: item
                .title()
                .ok_or_else(|| format!("No title for {:?}", item))?
                .to_owned(),
            author: item.author().map(|author| author.to_owned()),
            url: item
                .link()
                .ok_or_else(|| format!("No url for {:?}", item))?
                .to_owned(),
            // todo don't ignore parse error
            published: item
                .pub_date()
                .map(|date| {
                    DateTime::parse_from_rfc2822(date)
                        .map(|dt| dt.with_timezone(&Utc))
                        .map_err(|parse_err| {
                            format!("Failed to parse date '{}': {:?}", date, parse_err)
                        })
                })
                .ok_or_else(|| format!("No pub_date for {:?}", item))??,
            inserted: Utc::now(),
        };
        // todo this is a technical error, which should be handled differently from the above business error
        let already_seen = channel
            .last_fetched
            .map(|lf| update.published > lf)
            .unwrap_or(false);
        if already_seen {
            println!("Ignoring known update: {}", update.title);
        } else {
            match db::updates_insert_new(&conn, &update) {
                Ok(_) => {}
                Err(db::InsertError::Unknown) => {
                    return Err("Error during updates insert".to_owned())
                }
                Err(db::InsertError::Duplicate) => {
                    println!("Ignoring duplicate update: {}", update.title)
                }
            }
        }
    }
    Ok(())
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
