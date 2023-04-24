pub mod models;
pub mod schema;

use chrono::Duration;
use diesel::connection::SimpleConnection;
use dotenv::dotenv;
use std::env;
use std::error::Error;
use diesel::prelude::*;
use diesel::sqlite::Sqlite;
use diesel::r2d2::Pool;
use diesel::r2d2::ConnectionManager;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

pub type DbPool = Pool<ConnectionManager<SqliteConnection>>;
const MIGRATIONS: EmbeddedMigrations = embed_migrations!();


#[derive(Debug)]
pub struct ConnectionOptions {
    pub enable_wal: bool,
    pub enable_foreign_keys: bool, 
    pub busy_timout: Option<u128>
}

impl r2d2::CustomizeConnection<SqliteConnection, diesel::r2d2::Error> for ConnectionOptions {
    fn on_acquire(&self, conn: &mut SqliteConnection) -> Result<(), diesel::r2d2::Error> {
        (|| {
            if self.enable_wal {
                conn.batch_execute("PRAGMA journal_mode = WAL; PRAGMA synchronous = NORMAL;")?;
            }
            if self.enable_foreign_keys {
                conn.batch_execute("PRAGMA foreign_keys = ON;")?;
            }
            if let Some(d) = self.busy_timout {
                conn.batch_execute(&format!("PRAGMA busy_timout = {};", d))?;
            }
            Ok(())
        })()
        .map_err(diesel::r2d2::Error::QueryError)
    }
}



fn run_migrations(conn: &mut impl MigrationHarness<Sqlite>) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    conn.run_pending_migrations(MIGRATIONS)?;
    Ok(())
}


pub fn establish_connection() -> DbPool {
    //if cfg!(test) {
        let manager = ConnectionManager::<SqliteConnection>::new(":memory");
        let pool =Pool::builder()
            .max_size(16)
            .connection_customizer(Box::new(ConnectionOptions {
                enable_wal: true,
                enable_foreign_keys: true,
                busy_timout: Some(30000)
            }))
        .build(manager).expect("Failed to create DB pool.");
        let _ = run_migrations(&mut pool.get().unwrap()).expect("error running migrations");
        pool
    //} else {
    //    dotenv().ok();
    //    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    //    let manager = ConnectionManager::<SqliteConnection>::new(database_url);
    //    Pool::builder().build(manager).expect("Failed to create DB pool.")
    //}
}

