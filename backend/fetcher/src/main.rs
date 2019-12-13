use chrono::{DateTime, Duration, Utc};
use lib_db as db;
use lib_db::{Blog, NewPost};
use rss::Channel;

fn main() -> Result<(), String> {
    let fetch_frequency = Duration::hours(6);

    let db_conn = db::connection_from_env()?;
    let blogs = find_due_blogs(fetch_frequency, &db_conn)?;

    if blogs.is_empty() {
        println!("Found no blogs to update")
    }

    for blog in blogs {
        let res = fetch_articles(&blog, &db_conn);
        update_last_sync(&blog, res, &db_conn)?;
    }

    Ok(())
}

fn find_due_blogs(fetch_frequency: Duration, conn: &db::Connection) -> Result<Vec<Blog>, String> {
    db::blogs_find_by_last_fetched(conn, fetch_frequency)
}

fn fetch_articles(blog: &Blog, conn: &db::Connection) -> Result<(), String> {
    let channel = Channel::from_url(&blog.url)
        .map_err(|err| format!("failed to fetch blog from url '{}': {:?}", blog.url, err))?;
    println!(
        "Found {} articles for blog {}",
        channel.items().len(),
        blog.url
    );
    for item in channel.items() {
        let post = NewPost {
            blog_id: blog.id,
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
        let already_seen = blog
            .last_fetched
            .map(|lf| post.published > lf)
            .unwrap_or(false);
        if already_seen {
            println!("Ignoring known post: {}", post.title);
        } else {
            match db::posts_insert_new(&conn, &post) {
                Ok(_) => {}
                Err(db::InsertError::Unknown) => return Err("Error during posts insert".to_owned()),
                Err(db::InsertError::Duplicate) => {
                    println!("Ignoring duplicate post: {}", post.title)
                }
            }
        }
    }
    Ok(())
}

fn update_last_sync(
    blog: &Blog,
    sync_result: Result<(), String>,
    conn: &db::Connection,
) -> Result<(), String> {
    match sync_result {
        Err(err) => {
            eprintln!("update_last_sync: failures are not handled yet: {:?}", err);
            Ok(())
        }
        Ok(()) => {
            db::blogs_update_last_fetched(&conn, blog)?;
            println!("Updated last_fetched of blog {}", blog.id);
            Ok(())
        }
    }
}
