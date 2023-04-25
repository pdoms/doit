mod db;
mod services;

use services::task::{
    create, 
    index, 
    get_by_id, 
    task_update, 
    set_status,
    filter_by_status
};

const HOST: &str = "127.0.0.1";
const PORT: u16 = 8080;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
 
    use actix_cors::Cors;
    use actix_web::{App, web, HttpServer};
    use actix_web::middleware::Logger;
    use env_logger;

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    
    HttpServer::new(move || {
        let conn_pool = db::establish_connection();
        let cors = Cors::permissive();
        App::new()
            .wrap(Logger::default())
            .wrap(cors)
            .app_data(web::Data::new(conn_pool))
            .service(index)
            .service(create)
            .service(get_by_id)
            .service(set_status)
            .service(task_update)
            .service(filter_by_status)
    })
        .bind((HOST, PORT))?
        .run()
        .await
}
