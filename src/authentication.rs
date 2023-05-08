use eyre::Result;
use regex::Regex;

use crate::store::Store;

pub enum AuthenticationStrategy {
    NoAuthentication,
    TokenList(Store),
}

const TOKEN_REGEX: &str = r"^!token\s+(.+)$";

impl AuthenticationStrategy {
    pub fn auth_required(&self) -> bool {
        match self {
            AuthenticationStrategy::NoAuthentication => false,
            AuthenticationStrategy::TokenList(_) => true,
        }
    }

    /// Checks whether a user is authenticated,
    pub async fn is_user_authenticated(&self, user_id: &str) -> Result<bool> {
        match self {
            AuthenticationStrategy::NoAuthentication => Ok(true),
            AuthenticationStrategy::TokenList(store) => store.is_allocated(user_id).await,
        }
    }

    pub async fn authenticate(&self, user_id: &str, msg: &str) -> Result<bool> {
        match self {
            AuthenticationStrategy::NoAuthentication => Ok(true),
            AuthenticationStrategy::TokenList(store) => {
                let token_regex =
                    Regex::new(TOKEN_REGEX).expect("implementation error - invalid regex");
                if let Some(cap) = token_regex.captures(&msg) {
                    let token = &cap[1];
                    if store.is_token_valid(token).await? {
                        store.allocate(user_id, token).await?;
                        Ok(true)
                    } else {
                        Ok(false)
                    }
                } else {
                    Ok(false)
                }
            }
        }
    }
}
