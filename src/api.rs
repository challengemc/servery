use crate::{
    db::{self, server::Error as ServerDbError, ServerDb},
    server::Server,
};
use anyhow::Result;
use std::convert::Infallible;
use warp::{body, hyper::StatusCode, path, reply, Filter, Reply};

pub async fn run() -> Result<()> {
    let server_db = db::server::load("servers.json").await?;
    let servers = warp::path("servers");
    let all = path::end()
        .and(warp::get())
        .and(with_server_db(server_db.clone()))
        .and_then(get_all);
    let add = path::end()
        .and(warp::post())
        .and(body::content_length_limit(1024 * 16)) // 16 KiB
        .and(body::json())
        .and(with_server_db(server_db))
        .and_then(add);

    warp::serve(servers.and(all.or(add)))
        .run(([0; 4], 3030))
        .await;
    Ok(())
}

fn with_server_db(db: ServerDb) -> impl Filter<Extract = (ServerDb,), Error = Infallible> + Clone {
    warp::any().map(move || db.clone())
}

async fn get_all(server_db: ServerDb) -> Result<impl Reply, Infallible> {
    Ok(reply::json(&server_db.read().await.all()))
}

async fn add(server: Server, server_db: ServerDb) -> Result<impl Reply, Infallible> {
    match server_db.write().await.add(server).await {
        Ok(_) => Ok(StatusCode::CREATED),
        Err(ServerDbError::DuplicateId(_)) => Ok(StatusCode::CONFLICT),
        Err(_) => Ok(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
