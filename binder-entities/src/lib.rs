use diesel::r2d2::ConnectionManager;
use diesel::sqlite::SqliteConnection;
use diesel::r2d2::Pool;
use std::env;

pub mod schema;

pub mod file_entry {

  use diesel::prelude::*;

  #[derive(Queryable, Insertable, Selectable)]
  #[diesel(table_name = super::schema::file_entry)]
  pub struct FileEntry {
    pub id: i32,
    pub entry_path: String,
  }

  pub fn fetch_file_entry_by_id(file_id: i32, connection: &mut SqliteConnection) -> Option<FileEntry> {
    use super::schema::file_entry::dsl::file_entry as file_entries;

    match file_entries
      .find(file_id)
      .select(FileEntry::as_select())
      .first(connection)
    {
      Ok(file_entry) => Some(file_entry),
      Err(_error) => {
        println!("An error finding file: {}, reason: {:?}", file_id, _error);
        None
      },
    }
  }

  pub fn fetch_file_entry(path: String, connection: &mut SqliteConnection) -> Option<FileEntry> {
    use super::schema::file_entry::dsl::{entry_path, file_entry as file_entries};

    match file_entries
      .filter(entry_path.eq(&path))
      .select(FileEntry::as_select())
      .first(connection)
    {
      Ok(file_entry) => Some(file_entry),
      Err(_error) => {
        println!("An error finding file: {}, reason: {:?}", path, _error);
        None
      },
    }
  }

  pub fn fetch_file_entries(connection: &mut SqliteConnection) -> Vec<FileEntry> {
    use super::schema::file_entry::dsl::file_entry as file_entries;

    match file_entries
      .load(connection)
    {
      Ok(entries) => entries,
      Err(_error) => {
        println!("Error occurred fetching all entries: {:?}", _error);

        vec![]
      }
    }
  }

  pub fn remove_file_entry(file: String, connection: &mut SqliteConnection) {
    use super::schema::file_entry::dsl::{entry_path, file_entry as file_entries};

    let file_entry = file_entries.filter(entry_path.eq(file));

    match diesel::delete(file_entry)
      .execute(connection)
    {
      Ok(_) => {
        println!("entry was removed from the database.")
      },
      Err(_error) => {
        eprintln!("an error has occurred removing this entry: {:?}", _error)
      }
    }
  }

  pub fn insert_file_entry(entry: FileEntry, connection: &mut SqliteConnection) {
    use super::schema::file_entry;

    match diesel::insert_into(file_entry::table)
      .values(&entry)
      .execute(connection)
    {
      Ok(_) => println!("File has been stored"),
      Err(_error) => {
        println!("Error on file store: {:?}", _error)
      }
    };

  }
}

pub type Connection = Pool<ConnectionManager<SqliteConnection>>;

pub fn create_connection() -> Connection {
  let database_url = env::var("DATABASE_URL")
    .expect("database_url not found.");

  let manager = ConnectionManager::<SqliteConnection>::new(database_url);

  Pool::builder()
    .test_on_check_out(true)
    .build(manager)
    .expect("cannot create pooled connection")
}
