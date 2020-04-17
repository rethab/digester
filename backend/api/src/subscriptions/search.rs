use lib_db as db;

use db::ChannelType;
use diesel::pg::PgConnection;
use lib_channels::{Channel, SearchError as ChannelSearchError};

pub enum SearchError {
    InvalidInput,
    NotFound,
    Unknown,
    Timeout,
}

pub fn search(
    db: &PgConnection,
    channel_type: ChannelType,
    channel: &dyn Channel,
    query: &str,
) -> Result<Vec<db::Channel>, SearchError> {
    let db_query = channel.sanitize_for_db_search(&query).map_err(|err| {
        eprintln!(
            "Query is invalid for channel type {:?}: {:?}",
            channel_type, err
        );
        SearchError::InvalidInput
    })?;

    let channels = match db::channels_search(&db, channel_type, &db_query.0) {
        Ok(channels) => channels,
        Err(err) => {
            eprintln!(
                "Failed to search for channel by query '{}': {:?}",
                db_query.0, err
            );
            return Err(SearchError::Unknown);
        }
    };

    println!(
        "Found {} channels in db search '{}'",
        channels.len(),
        db_query.0
    );

    if !channels.is_empty() {
        return Ok(channels);
    }

    let online_query = channel.sanitize(&query).map_err(|err| {
        eprintln!("Query is not a URL: {:?}", err);
        SearchError::InvalidInput
    })?;

    let channels = channel.search(online_query).map_err(|err| match err {
        ChannelSearchError::ChannelNotFound(msg) => {
            eprintln!("Channel not found: {}", msg);
            SearchError::NotFound
        }
        ChannelSearchError::TechnicalError(msg) => {
            eprintln!("Technical error during search: {}", msg);
            SearchError::Unknown
        }
        ChannelSearchError::Timeout(msg) => {
            eprintln!("Timeout during online search: {}", msg);
            SearchError::Timeout
        }
    })?;

    if channels.is_empty() {
        return Ok(Vec::new());
    }

    let channels = channels
        .iter()
        .map(|f| db::NewChannel {
            channel_type,
            name: f.name.clone(),
            url: f.url.clone(),
            link: f.link.clone(),
        })
        .collect();

    db::channels_insert_many(&db, channels).map_err(|err| {
        eprintln!("Failed to insert new channels: {:?}", err);
        SearchError::Unknown
    })
}
