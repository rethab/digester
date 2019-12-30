use super::channel::*;

use chrono::{DateTime, Utc};
use github_rs::client::{Executor, Github};
use github_rs::StatusCode;
use serde::Deserialize;
use serde_json::Value;
use std::convert::TryInto;

pub struct GithubRelease {
    client: Github,
}

impl GithubRelease {
    pub fn new(api_token: &str) -> Result<GithubRelease, String> {
        let github = Github::new(api_token)
            .map_err(|err| format!("Failed to initialize github client: {:?}", err))?;
        Ok(GithubRelease { client: github })
    }

    fn split_name(name: &str) -> Result<(&str, &str), String> {
        let parts: Vec<&str> = name.split('/').collect();
        if parts.len() != 2 {
            return Err("Repository must have format owner/repository".into());
        }
        let owner = parts[0];
        let repo = parts[1];
        Ok((owner, repo))
    }

    fn parse_releases_response(
        json: Value,
        last_fetched: Option<DateTime<Utc>>,
    ) -> Result<Vec<Update>, String> {
        let cloned_json = json.clone();
        let releases = serde_json::from_value::<Vec<ReleaseResponse>>(json)
            .map_err(|err| format!("Failed to parse releases: {:?}, json: {}", err, cloned_json))?;
        let keep = |update: &Update| last_fetched.map(|lf| lf < update.published).unwrap_or(true);
        let mut updates = Vec::with_capacity(releases.len());
        for release in releases {
            match release.try_into() {
                Ok(update) if keep(&update) => updates.push(update),
                Ok(_) => {}
                Err(err) => return Err(format!("Failed to parse reponse: {}", err)),
            }
        }
        Ok(updates)
    }
}

#[derive(Deserialize, Debug)]
struct RepoResponse {
    full_name: String,
}

#[derive(Deserialize, Debug)]
struct ReleaseResponse {
    html_url: String,
    // name is not required. In that case we take the tag_name, which is required
    name: Option<String>,
    tag_name: String,
    published_at: String,
}

impl TryInto<Update> for ReleaseResponse {
    type Error = String;
    fn try_into(self) -> Result<Update, Self::Error> {
        let published = DateTime::parse_from_rfc3339(&self.published_at)
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(|err| {
                format!(
                    "Failed to parse {} as rfc3339: {:?}",
                    self.published_at, err
                )
            })?;
        let title = self
            .name
            .and_then(|name| {
                let trimmed = name.trim();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed.into())
                }
            })
            .unwrap_or(self.tag_name);

        Ok(Update {
            title,
            url: self.html_url,
            published,
        })
    }
}

