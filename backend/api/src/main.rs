#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

extern crate api;

use api::iam;

use rocket::fairing::AdHoc;
use std::env;

use api::controllers::auth;
use api::controllers::common::*;
use api::controllers::subscriptions;

#[catch(500)]
fn internal_error() -> JsonResponse {
    JsonResponse::InternalServerError
}

#[catch(404)]
fn not_found() -> JsonResponse {
    JsonResponse::NotFound
}

fn main() -> Result<(), rocket_cors::Error> {
    let github_identity_provider = AdHoc::on_attach("Github Identity Provider", |rocket| {
        let github =
            iam::Github::from_rocket_config(rocket.config()).expect("Failed to read github config");
        Ok(rocket.manage(github))
    });

    let github_api_token = AdHoc::on_attach("Github Api Token", |rocket| {
        let name = "GITHUB_API_TOKEN";
        let api_token = env::var(name).expect(&format!("Failed to read env variable {}", name));
        Ok(rocket.manage(subscriptions::GithubApiToken(api_token)))
    });

    let mut rocket = rocket::ignite();
    rocket = auth::mount(rocket);
    rocket = subscriptions::mount(rocket);

    rocket
        .attach(DigesterDbConn::fairing())
        .attach(Redis::fairing())
        .attach(cors_fairing()?)
        .attach(github_identity_provider)
        .attach(github_api_token)
        .register(catchers![internal_error, not_found])
        .launch();

    Ok(())
}
