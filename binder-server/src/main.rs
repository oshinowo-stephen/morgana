use tide::prelude::*;
use tide::http::StatusCode;

use binder_fm::local::{self, LocalFile};

use std::env;
use std::path::PathBuf;

#[derive(Clone, Debug)]
struct AppState {
  connection: binder_entities::Connection,
  _binder_token: String,
  container_limit: u64,
  upload_limit: u64,
}

#[async_std::main]
async fn main() -> tide::Result<()> {
  dotenvy::dotenv().ok();
  knil::init().ok();

  let binder_address = env::var("BINDER_ADDRESS")?;

  let mut app = tide::with_state(AppState {
    connection: binder_entities::create_connection(),
    _binder_token: env::var("BINDER_STORAGE_TOKEN")?,
    container_limit: env::var("MAIN_CONTAINER_LIMIT")?
      .to_string()
      .parse::<u64>()
      .unwrap_or(5),
    upload_limit: env::var("BINDER_UPLOAD_LIMIT")?
      .to_string()
      .parse::<u64>()
      .unwrap_or(80)
  });

  app.with(tide::log::LogMiddleware::new());

  app
    .at("/:file")
    .get(fetch_file)
    .post(insert_file)
    .delete(remove_file);

  app.listen(&binder_address).await?;
  Ok(())
}

async fn fetch_file(req: tide::Request<AppState>) -> tide::Result {
  let conn = req.state().connection.clone();

  match local::fetch_file(conn, req.param("file").expect("invalid file name.").to_string()) {
    Ok(entry) => if let Ok(body) = tide::Body::from_file(entry).await {
      Ok(body.into())
    } else {
      Ok(handle_rejection("not found.", StatusCode::InternalServerError))
    },
    Err(error) => {
      dbg!(&error);

      Ok(handle_rejection("internal server error", StatusCode::InternalServerError))
    }
  }
}

async fn insert_file(mut req: tide::Request<AppState>) -> tide::Result {
  if let Some(incoming_bearer_token) = req.header("Bearer") {
    if incoming_bearer_token.to_string().contains(&req.state()._binder_token) {
      let conn = req.state().connection.clone();

      let pending_entry = local::create_file(conn, &LocalFile {
        name: req.param("file").unwrap().to_string(),
        path: Box::new(PathBuf::from(req.param("file").unwrap())),
      }, req.body_bytes().await.unwrap());

      if let Err(error) = pending_entry {
        dbg!(&error);

        Ok(handle_rejection("internal server error", StatusCode::InternalServerError))
      } else {
        Ok(tide::Response::new(200))
      }
    } else {
      Ok(invalid_storage_token())
    }
  } else {
    Ok(invalid_storage_token())
  }
}

async fn remove_file(req: tide::Request<AppState>) -> tide::Result {
  if let Some(incoming_bearer_token) = req.header("Bearer") {
    if incoming_bearer_token.to_string().contains(&req.state()._binder_token) {
      let file_path = req.param("file")
        .unwrap();
      let conn = req.state().connection.clone();
      let target_path = PathBuf::from(file_path);

      if let Err(error) = local::remove_file(conn, target_path.as_path()) {
        dbg!(&error);

        Ok(handle_rejection("internal server error", StatusCode::InternalServerError))
      } else {
        Ok(tide::Response::new(200))
      }
    } else {
      Ok(invalid_storage_token())
    }
  } else {
    Ok(invalid_storage_token())
  }
}

fn handle_rejection(err: &str, status: tide::http::StatusCode) -> tide::Response {
  tide::Response::builder(status)
    .body(json!({
      "message": err,
      "status": status
    }))
    .build()
}

fn invalid_storage_token() -> tide::Response {
  tide::Response::builder(400)
    .body(json!({
      "message": "invalid or missing storage token",
      "status": 400
    }))
    .build()
}
