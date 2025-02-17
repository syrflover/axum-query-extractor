use std::{
    borrow::Cow,
    fmt::Debug,
    ops::{Deref, DerefMut},
};

use axum::{
    body::Body,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
};
use serde::de::DeserializeOwned;

#[derive(Debug, thiserror::Error)]
pub enum QueryRejection {
    #[error("Deserialize querystring: {0}")]
    Deserialize(serde_qs::Error),
}

impl IntoResponse for QueryRejection {
    fn into_response(self) -> axum::response::Response {
        Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(self.to_string()))
            .unwrap()
    }
}

pub struct Query<T>(pub T);

impl<T> Deref for Query<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Query<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: Debug> Debug for Query<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Query").field(&self.0).finish()
    }
}

impl<T: Clone> Clone for Query<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: Copy> Copy for Query<T> {}

impl<T: Default> Default for Query<T> {
    fn default() -> Self {
        Self(T::default())
    }
}

impl<T> TryFrom<&str> for Query<T>
where
    T: DeserializeOwned,
{
    type Error = QueryRejection;

    fn try_from(query: &str) -> Result<Self, Self::Error> {
        let value = serde_qs::from_str(query).map_err(QueryRejection::Deserialize)?;
        Ok(Query(value))
    }
}

impl<T> TryFrom<Cow<'_, str>> for Query<T>
where
    T: DeserializeOwned,
{
    type Error = QueryRejection;

    fn try_from(query: Cow<str>) -> Result<Self, Self::Error> {
        match query {
            Cow::Borrowed(query) => query.try_into(),
            Cow::Owned(query) => query.as_str().try_into(),
        }
    }
}

impl<T, S> FromRequestParts<S> for Query<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = QueryRejection;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let query = parts
            .uri
            .query()
            .and_then(|query| urlencoding::decode(query).ok())
            .unwrap_or_default();
        query.try_into()
    }
}
