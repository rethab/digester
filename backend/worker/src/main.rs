use channels::github_release::GithubRelease;
use channels::twitter::Twitter;
use lib_channels as channels;
use lib_db as db;
use lib_digester as digester;
use lib_fetcher as fetcher;
use std::str::FromStr;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(long)]
    github_api_token: String,
    #[structopt(long)]
    twitter_api_key: String,
    #[structopt(long)]
    twitter_api_secret_key: String,
    #[structopt(long)]
    twitter_access_token: String,
    #[structopt(long)]
    twitter_access_token_secret: String,
    #[structopt(long)]
    database_uri: String,
    #[structopt(long)]
    sendgrid_api_key: String,
    #[structopt(long = "app-env", default_value = "prod")]
    app_env: AppEnv,
}

#[derive(StructOpt, Debug)]
enum AppEnv {
    Dev,
    Stg,
    Prod,
}

fn main() -> Result<(), String> {
    let opt = Opt::from_args();
    let db_conn = db::connection_from_str(&opt.database_uri)?;
    let github = GithubRelease::new(&opt.github_api_token)?;
    let twitter = Twitter::new(
        &opt.twitter_api_key,
        &opt.twitter_api_secret_key,
        &opt.twitter_access_token,
        &opt.twitter_access_token_secret,
    )?;
    let sendgrid = digester::SendgridCredentials {
        api_key: opt.sendgrid_api_key,
    };
    println!("Running worker in {:?} mode", opt.app_env);
    fetcher::App::new(&db_conn, github, twitter).run()?;
    digester::App::new(&db_conn, sendgrid, opt.app_env.into()).run()
}

impl FromStr for AppEnv {
    type Err = String;
    fn from_str(param: &str) -> Result<Self, String> {
        match param {
            "dev" => Ok(AppEnv::Dev),
            "stg" => Ok(AppEnv::Stg),
            "prod" => Ok(AppEnv::Prod),
            unknown => Err(format!("Invalid value for app_env: {}", unknown)),
        }
    }
}

impl Into<digester::Env> for AppEnv {
    fn into(self) -> digester::Env {
        match self {
            AppEnv::Dev => digester::Env::Dev,
            AppEnv::Stg => digester::Env::Stg,
            AppEnv::Prod => digester::Env::Prod,
        }
    }
}
