use std::env;

#[derive(Debug)]
pub struct GitLabCredentials {
    url: String,
    token: String,
}

impl GitLabCredentials {
    pub fn new(url: String, token: String) -> GitLabCredentials {
        GitLabCredentials { url, token }
    }

    pub fn from_env() -> GitLabCredentials {
        // TODO: handle errors
        let url = env::var("GITLAB_URL").unwrap_or("https://gitlab.com".to_string());
        let token = env::var("GITLAB_TOKEN").unwrap();
        GitLabCredentials::new(url, token)
    }
}