impl Channel for GithubRelease {
    fn validate(&self, name: &str) -> Result<String, ValidationError> {
        let (owner, repo) = Self::split_name(name).map_err(ValidationError::ChannelInvalid)?;

        let query = self.client.get().repos().owner(owner).repo(repo);

        // todo handle rate limiting
        match query.execute::<Value>() {
            Ok((_, status, Some(json))) if status == StatusCode::OK => {
                serde_json::from_value::<RepoResponse>(json)
                    .map(|repo| repo.full_name)
                    .map_err(|_| ValidationError::TechnicalError) // todo log
            }
            Ok((_, status, _)) if status == StatusCode::NOT_FOUND => {
                Err(ValidationError::ChannelNotFound)
            }
            _ => Err(ValidationError::TechnicalError), // todo log
        }
    }
    fn fetch_updates(
        &self,
        name: &str,
        last_fetched: Option<DateTime<Utc>>,
    ) -> Result<Vec<Update>, String> {
        let (owner, repo) = Self::split_name(name)?;
        let query = self.client.get().repos().owner(owner).repo(repo).releases();
        match query.execute::<Value>() {
            Ok((_, status, Some(json))) if status == StatusCode::OK => {
                GithubRelease::parse_releases_response(json, last_fetched)
            }
            other => Err(format!("Failed to fetch: {:?}", other)),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const RELEASES_RESPONSE: &'static str = r#"[{"assets":[{"browser_download_url":"https://github.com/kubernetes/kubernetes/releases/download/v1.14.10/kubernetes.tar.gz","content_type":"application/x-compressed","created_at":"2019-12-11T18:40:52Z","download_count":21,"id":16739058,"label":"","name":"kubernetes.tar.gz","node_id":"MDEyOlJlbGVhc2VBc3NldDE2NzM5MDU4","size":646911,"state":"uploaded","updated_at":"2019-12-11T18:40:52Z","uploader":{"avatar_url":"https://avatars1.githubusercontent.com/u/33505452?v=4","events_url":"https://api.github.com/users/k8s-release-robot/events{/privacy}","followers_url":"https://api.github.com/users/k8s-release-robot/followers","following_url":"https://api.github.com/users/k8s-release-robot/following{/other_user}","gists_url":"https://api.github.com/users/k8s-release-robot/gists{/gist_id}","gravatar_id":"","html_url":"https://github.com/k8s-release-robot","id":33505452,"login":"k8s-release-robot","node_id":"MDQ6VXNlcjMzNTA1NDUy","organizations_url":"https://api.github.com/users/k8s-release-robot/orgs","received_events_url":"https://api.github.com/users/k8s-release-robot/received_events","repos_url":"https://api.github.com/users/k8s-release-robot/repos","site_admin":false,"starred_url":"https://api.github.com/users/k8s-release-robot/starred{/owner}{/repo}","subscriptions_url":"https://api.github.com/users/k8s-release-robot/subscriptions","type":"User","url":"https://api.github.com/users/k8s-release-robot"},"url":"https://api.github.com/repos/kubernetes/kubernetes/releases/assets/16739058"}],"assets_url":"https://api.github.com/repos/kubernetes/kubernetes/releases/22154714/assets","author":{"avatar_url":"https://avatars1.githubusercontent.com/u/33505452?v=4","events_url":"https://api.github.com/users/k8s-release-robot/events{/privacy}","followers_url":"https://api.github.com/users/k8s-release-robot/followers","following_url":"https://api.github.com/users/k8s-release-robot/following{/other_user}","gists_url":"https://api.github.com/users/k8s-release-robot/gists{/gist_id}","gravatar_id":"","html_url":"https://github.com/k8s-release-robot","id":33505452,"login":"k8s-release-robot","node_id":"MDQ6VXNlcjMzNTA1NDUy","organizations_url":"https://api.github.com/users/k8s-release-robot/orgs","received_events_url":"https://api.github.com/users/k8s-release-robot/received_events","repos_url":"https://api.github.com/users/k8s-release-robot/repos","site_admin":false,"starred_url":"https://api.github.com/users/k8s-release-robot/starred{/owner}{/repo}","subscriptions_url":"https://api.github.com/users/k8s-release-robot/subscriptions","type":"User","url":"https://api.github.com/users/k8s-release-robot"},"body":"See [kubernetes-announce@](https://groups.google.com/forum/#!forum/kubernetes-announce) and [CHANGELOG-1.14.md](https://github.com/kubernetes/kubernetes/blob/master/CHANGELOG-1.14.md#v11410) for details.\n\nSHA256 for `kubernetes.tar.gz`: `4d3bba77de6509325123b8f50c23eaf99a75f736471f75dba0fc237128334382`\nSHA512 for `kubernetes.tar.gz`: `b2b73d186769461236f94b7d1faa5d5806534bae5d9404f223f3e6aeaf1bc7a0c3bc505e2b8f3d34cec12d6657385927d82e67488f93ffde83c68239d563646d`\n\nAdditional binary downloads are linked in the [CHANGELOG-1.14.md](https://github.com/kubernetes/kubernetes/blob/master/CHANGELOG-1.14.md#downloads-for-v11410).","created_at":"2019-12-11T12:10:22Z","draft":false,"html_url":"https://github.com/kubernetes/kubernetes/releases/tag/v1.14.10","id":22154714,"name":"v1.14.10","node_id":"MDc6UmVsZWFzZTIyMTU0NzE0","prerelease":false,"published_at":"2019-12-11T18:40:51Z","tag_name":"v1.14.10","tarball_url":"https://api.github.com/repos/kubernetes/kubernetes/tarball/v1.14.10","target_commitish":"release-1.14","upload_url":"https://uploads.github.com/repos/kubernetes/kubernetes/releases/22154714/assets{?name,label}","url":"https://api.github.com/repos/kubernetes/kubernetes/releases/22154714","zipball_url":"https://api.github.com/repos/kubernetes/kubernetes/zipball/v1.14.10"}]"#;

    #[test]
    fn parse_one_release() {
        let val: Value = serde_json::from_str(RELEASES_RESPONSE).expect("Failed to parse json");
        let updates = GithubRelease::parse_releases_response(val, None)
            .expect("Failed to parse into updates");
        assert_eq!(1, updates.len())
    }

    #[test]
    fn take_new_release() {
        let val: Value = serde_json::from_str(RELEASES_RESPONSE).expect("Failed to parse json");
        let updates = GithubRelease::parse_releases_response(
            val,
            Some(
                DateTime::parse_from_rfc3339("2019-12-11T18:40:50Z")
                    .unwrap()
                    .with_timezone(&Utc),
            ), // one second before the release was published
        )
        .expect("Failed to parse into updates");
        assert_eq!(1, updates.len())
    }

    #[test]
    fn ignore_old_release() {
        let val: Value = serde_json::from_str(RELEASES_RESPONSE).expect("Failed to parse json");
        let updates = GithubRelease::parse_releases_response(val, Some(Utc::now()))
            .expect("Failed to parse into updates");
        assert_eq!(0, updates.len())
    }
}
