use async_trait::async_trait;
use futures::{future, stream, Future as StdFuture, FutureExt, Stream as StdStream, TryFutureExt};
use reqwest::{self, header, Method};
use serde::{de::DeserializeOwned, ser::Serialize, Deserialize};
use std::{env, pin::Pin, time::Duration};

mod result;
pub use result::{Error, Result};

pub mod bonuses;
pub mod models;
pub mod users;
pub mod webhooks;

/// A type alias for `Future` that may return `crate::error::Error`
pub type Future<T> = Pin<Box<dyn StdFuture<Output = Result<T>> + Send>>;

/// A type alias for `Stream` that may result in `crate::error::Error`
pub type Stream<T> = Pin<Box<dyn StdStream<Item = Result<T>> + Send>>;
pub use futures::StreamExt;

pub const REQUEST_TIMEOUT: Duration = Duration::from_secs(5);
pub const BASE_URL: &str = "https://bonus.ly/api/v1";
pub const PAGE_SIZE: usize = 20;

/// A utility constant to pass an empty query slice to the various client fetch
/// functions
pub const NO_QUERY: &[&str; 0] = &[""; 0];

#[derive(Debug, Deserialize)]
struct Response<T> {
    success: bool,
    message: Option<String>,
    result: Option<T>,
}

