use http_body_util::{BodyExt, Empty};
use hyper::{body::Bytes, Request, StatusCode};
use std::{env, str};
use tracing::debug;

const SERVER_DOMAIN: &str = "x.com";
const ROUTE: &str = "i/api/1.1/"; //fix url (hint: figure out the graphql route)
const USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.0 Safari/537.36";

const NOTARY_HOST: &str = "127.0.0.1";
const NOTARY_PORT: u16 = 7047;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // Load secret variables from environment for twitter server connection
    dotenv::dotenv().ok();
    let post_id = env::var("POST_ID").unwrap();
    let auth_token = env::var("AUTH_TOKEN").unwrap();
    let access_token = env::var("ACCESS_TOKEN").unwrap();
    let csrf_token = env::var("CSRF_TOKEN").unwrap();

    // todo: fill in gaps (hint: set up connection to notary server)

    let request = Request::builder()
        .uri(format!("https://{SERVER_DOMAIN}/{ROUTE}/{post_id}.json"))
        .header("Host", SERVER_DOMAIN)
        .header("Accept", "*/*")
        .header("Accept-Encoding", "identity")
        .header("Connection", "close")
        .header("User-Agent", USER_AGENT)
        .header("Authorization", format!("Bearer {access_token}"))
        .header(
            "Cookie",
            format!("auth_token={auth_token}; ct0={csrf_token}"),
        )
        .header("Authority", SERVER_DOMAIN)
        .header("X-Twitter-Auth-Type", "OAuth2Session")
        .header("x-twitter-active-user", "yes")
        .header("X-Csrf-Token", csrf_token.clone())
        .body(Empty::<Bytes>::new())
        .unwrap();

    debug!("Sending request");

    // todo: complete notary request/session
}
