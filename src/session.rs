use crate::config::project_dirs;
use chrono::{DateTime, Duration, Utc};
use std::fs::{self, create_dir_all};

use anyhow::{Context, Ok};
use oauth2::{CsrfToken, PkceCodeChallenge, PkceCodeVerifier};
use regex::Regex;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use unescaper::unescape;

const USER_AGENT: &'static str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/133.0.0.0 Safari/537.36";
const MLB_OKTA_URL: &'static str = "https://www.mlbstatic.com/mlb.com/vendor/mlb-okta/mlb-okta.js";
const OKTA_AUTHORIZE_URL: &'static str =
    "https://ids.mlb.com/oauth2/aus1m088yK07noBfh356/v1/authorize";
const OKTA_TOKEN_URL: &'static str = "https://ids.mlb.com/oauth2/aus1m088yK07noBfh356/v1/token";

pub struct MlbSession<State> {
    pub client: reqwest::Client,
    pub state: State,
}

pub struct Unauthenticated;

pub struct Authenticated {
    pub authn: AuthnResponse,
}

pub struct OktaCodeReceived {
    pub client_id: String,
    pub okta_code: String,
    pub pkce_verifier: PkceCodeVerifier,
}

pub struct Authorized {
    pub okta_tokens: OktaAuthResponse,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthnResponse {
    // pub expires_at: String,
    // pub status: String,
    pub session_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OktaAuthResponse {
    pub token_type: String,
    pub expires_in: i64,
    pub access_token: String,
    pub scope: String,
    pub id_token: String,
    pub expires_at: Option<DateTime<Utc>>, // Not from API.
}

impl OktaAuthResponse {
    pub fn save(&self) -> anyhow::Result<()> {
        let cache_dir = project_dirs().cache_dir().to_path_buf();
        let token_file = cache_dir.join("token.json");
        create_dir_all(&cache_dir)?;

        let json = serde_json::to_string_pretty(self)?;
        fs::write(&token_file, json).context("Failed to save token to file.")?;
        println!("Saved okta auth token to {:?}", cache_dir);

        Ok(())
    }

    pub fn load() -> anyhow::Result<Option<Self>> {
        let token_file = project_dirs().cache_dir().join("token.json");
        if !token_file.exists() {
            return Ok(None);
        }
        let data = fs::read_to_string(&token_file)?;
        let token: Self = serde_json::from_str(&data)?;

        Ok(Some(token))
    }

    pub fn is_valid(&self) -> bool {
        // Consider token invalid if it'll expire in the next 60s
        let now = Utc::now();
        let buffer = Duration::seconds(60);

        if let Some(expires_at) = self.expires_at {
            expires_at > now + buffer
        } else {
            false
        }
    }
}

impl MlbSession<Unauthenticated> {
    pub fn new() -> anyhow::Result<Self> {
        let client = Client::builder().user_agent(USER_AGENT).build()?;

        Ok(Self {
            client,
            state: Unauthenticated,
        })
    }

    pub async fn authenticate(
        self,
        username: &str,
        password: &str,
    ) -> anyhow::Result<MlbSession<Authenticated>> {
        let req_body = json!({
            "username": username,
            "password": password,
            "options": {
                "multiOptionalFactorEnroll": false,
                "warnBeforePasswordExpired": true
            }
        });

        // TODO: Handle invalid authentication / bad credentials.
        let res = self
            .client
            .post("https://ids.mlb.com/api/v1/authn")
            .header("Content-Type", "application/json")
            .json(&req_body)
            .send()
            .await?;

        let authn = res.json().await?;

        Ok(MlbSession {
            client: self.client,
            state: Authenticated { authn },
        })
    }

    pub async fn login_and_authorize(
        self,
        username: &str,
        password: &str,
    ) -> anyhow::Result<MlbSession<Authorized>> {
        if let Some(cached_token) = OktaAuthResponse::load()? {
            if cached_token.is_valid() {
                println!("Loaded existing token from cache.");
                return Ok(MlbSession {
                    client: self.client,
                    state: Authorized {
                        okta_tokens: cached_token
                    },
                });
            }
        }

        let session = self.authenticate(username, password).await?;
        let session = session.fetch_okta_code().await?;
        let mut session = session.exchange_code_for_token().await?;

        session.state.okta_tokens.expires_at =
            Some(Utc::now() + Duration::seconds(session.state.okta_tokens.expires_in));
        session.state.okta_tokens.save()?;
        Ok(session)
    }
}

impl MlbSession<Authenticated> {
    async fn fetch_client_id(&self) -> anyhow::Result<String> {
        let res = self.client.get(MLB_OKTA_URL).send().await?;
        let res_body = res.text().await?;

        // Capture the value after production:{clientId:" and before the next "
        let re = Regex::new(r#"production:\{clientId:"([^"]+)","#)?;
        if let Some(caps) = re.captures(&res_body) {
            if let Some(client_id) = caps.get(1) {
                return Ok(client_id.as_str().to_string());
            }
        }
        anyhow::bail!("clientId not found in OKTA JS")
    }

    pub async fn fetch_okta_code(self) -> anyhow::Result<MlbSession<OktaCodeReceived>> {
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
        let state = CsrfToken::new_random_len(48);
        let nonce = CsrfToken::new_random_len(48);

        let client_id = self.fetch_client_id().await?;

        let res = self
            .client
            .get(OKTA_AUTHORIZE_URL)
            .query(&[
                ("client_id", client_id.as_str()),
                ("response_type", "code"),
                ("response_mode", "okta_post_message"),
                ("scope", "openid profile email"),
                ("redirect_uri", "https://www.mlb.com/login"),
                ("state", &state.secret()),
                ("nonce", &nonce.secret()),
                ("code_challenge", pkce_challenge.as_str()),
                ("code_challenge_method", "S256"),
                ("sessionToken", self.state.authn.session_token.as_str()),
            ])
            .send()
            .await?;

        let res_body = res.text_with_charset("utf-8").await?;

        // look for a line like: data.code = 'an_okta_code_in_single_quotes';
        let re = Regex::new(r#"data\.code\s*=\s*'([^']+)'"#)?;
        if let Some(caps) = re.captures(&res_body) {
            if let Some(m) = caps.get(1) {
                let okta_code = unescape(m.as_str())?;
                return Ok(MlbSession {
                    client: self.client,
                    state: OktaCodeReceived {
                        client_id,
                        okta_code,
                        pkce_verifier,
                    },
                });
            }
        }
        anyhow::bail!("Authorization code not found in okta_post_message response")
    }
}

impl MlbSession<OktaCodeReceived> {
    pub async fn exchange_code_for_token(self) -> anyhow::Result<MlbSession<Authorized>> {
        let res = self
            .client
            .post(OKTA_TOKEN_URL)
            .header("Accept", "application/json")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&[
                ("client_id", self.state.client_id.as_str()),
                ("redirect_uri", "https://www.mlb.com/login"),
                ("grant_type", "authorization_code"),
                ("code_verifier", self.state.pkce_verifier.secret()),
                ("code", self.state.okta_code.as_str()),
            ])
            .send()
            .await?;

        let res_body: OktaAuthResponse = res.json().await?;

        Ok(MlbSession {
            client: self.client,
            state: Authorized {
                okta_tokens: res_body,
            },
        })
    }
}
