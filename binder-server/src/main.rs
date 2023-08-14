use tide::log;
use tide::prelude::*;
use tide::http::StatusCode;

use binder_fm::local::{self, LocalFile};

use std::env;
use std::path::PathBuf;

#[derive(Clone, Debug)]
struct AppState {
  connection: binder_entities::Connection,
  _binder_token: String,
  container_limit: usize,
  binder_address: String,
}

#[async_std::main]
async fn main() -> tide::Result<()> {
  dotenvy::dotenv().ok();
  knil::init().ok();

  let binder_address = env::var("BINDER_ADDRESS")?;

  let mut app = tide::with_state(AppState {
    binder_address: binder_address.clone(),
    connection: binder_entities::create_connection(),
    _binder_token: env::var("BINDER_STORAGE_TOKEN")?,
    container_limit: env::var("MAIN_CONTAINER_LIMIT")?
      .to_string()
      .parse::<usize>()
      .unwrap_or(5),
  });

  app.with(tide::log::LogMiddleware::new());

  app
    .at("/")
    .get(fetch_all);

  app
    .at("/files/:file")
    .get(fetch_file)
    .post(insert_file)
    .delete(remove_file);

  app.listen(&binder_address.clone()).await?;
  Ok(())
}

async fn fetch_all(req: tide::Request<AppState>) -> tide::Result {
  let conn = req.state().connection.clone();

  match local::fetch_all_files(conn) {
    Ok(file_paths) => {
      Ok(
        json!({
          "file_paths": file_paths
        }).into()
      )
    },
    Err(error) => {
      dbg!(&error);

      Ok(handle_rejection("internal server error", StatusCode::InternalServerError))
    }
  }
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
    if let Some(content_length) = req.len() {
      log::debug!("incoming file content length = {}", content_length);

      log::debug!("current container size: {:#?}, maxmium container size: {:#?}",
        binder_fm::local::get_container_size() + content_length,
        req.state().container_limit * 1000_usize.pow(3)
      );

      if (binder_fm::local::get_container_size() + content_length) as usize > req.state().container_limit * 1000_usize.pow(3) {
        return Ok(handle_rejection("exceeded container limit", StatusCode::BadRequest))
      }
    } else {
      return Ok(handle_rejection("no content", StatusCode::NoContent))
    }

    if incoming_bearer_token.to_string().contains(&req.state()._binder_token) {
      let conn = req.state().connection.clone();

      let pending_entry = local::create_file(conn, &LocalFile {
        name: req.param("file").unwrap().to_string(),
        path: Box::new(PathBuf::from(req.param("file").unwrap())),
      }, req.body_bytes().await.unwrap());

      if let Err(error) = pending_entry {
        log::error!("pending entry {:#?} encountered error: {:#?}", req.param("file"), error);

        Ok(handle_rejection("internal server error", StatusCode::InternalServerError))
      } else {
        log::info!("new file entry - {:?}, successfully stored.", req.param("file"));
        let location = format!("http://{}/files/{:#?}",
          req.state().binder_address,
          req.param("file")
        );

        Ok(
          tide::Response::builder(200)
            .body(location)
            .build()
        )
      }
    } else {
      Ok(handle_rejection("invalid or missing token.", StatusCode::Forbidden))
    }
  } else {
    Ok(handle_rejection("invalid or missing token.", StatusCode::Forbidden))
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
        log::error!("pending entry {:#?} encountered error: {:#?}", req.param("file"), error);

        Ok(handle_rejection("internal server error", StatusCode::InternalServerError))
      } else {
        Ok(tide::Response::new(200))
      }
    } else {
      Ok(handle_rejection("invalid or missing token.", StatusCode::Forbidden))
    }
  } else {
    Ok(handle_rejection("invalid or missing token.", StatusCode::Forbidden))
  }
}

fn handle_rejection(err: &str, status: tide::http::StatusCode) -> tide::Response {
  tide::Response::builder(status)
    .body(json!({ "message": err, "status": status }))
    .build()
}