impl<T> From<Response<T>> for Result<T> {
    fn from(v: Response<T>) -> Self {
        match v.success {
            true => Ok(v.result.expect("result data")),
            false => Err(Error::api_error(
                v.message.unwrap_or_else(|| "no message".to_string()),
            )),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Client {
    base_url: String,
    client: reqwest::Client,
}

impl Default for Client {
    fn default() -> Self {
        Self::from_dotenv().expect("client")
    }
}

impl Client {
    pub fn from_dotenv() -> Result<Self> {
        dotenv::dotenv().ok();
        Ok(Self::new(&token_from_env()?))
    }

    pub fn from_env(filename: &str) -> Result<Self> {
        dotenv::from_filename(filename).ok();
        Ok(Self::new(&token_from_env()?))
    }

    /// Create a new bonus.ly client using a given access token
    pub fn new(token: &str) -> Self {
        let mut headers = header::HeaderMap::new();
        let mut token_value = header::HeaderValue::from_str(&format!("Bearer {}", token))
            .expect("valid bearer token");
        token_value.set_sensitive(true);
        headers.insert(header::AUTHORIZATION, token_value);
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .gzip(true)
            .timeout(REQUEST_TIMEOUT)
            .build()
            .expect("reqwest client");
        Self {
            base_url: BASE_URL.to_owned(),
            client,
        }
    }

    fn _get<T, Q, V>(&self, path: &str, query: &Q, add_query: &V) -> Future<T>
    where
        T: 'static + DeserializeOwned + std::marker::Send,
        Q: Serialize + ?Sized,
        V: Serialize + ?Sized,
    {
        let request_url = format!("{}{}", self.base_url, path);
        self.client
            .get(&request_url)
            .query(query)
            .query(add_query)
            .send()
            .map_err(Error::from)
            .and_then(|result| match result.error_for_status() {
                Ok(result) => {
                    let fut: Future<T> = result
                        .json::<Response<T>>()
                        .map_err(Error::from)
                        .and_then(|response| async { Result::from(response) })
                        .boxed();
                    fut
                }
                Err(e) => future::err(Error::from(e)).boxed(),
            })
            .boxed()
    }

    pub(crate) fn get<T, Q>(&self, path: &str, query: &Q) -> Future<T>
    where
        T: 'static + DeserializeOwned + std::marker::Send,
        Q: Serialize + ?Sized + std::marker::Sync,
    {
        self._get(path, query, NO_QUERY)
    }

    pub(crate) fn get_stream<E, Q>(&self, path: &str, limit: usize, query: &'static Q) -> Stream<E>
    where
        E: 'static + DeserializeOwned + std::marker::Send,
        Q: Serialize + ?Sized + std::marker::Sync,
    {
        let path = path.to_string();
        let client = self.clone();
        client
            ._get::<Vec<E>, _, _>(&path.clone(), query, &[("limit", limit)])
            .map_ok(move |mut data| {
                data.reverse();
                let data_len = data.len();
                let path = path.to_string();
                stream::try_unfold(
                    (data, path, client, data_len),
                    move |(mut data, path, client, skip)| async move {
                        match data.pop() {
                            Some(entry) => Ok(Some((entry, (data, path, client, skip)))),
                            None => {
                                //loop until we find next bit of data or run
                                // out of cursors
                                let mut data: Vec<E> = client
                                    ._get::<Vec<E>, _, _>(
                                        &path,
                                        query,
                                        &[("skip", skip), ("limit", limit)],
                                    )
                                    .await?;
                                if !data.is_empty() {
                                    data.reverse();
                                    let data_len = data.len();
                                    let entry = data.pop().unwrap();
                                    Ok(Some((entry, (data, path, client, skip + data_len))))
                                } else {
                                    Ok(None)
                                }
                            }
                        }
                    },
                )
            })
            .try_flatten_stream()
            .boxed()
    }

    pub(crate) fn put<T, R>(&self, path: &str, json: &T) -> Future<R>
    where
        T: Serialize + ?Sized,
        R: 'static + DeserializeOwned + std::marker::Send,
    {
        self.submit(Method::PUT, path, json)
    }

    pub(crate) fn post<T, R>(&self, path: &str, json: &T) -> Future<R>
    where
        T: Serialize + ?Sized,
        R: 'static + DeserializeOwned + std::marker::Send,
    {
        self.submit(Method::POST, path, json)
    }

    fn submit<T, R>(&self, method: Method, path: &str, json: &T) -> Future<R>
    where
        T: Serialize + ?Sized,
        R: 'static + DeserializeOwned + std::marker::Send,
    {
        let request_url = format!("{}{}", self.base_url, path);
        self.client
            .request(method, &request_url)
            .json(json)
            .send()
            .map_err(Error::from)
            .and_then(|response| match response.error_for_status() {
                Ok(result) => {
                    let fut: Future<R> = result
                        .json::<Response<R>>()
                        .map_err(Error::from)
                        .and_then(|response| async { Result::from(response) })
                        .boxed();
                    fut
                }
                Err(e) => future::err(Error::from(e)).boxed(),
            })
            .boxed()
    }

    pub(crate) fn delete<R>(&self, path: &str) -> Future<R>
    where
        R: 'static + DeserializeOwned + std::marker::Send,
    {
        let request_url = format!("{}{}", self.base_url, path);
        self.client
            .delete(&request_url)
            .send()
            .map_err(Error::from)
            .and_then(|response| match response.error_for_status() {
                Ok(result) => {
                    let fut: Future<R> = result
                        .json::<Response<R>>()
                        .map_err(Error::from)
                        .and_then(|response| async { Result::from(response) })
                        .boxed();
                    fut
                }
                Err(e) => future::err(Error::from(e)).boxed(),
            })
            .boxed()
    }
}

fn token_from_env() -> Result<String> {
    env_var("BONUSLY_TOKEN")
}

pub(crate) fn env_var<T: std::str::FromStr>(name: &str) -> Result<T> {
    env::var(name)
        .map_err(|_| Error::custom(format!("Missing env var: {}", name)))
        .and_then(|var| {
            var.parse::<T>()
                .map_err(|_| Error::custom(format!("Error parsing {}", name)))
        })
}

impl<T: ?Sized> IntoVec for T where T: StdStream {}

#[async_trait]
pub trait IntoVec: StreamExt {
    async fn into_vec<T>(self) -> Result<Vec<T>>
    where
        Self: Sized,
        T: std::marker::Send,
        Vec<Result<T>>: Extend<Self::Item>,
    {
        self.collect::<Vec<Result<T>>>().await.into_iter().collect()
    }
}
