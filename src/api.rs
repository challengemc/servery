use crate::db::ServerDb;
use anyhow::Result;
use std::convert::Infallible;
use warp::{path, reply, Filter, Reply};

pub async fn run() -> Result<()> {
    let server_db = ServerDb::load("servers.json").await?;
    let servers = warp::path("servers");
    let all_servers = servers
        .and(path::end())
        .and(warp::get())
        .and(with_server_db(server_db))
        .and_then(get_all);

    warp::serve(all_servers).run(([0; 4], 3030)).await;
    Ok(())
}

fn with_server_db(db: ServerDb) -> impl Filter<Extract = (ServerDb,), Error = Infallible> + Clone {
    warp::any().map(move || db.clone())
}

async fn get_all(server_db: ServerDb) -> Result<impl Reply, Infallible> {
    Ok(reply::json(&&*server_db.all().await))
}
