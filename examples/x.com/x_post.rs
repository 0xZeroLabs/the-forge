use http_body_util::{BodyExt, Empty};
use hyper::{body::Bytes, Request, StatusCode};
use hyper_util::rt::TokioIo;
use notary_client::{Accepted, NotarizationRequest, NotaryClient};
use std::{env, str};
use tlsn_core::{commitment::CommitmentKind, proof::TlsProof};
use tlsn_prover::tls::{Prover, ProverConfig};
use tokio::io::AsyncWriteExt as _;
use tokio_util::compat::{FuturesAsyncReadCompatExt, TokioAsyncReadCompatExt};
use tracing::debug;

const SERVER_DOMAIN: &str = "x.com";
const ROUTE: &str = "i/api/1.1/"; //fix url (hint: figure out the graphql route)
const USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.0 Safari/537.36";

const NOTARY_HOST: &str = "127.0.0.1:7047";

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // Load secret variables from environment for twitter server connection
    dotenv::dotenv().ok();
    let post_id = env::var("POST_ID").unwrap();
    let auth_token = env::var("AUTH_TOKEN").unwrap();
    let access_token = env::var("ACCESS_TOKEN").unwrap();
    let csrf_token = env::var("CSRF_TOKEN").unwrap();

    let notary_client = NotaryClient::builder().host(NOTARY_HOST).build().unwrap();
    let notarization_request = NotarizationRequest::builder().build().unwrap();

    let Accepted {
        io: notary_connection,
        id: session_id,
        ..
    } = notary_client
        .request_notarization(notarization_request)
        .await
        .unwrap();

    let prover_config = ProverConfig::builder()
        .id(session_id)
        .server_dns(SERVER_DOMAIN)
        .build()
        .unwrap();
    let prover = Prover::new(prover_config)
        .setup(notary_connection.compat())
        .await
        .unwrap();

    let client_socket = tokio::net::TcpStream::connect((SERVER_DOMAIN, 443))
        .await
        .unwrap();

    let (tls_connection, prover_fut) = prover.connect(client_socket.compat()).await.unwrap();
    let tls_connection = TokioIo::new(tls_connection.compat());

    let prover_ctrl = prover_fut.control();

    let prover_task = tokio::spawn(prover_fut);

    let (mut request_sender, connection) = hyper::client::conn::http1::handshake(tls_connection)
        .await
        .unwrap();

    tokio::spawn(connection);

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

    prover_ctrl.defer_decryption().await.unwrap();

    let response = request_sender.send_request(request).await.unwrap();

    debug!("Sent request");

    assert!(response.status() == StatusCode::OK, "{}", response.status());

    debug!("Request OK");
}
