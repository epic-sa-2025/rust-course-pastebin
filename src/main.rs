use std::sync::Arc;

use axum::{
    Extension, Router,
    body::Body,
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use clap::Parser;
use futures::TryStreamExt;
use service::Service;
use state::State;
use uuid::Uuid;

mod cli;
mod service;
mod state;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = cli::Args::parse();
    let state = State::load(&args.state)?;
    let service = Service::new(args.data_dir, state)?;

    let app = Router::new()
        .route("/", get(root))
        .route("/paste", post(post_paste))
        .route("/paste/{id}", get(get_paste))
        .layer(Extension(Arc::new(service)));

    let address: (&'static str, u16) = ("0.0.0.0", args.port);
    let listener = tokio::net::TcpListener::bind(address).await.unwrap();
    let fut = axum::serve(listener, app);
    println!("Listening on {}:{}", address.0, address.1);
    fut.await.unwrap();

    Ok(())
}

async fn root() -> &'static str {
    "Hello!"
}

async fn get_paste(Extension(service): Extension<Arc<Service>>, Path(id): Path<Uuid>) -> Response {
    match service.read(&id).await {
        Ok(reader) => {
            let stream = tokio_util::io::ReaderStream::new(reader);
            axum::body::Body::from_stream(stream).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn post_paste(Extension(service): Extension<Arc<Service>>, body: Body) -> Response {
    let reader =
        tokio_util::io::StreamReader::new(body.into_data_stream().map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::ConnectionAborted, e.to_string())
        }));
    match service.create(reader, None).await {
        Ok(id) => id.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}
