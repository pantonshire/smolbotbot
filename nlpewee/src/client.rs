use crate::language::Language;
use crate::data::*;
use crate::deserialize::Deserialize;
use crate::protocol;
use crate::error::{ConnectionResult, ConnectionError, RequestResult};

use tonic::Request;
use tonic::transport::{Endpoint, Channel};
use chrono::Duration;

pub struct ClientBuilder {
  par_scheme: String,
  par_host: String,
  par_port: u32,
  par_request_timeout: Duration,
  par_concurrency_limit: Option<usize>,
}

impl ClientBuilder {
    pub fn new() -> ClientBuilder {
        ClientBuilder{
            par_scheme: "http".to_owned(),
            par_host: "localhost".to_owned(),
            par_port: 8000,
            par_request_timeout: Duration::zero(),
            par_concurrency_limit: None,
        }
    }

    pub async fn connect(self) -> ConnectionResult<Client> {
        let uri = format!("{}://{}:{}", self.par_scheme, self.par_host, self.par_port);
        let mut ep = Endpoint::from_shared(uri)
            .map_err(|_| ConnectionError::InvalidUri)?;

        if self.par_request_timeout > Duration::zero() {
            ep = ep.timeout(self.par_request_timeout.to_std().unwrap());
        }

        if let Some(concurrency_limit) = self.par_concurrency_limit {
            ep = ep.concurrency_limit(concurrency_limit);
        }

        let channel = ep.connect().await?;

        Ok(Client{
            grpc_client: protocol::nl_pewee_client::NlPeweeClient::new(channel),
        })
    }

    pub fn scheme(&mut self, scheme: String) -> &mut Self {
        self.par_scheme = scheme;
        self
    }

    pub fn host(&mut self, host: String) -> &mut Self {
        self.par_host = host;
        self
    }

    pub fn port(&mut self, port: u32) -> &mut Self {
        self.par_port = port;
        self
    }

    pub fn socket(&mut self, host: String, port: u32) -> &mut Self {
        self.host(host)
            .port(port)
    }

    pub fn request_timeout(&mut self, timeout: Duration) -> &mut Self {
        self.par_request_timeout = timeout;
        self
    }

    pub fn concurrency_limit(&mut self, limit: usize) -> &mut Self {
        self.par_concurrency_limit = Some(limit);
        self
    }
}

#[derive(Clone)]
pub struct Client {
    grpc_client: protocol::nl_pewee_client::NlPeweeClient<Channel>,
}

impl Client {
    pub async fn tokenize(&mut self, text: String, lang: Language) -> RequestResult<Vec<Sentence>> {
        let request = Request::new(protocol::TokenizeRequest{
            text,
            language: lang.serialize_i32()
        });
        let response = self.grpc_client
            .tokenize(request)
            .await?
            .into_inner();
        let sentences = response.sentences.deserialize()?;
        Ok(sentences)
    }

    pub async fn tokenize_ref(&mut self, text: &str, lang: Language) -> RequestResult<Vec<Sentence>> {
        self.tokenize(text.to_owned(), lang).await
    }
}
