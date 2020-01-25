use rocket_contrib::databases::redis::Connection as RedisConnection;
use time::Duration;

use diesel::pg::PgConnection;

use super::cache;
use lib_db as db;

use uuid::Uuid;

pub mod facebook;
pub mod github;

pub struct User {
    id: i32,
    username: String,
    pub first_login: bool,
}

impl User {
    fn from_db(user: db::User, identity: db::Identity, first_login: bool) -> User {
        User {
            id: user.id,
            username: identity.username,
            first_login,
        }
    }
}

pub struct ProviderUserInfo {
    provider: &'static str,
    pid: String,
    email: String,
    username: String,
}

#[derive(Clone)]
pub struct Session {
    pub id: Uuid,
    pub user_id: i32,
    pub username: String,
}

impl Session {
    pub fn lifetime() -> Duration {
        Duration::weeks(48)
    }
    pub fn generate(user_id: i32, username: String) -> Session {
        Session {
            id: Uuid::new_v4(),
            user_id,
            username,
        }
    }

    fn from_data(id: Uuid, data: cache::SessionData) -> Session {
        Session {
            id,
            user_id: data.user_id,
            username: data.username,
        }
    }
}

// the code we get from github when they call us
pub struct AuthorizationCode(pub String);
// the code we get in exchange for the AuthorizationCode
pub struct AccessToken(String);

pub enum AuthenticationError {
    TokenExchangeFailed(String),
    MissingPermissions(String),
    UnknownFailure(String),
}

pub trait IdentityProvider {
    fn exchange_token(&self, code: AuthorizationCode) -> Result<AccessToken, AuthenticationError>;
    fn fetch_user_info(
        &self,
        access_token: AccessToken,
    ) -> Result<ProviderUserInfo, AuthenticationError>;
}

pub fn authenticate<P: IdentityProvider>(
    conn: &PgConnection,
    cache: &mut RedisConnection,
    provider: &P,
    code: AuthorizationCode,
) -> Result<(User, Session), AuthenticationError> {
    let access_token = provider.exchange_token(code)?;
    let user_info = provider.fetch_user_info(access_token)?;
    let user = fetch_or_insert_user_in_db(conn, &user_info)
        .map_err(AuthenticationError::UnknownFailure)?;
    let session = create_session(cache, &user).map_err(AuthenticationError::UnknownFailure)?;
    Ok((user, session))
}

pub fn fetch_session(cache: &RedisConnection, session_id: Uuid) -> Result<Option<Session>, String> {
    let id = cache::SessionId(session_id);
    let maybe_data = cache::session_find(cache, id)?;
    Ok(maybe_data.map(|data| Session::from_data(session_id, data)))
}

pub fn logout(cache: &mut RedisConnection, session: Session) -> Result<(), String> {
    cache::session_delete(cache, cache::SessionId(session.id))
}

// inserts or updates the identity and creates/fetches the linked user
// identities from different providers with the same e-mail address are linked to the same user
// notably what we don't support (yet) is linking identities from different
// providers with different e-mail addresses
fn fetch_or_insert_user_in_db(
    conn: &PgConnection,
    user_info: &ProviderUserInfo,
) -> Result<User, String> {
    let identities = db::identities_find_by_email_or_id(
        conn,
        user_info.provider,
        &user_info.pid,
        &user_info.email,
    )?;

    let maybe_identity: Option<db::Identity> = identities
        .iter()
        .find(|i| i.pid == user_info.pid && i.provider == user_info.provider)
        .cloned();

    let maybe_other_identity: Option<db::Identity> = identities
        .iter()
        .find(|i| i.provider != user_info.provider && i.email == user_info.email)
        .cloned();

    match maybe_identity {
        Some(identity) if identity.email != user_info.email => {
            // user has updated e-mail in identity provider
            let identity = db::identities_update_email(conn, identity, &user_info.email)?;
            println!("Updated e-mail address of identity {}", identity.id);
            let user = db::users_find_by_id(conn, identity.user_id)?;
            Ok(User::from_db(user, identity, false))
        }
        Some(identity) => {
            // user exists unmodified. just fetch
            let user = db::users_find_by_id(conn, identity.user_id)?;
            Ok(User::from_db(user, identity, false))
        }
        None => {
            // new login from this provider.
            match maybe_other_identity {
                Some(identity) => {
                    // another identity has the same e-mail address
                    // --> link new one to same user
                    let new_identity = db::NewIdentity {
                        provider: user_info.provider.to_owned(),
                        pid: user_info.pid.to_owned(),
                        user_id: identity.user_id,
                        email: user_info.email.to_owned(),
                        username: user_info.username.to_owned(),
                    };
                    let identity = db::identities_insert(conn, new_identity)?;
                    let user = db::users_find_by_id(conn, identity.user_id)?;
                    Ok(User::from_db(user, identity, false))
                }
                None => {
                    // e-mail address not seen before, all new
                    let new_identity = db::NewUserData {
                        provider: user_info.provider.to_owned(),
                        pid: user_info.pid.to_owned(),
                        email: user_info.email.to_owned(),
                        username: user_info.username.to_owned(),
                    };
                    let (user, identity) = db::users_insert(conn, new_identity)?;
                    Ok(User::from_db(user, identity, true)) // true means new user
                }
            }
        }
    }
}

