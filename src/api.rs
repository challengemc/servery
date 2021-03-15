use crate::{db::ServerDb, server::NewServer, AppState};
use anyhow::Result;
use log::error;
use std::{convert::Infallible, sync::Arc};
use tokio_stream::StreamExt;
use warp::{
    body,
    hyper::StatusCode,
    path,
    reject::{self, Reject},
    reply::{self, Response},
    Filter, Rejection, Reply,
};

type State = Arc<AppState>;

pub async fn run(state: AppState, server_db: impl ServerDb) -> Result<()> {
    let state: State = state.into();
    let servers = warp::path("servers");
    let all = path::end()
        .and(warp::get())
        .and(with_clone(server_db.clone()))
        .and_then(|db| async { get_all(db).await.map_err(reject::custom) })
        .recover(recover_route);
    let create = path::end()
        .and(warp::post())
        .and(body::content_length_limit(1024 * 16)) // 16 KiB
        .and(body::json())
        .and(with_clone(state))
        .and(with_clone(server_db))
        .and_then(|server, state, db| async {
            create(server, state, db).await.map_err(reject::custom)
        })
        .recover(recover_route);

    warp::serve(servers.and(all.or(create)))
        .run(([0; 4], 8080))
        .await;
    Ok(())
}

fn with_clone<T: Clone + Send>(val: T) -> impl Filter<Extract = (T,), Error = Infallible> + Clone {
    warp::any().map(move || val.clone())
}

async fn get_all(db: impl ServerDb) -> Result<impl Reply, InternalError> {
    Ok(reply::json(
        &db.all().await?.collect::<Result<Vec<_>, _>>().await?,
    ))
}

async fn create(
    server: NewServer,
    state: State,
    db: impl ServerDb,
) -> Result<Response, InternalError> {
    let id = server.create(&state.name, &db).await?;
    Ok(reply::with_status(reply::json(&id), StatusCode::CREATED).into_response())
}
#[derive(Debug)]
struct InternalError(anyhow::Error);

impl Reject for InternalError {}

impl<E: Into<anyhow::Error>> From<E> for InternalError {
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

async fn recover_route(reject: Rejection) -> Result<impl Reply, Rejection> {
    if let Some(InternalError(err)) = reject.find() {
        error!("unhandled error: {}", err);
        Ok(StatusCode::INTERNAL_SERVER_ERROR)
    } else {
        Err(reject)
    }
}
