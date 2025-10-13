use crate::config::AppConfig;
use anyhow::Ok;
use oauth2::{CsrfToken, PkceCodeChallenge, PkceCodeVerifier};
use regex::Regex;
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;
use unescaper::unescape;

const USER_AGENT: &'static str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/133.0.0.0 Safari/537.36";
const MLB_OKTA_URL: &'static str = "https://www.mlbstatic.com/mlb.com/vendor/mlb-okta/mlb-okta.js";
const OKTA_AUTHORIZE_URL: &'static str =
    "https://ids.mlb.com/oauth2/aus1m088yK07noBfh356/v1/authorize";
const OKTA_TOKEN_URL: &'static str = "https://ids.mlb.com/oauth2/aus1m088yK07noBfh356/v1/token";

pub struct MlbSession {
    client: reqwest::Client,
    pub authn: Option<AuthnResponse>,
    pub client_id: Option<String>,
    pub okta_code: Option<String>,
    pub okta_tokens: Option<OktaAuthResponse>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthnResponse {
    // pub expires_at: String,
    // pub status: String,
    pub session_token: String,
}

#[derive(Debug, Deserialize)]
pub struct OktaAuthResponse {
    // pub token_type: String,
    // pub expires_in: u32,
    pub access_token: String,
    // pub scope: String,
    // pub id_token: String,
}

impl MlbSession {
    pub fn new() -> anyhow::Result<Self> {
        let client = Client::builder().user_agent(USER_AGENT).build()?;
        Ok(Self { client, authn: None, client_id: None, okta_code: None, okta_tokens: None })
    }

    pub async fn login(&mut self) -> anyhow::Result<()> {
        let cfg = AppConfig::load()?;

        let req_body = json!({
            "username": cfg.credentials.username,
            "password": cfg.credentials.password,
            "options": {
                "multiOptionalFactorEnroll": false,
                "warnBeforePasswordExpired": true
            }
        });

        let res = self.client
            .post("https://ids.mlb.com/api/v1/authn")
            .header("Content-Type", "application/json")
            .json(&req_body)
            .send()
            .await?;

        // TODO: React accordingly if this fails. Know from personal experience that
        // it generates a 403 error if you attempt to login while connected to a VPN.
        let res_body: AuthnResponse = res.json().await?;
        self.authn = Some(res_body);
        Ok(())
    }

    pub async fn fetch_client_id(&mut self) -> anyhow::Result<()> {
        let res = self.client.get(MLB_OKTA_URL).send().await?;
        let res_body = res.text().await?;

        // Capture the value after production:{clientId:" and before the next "
        let re = Regex::new(r#"production:\{clientId:"([^"]+)","#)?;
        if let Some(caps) = re.captures(&res_body) {
            if let Some(client_id) = caps.get(1) {
                self.client_id = Some(client_id.as_str().to_string());
                return Ok(())
                // return Ok(client_id.as_str().to_string());
            }
        }
        anyhow::bail!("clientId not found in OKTA JS")
    }

    pub async fn fetch_okta_code(&mut self, pkce_challenge: &PkceCodeChallenge) -> anyhow::Result<()> {
        let state = CsrfToken::new_random_len(48);
        let nonce = CsrfToken::new_random_len(48);

        let res = self.client
            .get(OKTA_AUTHORIZE_URL)
            .query(&[
                ("client_id", self.client_id.as_deref().expect("do a thing")),
                ("response_type", "code"),
                ("response_mode", "okta_post_message"),
                ("scope", "openid profile email"),
                ("redirect_uri", "https://www.mlb.com/login"),
                ("state", &state.secret()),
                ("nonce", &nonce.secret()),
                ("code_challenge", pkce_challenge.as_str()),
                ("code_challenge_method", "S256"),
                ("sessionToken", self.authn.as_ref().expect("authn must be set").session_token.as_str()),
            ])
            .send()
            .await?;

        let res_body = res.text_with_charset("utf-8").await?;

        // look for a line like: data.code = 'an_okta_code_in_single_quotes';
        let re = Regex::new(r#"data\.code\s*=\s*'([^']+)'"#)?;
        if let Some(caps) = re.captures(&res_body) {
            if let Some(m) = caps.get(1) {
                self.okta_code = Some(unescape(m.as_str())?);
                return Ok(());
            }
        }
        anyhow::bail!("Authorization code not found in okta_post_message response")
    }

    pub async fn fetch_okta_token(&mut self, pkce_verifier: &PkceCodeVerifier) -> anyhow::Result<()> {
        let res = self.client
            .post(OKTA_TOKEN_URL)
            .header("Accept", "application/json")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&[
                ("client_id", self.client_id.as_deref().expect("do a thing")),
                ("redirect_uri", "https://www.mlb.com/login"),
                ("grant_type", "authorization_code"),
                ("code_verifier", pkce_verifier.secret()),
                ("code", self.okta_code.as_deref().expect("okta code must be set")),
            ])
            .send()
            .await?;

        let res_body: OktaAuthResponse = res.json().await?;
        self.okta_tokens = Some(res_body);
        Ok(())
    }

    pub async fn authenticate(&mut self) -> anyhow::Result<()> {
        let (pkce_challenge, pkce_verifier) = oauth2::PkceCodeChallenge::new_random_sha256();

        self.login().await?; // populates self.authn with a response containing a session token
        self.fetch_client_id().await?; // populates self.client_id
        self.fetch_okta_code(&pkce_challenge).await?; // populates self.okta_code
        self.fetch_okta_token(&pkce_verifier).await?; // populates self.okta_tokens

        let token = self.okta_tokens.as_ref().expect("fart frick heck!");

        println!("{}", token.access_token);
        Ok(())
    }
}
