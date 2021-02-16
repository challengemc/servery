use crate::{
    db::{self, ServerDb},
    server::NewServer,
};
use anyhow::Result;
use log::error;
use std::convert::Infallible;
use warp::{
    body,
    hyper::StatusCode,
    path,
    reply::{self, Response},
    Filter, Reply,
};

pub async fn run() -> Result<()> {
    let server_db = db::server::load("servers.json").await?;
    let servers = warp::path("servers");
    let all = path::end()
        .and(warp::get())
        .and(with_server_db(server_db.clone()))
        .and_then(get_all);
    let create = path::end()
        .and(warp::post())
        .and(body::content_length_limit(1024 * 16)) // 16 KiB
        .and(body::json())
        .and(with_server_db(server_db))
        .and_then(create);

    warp::serve(servers.and(all.or(create)))
        .run(([0; 4], 8080))
        .await;
    Ok(())
}

fn with_server_db(db: ServerDb) -> impl Filter<Extract = (ServerDb,), Error = Infallible> + Clone {
    warp::any().map(move || db.clone())
}

async fn get_all(server_db: ServerDb) -> Result<impl Reply, Infallible> {
    Ok(reply::json(&server_db.read().await.all()))
}

async fn create(server: NewServer, server_db: ServerDb) -> Result<Response, Infallible> {
    match server_db.write().await.add(server).await {
        Ok(id) => Ok(reply::with_status(reply::json(&id), StatusCode::CREATED).into_response()),
        Err(e) => {
            // TODO: actual fucking error handling
            error!("error creating server: {}", e);
            Ok(StatusCode::INTERNAL_SERVER_ERROR.into_response())
        }
    }
}
