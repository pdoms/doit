use std::fmt;
use actix_web::{Responder, web, get, post, put, HttpResponse, http::header::ContentType};
use serde::{Serialize, Deserialize, de};
use chrono::NaiveDateTime;


use crate::db::{DbPool, models::Task};

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskForm {
    name: String,
    description: Option<String>,
    #[serde(deserialize_with = "deserialize_due")]
    due: Option<chrono::NaiveDateTime>
}


#[derive(Debug, Serialize, Deserialize)]
pub struct TaskUpdate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub status: i32,
    #[serde(deserialize_with = "deserialize_due")]
    pub due: Option<chrono::NaiveDateTime>,
    #[serde(deserialize_with = "deserialize_ats")]
    pub created_at: chrono::NaiveDateTime,
    #[serde(deserialize_with = "deserialize_ats")]
    pub updated_at: chrono::NaiveDateTime
}
const FORMAT: &str = "%Y-%m-%dT%H:%M:%S%.fZ";
const FORMATNAIVE: &str = "%Y-%m-%dT%H:%M:%S%.f";
fn deserialize_due<'de, D>(deserializer: D) -> Result<Option<NaiveDateTime>, D::Error>
where
    D: de::Deserializer<'de>,
{
    struct DueDTVisitor;

    impl<'de> de::Visitor<'de> for DueDTVisitor {
        type Value = Option<NaiveDateTime>;
    
        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("unix datetime str")
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: de::Error, {
            Ok(None)
        }

        fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: serde::Deserializer<'de>, {
                deserializer.deserialize_any(DueDTVisitor)
        }
    
        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            
            match NaiveDateTime::parse_from_str(v, FORMAT) {
                        Ok(res) => Ok(Some(res)),
                        Err(_) => Err(de::Error::custom("dt parse err"))
                    }
            }
    }
    deserializer.deserialize_option(DueDTVisitor)
}
fn deserialize_ats<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
where
    D: de::Deserializer<'de>,
{
    struct DueDTVisitor;

    impl<'de> de::Visitor<'de> for DueDTVisitor {
        type Value = NaiveDateTime;
    
        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("unix datetime str")
        }
    
        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            if v.ends_with("Z") {
            match NaiveDateTime::parse_from_str(v, FORMAT) {
                Ok(res) => Ok(res),
                Err(_) => Err(de::Error::custom("dt parse err"))
            }
            } else {
            match NaiveDateTime::parse_from_str(v, FORMATNAIVE) {
                Ok(res) => Ok(res),
                Err(_) => Err(de::Error::custom("dt parse err"))
            }
            }
        }
    }
    deserializer.deserialize_any(DueDTVisitor)
}


#[derive(Debug, Serialize, Deserialize)]
pub struct FilterStatus {
    status: i32 
}
#[derive(Debug, Serialize, Deserialize)]
pub struct FilterText {
    term: String
}

#[post("/create")]
pub async fn create(task_form: web::Json<TaskForm>, pool: web::Data<DbPool>) -> impl Responder {
    let mut conn = pool.get().unwrap();
    match Task::create(task_form.name.as_str(), task_form.description.as_deref(), task_form.due, &mut conn) {
        Some(task) => HttpResponse::Created().insert_header(ContentType::json()).json(task),
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
pub async fn task_update(task: web::Json<TaskUpdate>, pool: web::Data<DbPool>) -> impl Responder {
    let mut conn = pool.get().unwrap();
    match Task::update(task.into_inner(), &mut conn) {
        Some(tsk) => HttpResponse::Ok().insert_header(ContentType::json()).json(tsk),
        _ => HttpResponse::NotFound().json("Not Found")
    }
}

#[get("/set/{id}/{status}")]
pub async fn set_status(extracted: web::Path<(String, i32)>, pool: web::Data<DbPool>) -> impl Responder {
    let mut conn = pool.get().unwrap();
    match Task::set_status(&extracted.0, extracted.1, &mut conn) {
        Some(tsk) => HttpResponse::Ok().json(tsk),
        _ => HttpResponse::NotFound().json("Not Found")
    }
}

#[get("/filter")]
pub async fn filter_by_status(status_query: web::Query<FilterStatus>,pool: web::Data<DbPool>) -> impl Responder {
    let mut conn = pool.get().unwrap();
    let result = Task::filter_by_status(status_query.status, &mut conn);
    match result.len() {
        0 => HttpResponse::NotFound().json("No entries found."),
        _ => HttpResponse::Ok().json(result)
    }
}

#[get("/global")]
pub async fn filter_text(text_query: web::Query<FilterText>,pool: web::Data<DbPool>) -> impl Responder {
    let mut conn = pool.get().unwrap();
    let result = Task::text_filter(&text_query.term, &mut conn);
    match result.len() {
        0 => HttpResponse::NotFound().json("No entries found."),
        _ => HttpResponse::Ok().json(result)
    }
}

