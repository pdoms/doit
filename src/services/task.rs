use actix_web::{Responder, web, get, post, put, HttpResponse};
use serde::{Serialize, Deserialize};

use crate::db::{DbPool, models::Task};

#[derive(Serialize, Deserialize)]
pub struct TaskForm {
    name: String,
    description: Option<String>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FilterStatus {
    status: String
}
#[derive(Debug, Serialize, Deserialize)]
pub struct RawTask {
    pub id: String,
    pub name: String,
    pub description: String,
    pub status:  String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime
}

#[post("/")]
pub async fn create(task_form: web::Json<TaskForm>, pool: web::Data<DbPool>) -> impl Responder {
    let mut conn = pool.get().unwrap();
    match Task::create(task_form.name.as_str(), task_form.description.as_deref(), &mut conn) {
        Some(task) => HttpResponse::Created().json(task),
        _ => HttpResponse::InternalServerError().json("Could not create user")
    }
}

#[get("/")]
pub async fn index(pool: web::Data<DbPool>) -> impl Responder {
    let mut conn = pool.get().unwrap();
    HttpResponse::Ok().json(Task::list(&mut conn))
}
#[get("/{id}")]
pub async fn get_by_id(id: web::Path<String>, pool: web::Data<DbPool>) -> impl Responder {
    let mut conn = pool.get().unwrap();
    match Task::by_id(&id, &mut conn) {
        Some(task) => HttpResponse::Ok().json(task),
        _ => HttpResponse::NotFound().json("Not Found")
    }
}

#[put("/")]
pub async fn task_update(task: web::Json<Task>, pool: web::Data<DbPool>) -> impl Responder {
    let mut conn = pool.get().unwrap();
    match Task::update(task.into_inner(), &mut conn) {
        Some(tsk) => HttpResponse::Ok().json(tsk),
        _ => HttpResponse::NotFound().json("Not Found")
    }
}

#[get("/set/{id}/{status}")]
pub async fn set_status(extracted: web::Path<(String, String)>, pool: web::Data<DbPool>) -> impl Responder {
    let mut conn = pool.get().unwrap();
    match Task::set_status(&extracted.0, &extracted.1, &mut conn) {
        Some(tsk) => HttpResponse::Ok().json(tsk),
        _ => HttpResponse::NotFound().json("Not Found")
    }
}

#[get("/filter")]
pub async fn filter_by_status(status_query: web::Query<FilterStatus>,pool: web::Data<DbPool>) -> impl Responder {
    let mut conn = pool.get().unwrap();
    let result = Task::filter_by_status(&status_query.status, &mut conn);
    match result.len() {
        0 => HttpResponse::NotFound().json("No entries found."),
        _ => HttpResponse::Ok().json(result)
    }


}

