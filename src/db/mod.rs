pub mod models;
pub mod schema;

use diesel::pg::Pg;
use dotenv::dotenv;
use std::env;
use std::error::Error;
use diesel::PgConnection;
use diesel::r2d2::Pool;
use diesel::r2d2::ConnectionManager;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

pub type DbPool = Pool<ConnectionManager<PgConnection>>;
const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

fn run_migrations(conn: &mut impl MigrationHarness<Pg>) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    conn.run_pending_migrations(MIGRATIONS)?;
    Ok(())
}


pub fn establish_connection() -> DbPool {
        dotenv().ok();
        let mut database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        if cfg!(test) {
            database_url =  env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        }
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        let pool =Pool::builder()
            .build(manager)
            .expect("Failed to create DB pool.");
        if cfg!(test) {
            let _ = run_migrations(&mut pool.get().unwrap()).expect("error running migrations");
        }
        pool
}

