use channels::github_release::GithubRelease;
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
    database_uri: String,
    #[structopt(long)]
    mailjet_user: String,
    #[structopt(long)]
    mailjet_password: String,
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
    let mailjet = digester::MailjetCredentials {
        username: opt.mailjet_user,
        password: opt.mailjet_password,
    };
    println!("Running worker in {:?} mode", opt.app_env);
    fetcher::App::new(&db_conn, github).run()?;
    digester::App::new(&db_conn, mailjet, opt.app_env.into()).run()
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
