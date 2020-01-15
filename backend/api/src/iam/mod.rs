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
