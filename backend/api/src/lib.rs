#![feature(proc_macro_hygiene, decl_macro, option_result_contains)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate serde_derive;

pub mod cache;
pub mod controllers;
pub mod iam;
pub mod lists;
pub mod ratelimiting;
pub mod subscriptions;
