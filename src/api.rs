use anyhow::Result;
use warp::{path, Filter};

pub async fn run() {
    let servers = warp::path("servers");
    let get_servers = servers.and(path::end()).and(warp::get());

    warp::serve(warp::get().map(|| Ok("")))
        .run(([0; 4], 3030))
        .await;
}
