use crate::{db::ServerDb, server::NewServer};
use anyhow::Result;
use log::error;
use std::convert::Infallible;
use tokio_stream::StreamExt;
use warp::{
    body,
    hyper::StatusCode,
    path,
    reject::{self, Reject},
    reply::{self, Response},
    Filter, Rejection, Reply,
};

pub async fn run(app_name: String, server_db: impl ServerDb) -> Result<()> {
    let servers = warp::path("servers");
    let all = path::end()
        .and(warp::get())
        .and(with_server_db(server_db.clone()))
        .and_then(|db| async { get_all(db).await.map_err(|err| reject::custom(err)) })
        .recover(recover_route);
    let create = path::end()
        .and(warp::post())
        .and(body::content_length_limit(1024 * 16)) // 16 KiB
        .and(body::json())
        .and(with_server_db(server_db))
        .and_then(|server, db| async {
            create(server, db).await.map_err(|err| reject::custom(err))
        })
        .recover(recover_route);

    warp::serve(servers.and(all.or(create)))
        .run(([0; 4], 8080))
        .await;
    Ok(())
}

fn with_server_db<Db: ServerDb>(
    db: Db,
) -> impl Filter<Extract = (Db,), Error = Infallible> + Clone {
    warp::any().map(move || db.clone())
}

async fn get_all(server_db: impl ServerDb) -> Result<impl Reply, InternalError> {
    Ok(reply::json(
        &server_db
            .all()
            .await?
            .collect::<Result<Vec<_>, _>>()
            .await?,
    ))
}

async fn create(server: NewServer, server_db: impl ServerDb) -> Result<Response, InternalError> {
    todo!()
    // Ok(reply::with_status(reply::json(&id), StatusCode::CREATED).into_response())
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
        return Ok(StatusCode::INTERNAL_SERVER_ERROR);
    }
    Err(reject)
}
