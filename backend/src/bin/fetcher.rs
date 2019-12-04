use chrono::{DateTime, Duration, Utc};
use postgres::{Client, NoTls};
use rss::Channel;
use std::env;

struct Blog {
    id: i32,
    url: String,
    last_fetched: Option<DateTime<Utc>>,
}

#[derive(Debug)]
struct Post {
    blog_id: i32,
    title: String,
    author: Option<String>,
    url: String,
    created: DateTime<Utc>,
}

fn main() -> Result<(), String> {
    let fetch_frequency = Duration::hours(6);

    let mut db_conn = setup_connection()?;
    let blogs = find_due_blogs(fetch_frequency, &mut db_conn)?;

    if blogs.is_empty() {
        println!("Found no blogs to update")
    }

    for blog in blogs {
        let res = fetch_articles(&blog, &mut db_conn);
        update_last_sync(&blog, res, &mut db_conn)?;
    }

    Ok(())
}

fn setup_connection() -> Result<Client, String> {
    let connection_string = env::var("DATABASE_CONNECTION")
        .map_err(|_err| "Missing connection string in env variable".to_owned())?;
    Client::connect(connection_string.as_str(), NoTls)
        .map_err(|err| format!("could not connect to db: {:?}", err))
}

fn find_due_blogs(fetch_frequency: Duration, client: &mut Client) -> Result<Vec<Blog>, String> {
    let mut blogs = Vec::new();
    let since_last_fetched = Utc::now() - fetch_frequency;
    for row in client
        .query(
            "SELECT id, url, last_fetched FROM blogs WHERE last_fetched < $1 OR last_fetched IS NULL",
            &[&since_last_fetched],
        )
        .map_err(|err| format!("failed to run query in find_due_blogs: {:?}", err))?
    {
        let id: i32 = row.get(0);
        let url: String = row.get(1);
        let last_fetched: Option<DateTime<Utc>> = row.get(2);
        blogs.push(Blog {
            id,
            url,
            last_fetched,
        });
    }
    Ok(blogs)
}

fn fetch_articles(blog: &Blog, client: &mut Client) -> Result<(), String> {
    let channel = Channel::from_url(&blog.url)
        .map_err(|err| format!("failed to fetch blog from url '{}': {:?}", blog.url, err))?;
    println!(
        "Found {} articles for blog {}",
        channel.items().len(),
        blog.url
    );
    for item in channel.items() {
        let post = Post {
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
            created: item
                .pub_date()
                .map(|date| {
                    DateTime::parse_from_rfc2822(date)
                        .map(|dt| dt.with_timezone(&Utc))
                        .map_err(|parse_err| {
                            format!("Failed to parse date '{}': {:?}", date, parse_err)
                        })
                })
                .ok_or_else(|| format!("No pub_date for {:?}", item))??,
        };
        // todo this is a technical error, which should be handled differently from the above business error
        let already_seen = blog
            .last_fetched
            .map(|lf| post.created > lf)
            .unwrap_or(false);
        if already_seen {
            println!("Ignoring known post{}", post.title);
        } else {
            client
                .execute(
                    "INSERT INTO posts (blog_id, title, author, url) VALUES ($1, $2, $3, $4)",
                    &[&post.blog_id, &post.title, &post.author, &post.url],
                )
                .map_err(|err| format!("failed to insert new post: {:?}", err))?;
        }
    }
    Ok(())
}

fn update_last_sync(
    blog: &Blog,
    sync_result: Result<(), String>,
    client: &mut Client,
) -> Result<(), String> {
    match sync_result {
        Err(err) => {
            eprintln!("update_last_sync: failures are not handled yet: {:?}", err);
            Ok(())
        }
        Ok(()) => {
            client
                .execute(
                    "UPDATE blogs SET last_fetched = NOW() WHERE id = $1",
                    &[&blog.id],
                )
                .map_err(|err| {
                    format!(
                        "failed to update last_fetched field for blog {}: {:?}",
                        blog.id, err
                    )
                })?;
            println!("Updated last_fetched of blog {}", blog.id);
            Ok(())
        }
    }
}
