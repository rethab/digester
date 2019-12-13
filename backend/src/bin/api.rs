#![feature(proc_macro_hygiene, decl_macro)]

extern crate backend;

#[macro_use]
extern crate rocket;

extern crate url;

use backend::iam;

use rocket::fairing::AdHoc;

use backend::api::auth;
use backend::api::blog;
use backend::api::common::*;

#[catch(500)]
fn internal_error() -> JsonResponse {
    JsonResponse::InternalServerError
}

#[catch(404)]
fn not_found() -> JsonResponse {
    JsonResponse::NotFound
}

fn main() -> Result<(), rocket_cors::Error> {
    let config_reader = AdHoc::on_attach("Github Identity Provider", |rocket| {
        let github =
            iam::Github::from_rocket_config(rocket.config()).expect("Failed to read github config");
        Ok(rocket.manage(github))
    });

    let mut rocket = rocket::ignite();
    rocket = auth::mount(rocket);
    rocket = blog::mount(rocket);

    rocket
        .attach(DigesterDbConn::fairing())
        .attach(Redis::fairing())
        .attach(cors_fairing()?)
        .attach(config_reader)
        .register(catchers![internal_error, not_found])
        .launch();

    Ok(())
}