fn create_session(c: &mut RedisConnection, user: &User) -> Result<Session, String> {
    let session = Session::generate(user.id, user.username.clone());
    let data = cache::SessionData {
        user_id: user.id,
        username: user.username.clone(),
    };
    cache::session_store(c, cache::SessionId(session.id), &data, Session::lifetime())?;
    Ok(session)
}

pub fn create_delete_challenge(c: &mut RedisConnection, user_id: i32) -> Result<String, String> {
    let challenge = Uuid::new_v4().to_string().split_at(6).0.to_owned();
    let duration = Duration::minutes(3);
    cache::delete_challenge_store(c, user_id, &challenge, duration).map(|_| challenge)
}

pub enum DeleteError {
    Unknown(String),
    MissingChallenge,
    InvalidChallengeResponse,
}
pub fn delete_account(
    c: &mut RedisConnection,
    db: &PgConnection,
    user_id: i32,
    challenge_response: &str,
) -> Result<(), DeleteError> {
    let challenge = match cache::delete_challenge_get_and_delete(c, user_id) {
        Err(err) => {
            return Err(DeleteError::Unknown(format!(
                "Failed to get delete challenge: {:?}",
                err
            )))
        }
        Ok(None) => return Err(DeleteError::MissingChallenge),
        Ok(Some(challenge)) => challenge,
    };

    if challenge_response != challenge {
        return Err(DeleteError::InvalidChallengeResponse);
    }

    println!("Going to delete user with id {}", user_id);

    db.build_transaction()
        .run(|| {
            db::subscriptions_delete_by_user_id(db, user_id)?;
            db::users_delete_by_id(db, user_id)
        })
        .map_err(|err| {
            DeleteError::Unknown(format!(
                "Failed to delete subscriptions and user {}: {:?}",
                user_id, err,
            ))
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use diesel;
    use diesel::pg::PgConnection;
    use diesel::prelude::*;
    use dockertest::waitfor::{MessageSource, MessageWait};
    use dockertest::{Composition, DockerOperations, DockerTest, PullPolicy, Source};
    use std::rc::Rc;

    fn create_docker() -> DockerTest {
        // Define our test
        let source = Source::DockerHub(PullPolicy::IfNotPresent);
        let mut test = DockerTest::new().with_default_source(source);

        // Define our Composition - the Image we will start and end up as our RunningContainer
        let postgres =
            Composition::with_repository("postgres").with_wait_for(Rc::new(MessageWait {
                message: "database system is ready to accept connections".to_string(),
                source: MessageSource::Stderr,
                timeout: 20,
            }));
        test.add_composition(postgres);
        test
    }

    fn open_connection(ops: &DockerOperations) -> PgConnection {
        let container = ops.handle("postgres").expect("retrieve postgres container");
        let ip = container.ip();
        // This is the default postgres serve port
        let port = "5432";
        let conn_string = format!("postgres://postgres:postgres@{}:{}", ip, port);
        PgConnection::establish(&conn_string).expect("Failed to establish PG connection")
    }

    #[test]
    fn iam_add_user() {
        let docker = create_docker();
        docker.run(|ops| {
            let conn = open_connection(&ops);

            let users_table = "CREATE TABLE users (
  id SERIAL PRIMARY KEY,
  timezone VARCHAR NULL,
  inserted TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);";

            let identities_table = "
CREATE TABLE identities (
  id SERIAL PRIMARY KEY,
  provider VARCHAR NOT NULL, -- eg. 'github'
  pid VARCHAR NOT NULL, -- user's id in that provider
  user_id INT NOT NULL REFERENCES users(id),
  email VARCHAR NOT NULL,
  username VARCHAR NOT NULL,
  inserted TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE(provider, id)
);";

            diesel::sql_query(users_table)
                .execute(&conn)
                .expect("failed to create table users");
            diesel::sql_query(identities_table)
                .execute(&conn)
                .expect("failed to create table identities");

            // SCENARIO 1: Insert new user and identity
            let user_info = ProviderUserInfo {
                provider: "facebook",
                pid: "1094".into(),
                email: "lau@lau.nl".into(),
                username: "Lautje".into(),
            };
            let user = fetch_or_insert_user_in_db(&conn, &user_info)
                .expect("failed: fetch_or_insert_user_in_db: {}");
            assert_eq!(true, user.first_login);
            let identities =
                db::identities_find_by_email_or_id(&conn, "facebook", "1094", "lau@lau.nl")
                    .expect("failed: identities_find_by_email_or_id");
            assert_eq!(1, identities.len());

            // SCENARIO 2a: Insert new user and identity
            let user_info = ProviderUserInfo {
                provider: "facebook",
                pid: "12345".into(),
                email: "reto@reto.com".into(),
                username: "Reto".into(),
            };
            let user = fetch_or_insert_user_in_db(&conn, &user_info)
                .expect("failed: fetch_or_insert_user_in_db: {}");
            assert_eq!(true, user.first_login);
            let identities =
                db::identities_find_by_email_or_id(&conn, "facebook", "12345", "reto@reto.com")
                    .expect("failed: identities_find_by_email_or_id");
            assert_eq!(1, identities.len());

            // SCENARIO 2b: Login unchanged user
            let user = fetch_or_insert_user_in_db(&conn, &user_info)
                .expect("failed: fetch_or_insert_user_in_db: {}");
            assert_eq!(false, user.first_login);
            let identities =
                db::identities_find_by_email_or_id(&conn, "facebook", "12345", "reto@reto.com")
                    .expect("failed: identities_find_by_email_or_id");
            assert_eq!(1, identities.len());

            // SCENARIO 2c: Login with new provider but same e-mail
            let user_info = ProviderUserInfo {
                provider: "github",
                pid: "6789".into(),
                email: "reto@reto.com".into(),
                username: "rethab".into(),
            };
            let user = fetch_or_insert_user_in_db(&conn, &user_info)
                .expect("failed: fetch_or_insert_user_in_db: {}");
            assert_eq!(false, user.first_login);
            let identities =
                db::identities_find_by_email_or_id(&conn, "facebook", "12345", "reto@reto.com")
                    .expect("failed: identities_find_by_email_or_id");
            assert_eq!(2, identities.len());
            let identities =
                db::identities_find_by_email_or_id(&conn, "github", "6789", "reto@reto.com")
                    .expect("failed: identities_find_by_email_or_id");
            assert_eq!(2, identities.len());

            // SCENARIO 2d: User changed e-mail on github
            let user_info = ProviderUserInfo {
                provider: "github",
                pid: "6789".into(),
                email: "new@reto.com".into(),
                username: "rethab".into(),
            };
            let user = fetch_or_insert_user_in_db(&conn, &user_info)
                .expect("failed: fetch_or_insert_user_in_db: {}");
            assert_eq!(false, user.first_login);
            let identities =
                db::identities_find_by_email_or_id(&conn, "github", "6789", "new@reto.com")
                    .expect("failed: identities_find_by_email_or_id");
            assert_eq!(1, identities.len());

            let identities =
                db::identities_find_by_email_or_id(&conn, "facebook", "12345", "reto@reto.com")
                    .expect("failed: identities_find_by_email_or_id");
            assert_eq!(1, identities.len());

            let identities =
                db::identities_find_by_email_or_id(&conn, "facebook", "12345", "new@reto.com")
                    .expect("failed: identities_find_by_email_or_id");
            assert_eq!(2, identities.len());

            // SCENARIO 3a: Insert new user and identity
            let user_info = ProviderUserInfo {
                provider: "facebook",
                pid: "2020".into(),
                email: "marre@imagine.com".into(),
                username: "Marre".into(),
            };
            let user = fetch_or_insert_user_in_db(&conn, &user_info)
                .expect("failed: fetch_or_insert_user_in_db: {}");
            assert_eq!(true, user.first_login);
            let identities =
                db::identities_find_by_email_or_id(&conn, "facebook", "2020", "marre@imagine.com")
                    .expect("failed: identities_find_by_email_or_id");
            assert_eq!(1, identities.len());
            // SCENARIO 3b: User also logs in via github with same e-mail
            let user_info = ProviderUserInfo {
                provider: "github",
                pid: "89421".into(),
                email: "marre@imagine.com".into(),
                username: "marrethecoder".into(),
            };
            let user = fetch_or_insert_user_in_db(&conn, &user_info)
                .expect("failed: fetch_or_insert_user_in_db: {}");
            assert_eq!(false, user.first_login);
            let identities =
                db::identities_find_by_email_or_id(&conn, "github", "89421", "marre@imagine.com")
                    .expect("failed: identities_find_by_email_or_id");
            assert_eq!(2, identities.len());
        })
    }
}
