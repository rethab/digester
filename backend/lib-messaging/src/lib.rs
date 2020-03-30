#[derive(PartialEq, Debug)]
pub enum Env {
    Prod,
    Stg,
    Dev,
}

pub mod sendgrid;
