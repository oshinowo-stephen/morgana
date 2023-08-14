pub mod local {

  use std::env;
  use std::io::Write;
  use std::path::Path;
  use std::path::PathBuf;
  use std::fs::{self, OpenOptions};

  use binder_entities::Connection;
  use binder_entities::file_entry;

  pub struct LocalFile {
    pub name: String,
    pub path: Box<PathBuf>,
  }

  #[derive(Debug)]
  pub struct IoError {
    pub message: String,
    pub cause: String,
    pub code: usize,
  }

  pub type Result<T> = std::result::Result<T, IoError>;

  fn get_container_path() -> Box<Path> {
    let container_path = env::var("MAIN_CONTAINER_PATH")
      .unwrap_or_else(|_| "bin".to_owned());

    Path::new(&container_path).into()
  }

  pub fn get_container_size() -> usize {
    let mut container_length = 0;

    let container_path = env::var("MAIN_CONTAINER_PATH")
      .expect("MAIN_CONTIANER_PATH is not set. Please set in environment");

    match fs::read_dir(&container_path) {
      Ok(entries) => for entry_result in entries {
        let entry = entry_result.unwrap();

        if let Ok(metadata) = entry.metadata() {
          if metadata.is_file() {
            container_length += metadata.len()
          } else {
            continue
          }
        }
      },
      Err(_error) => {
        println!("Unable get entries, reason: {:#?}", _error);
      }
    }

    container_length.try_into().unwrap()
  }

  pub fn create_file(conn: Connection, file_prop: &LocalFile, bytes: Vec<u8>) -> Result<()> {
    use file_entry::FileEntry;

    let target = get_container_path()
      .join(file_prop.path.as_path());

    match OpenOptions::new()
      .create(true)
      .append(true)
      .open(target)
    {
      Ok(mut file) => if let Err(error) = file.write_all(&bytes) {
        eprintln!("An error occurred has occurred, {:?}", error);
      } else {
        if let Ok(mut connection) = conn.get() {
          file_entry::insert_file_entry(FileEntry {
            id: binder_utils::generate_random_number() as i32,
            entry_path: format!("{}", file_prop.path.display())
          }, &mut connection)
        } else {
          eprintln!("Unable to store this file into the database.");
        }
      },
      Err(_error) => {
        eprintln!("An error has occurred: {:?}", _error)
      }
    }

    Ok(())
  }

  pub fn fetch_all_files(conn: Connection) -> Result<Vec<Box<Path>>> {
    if let Ok(mut connection) = conn.get() {
      let mut file_paths: Vec<Box<Path>> = Vec::new();

      for entry in file_entry::fetch_file_entries(&mut connection) {
        file_paths.push(Path::new(&entry.entry_path).into())
      }

      Ok(file_paths)
    } else {
      Err(IoError {
        message: String::from("failed to fetch files"),
        cause: String::from("cannot fetch connection."),
        code: 500
      })
    }
  }

  pub fn fetch_file(conn: Connection, file_name: String) -> Result<Box<Path>> {
    if let Ok(mut connection) = conn.get() {
      if let Some(entry) = file_entry::fetch_file_entry(file_name, &mut connection) {
        Ok(get_container_path().join(Path::new(&entry.entry_path)).as_path().into())
      } else {
        Err(IoError {
          message: String::from("something"),
          cause: String::from("something"),
          code: 500
        })
      }
    } else {
      eprintln!("unable to secure connection");

      Err(IoError {
        message: String::from("something"),
        cause: String::from("something"),
        code: 500
      })
    }
  }

  pub fn remove_file(conn: Connection, file_path: &Path) -> Result<()> {
    let target = get_container_path().join(file_path);

    if let Ok(mut connection) = conn.get() {
      if let Ok(_) = fs::remove_file(target) {
        let file_name = file_path.file_name().unwrap().to_str().unwrap();
        file_entry::remove_file_entry(file_name.to_string(), &mut connection);
      }
    }

    Ok(())
  }

}

