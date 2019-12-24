use channels::github_release::GithubRelease;
use lib_channels as channels;
use lib_db as db;
use lib_digester as digester;
use lib_fetcher as fetcher;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(long)]
    github_api_token: String,
    #[structopt(long)]
    database_uri: String,
    #[structopt(long)]
    mailjet_user: String,
    #[structopt(long)]
    mailjet_password: String,
}

fn main() -> Result<(), String> {
    let opt = Opt::from_args();
    let db_conn = db::connection_from_str(&opt.database_uri)?;
    let github = GithubRelease::new(&opt.github_api_token)?;
    let mailjet = digester::MailjetCredentials {
        username: opt.mailjet_user,
        password: opt.mailjet_password,
    };
    fetcher::App::new(&db_conn, github).run()?;
    digester::App::new(&db_conn, mailjet).run()
}
