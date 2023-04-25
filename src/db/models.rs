use std::fmt;

use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use uuid::Uuid;
use super::schema::tasks;
use super::schema::tasks::dsl::tasks as task_dsl;

#[derive(Debug, Deserialize, Serialize, Queryable, Insertable, Identifiable)]
#[diesel(table_name = tasks)]
pub struct Task {
    pub id: String,
    pub name: String,
    pub description: String,
    pub status:  String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime
}

#[derive(Debug, Deserialize, Serialize)]
pub enum TaskStatus {
    Created,
    InProgress,
    Done,
    Archived
}

impl fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            TaskStatus::Created    => write!(f, "created"),
            TaskStatus::InProgress => write!(f, "in_progress"),
            TaskStatus::Done       => write!(f, "done"),
            TaskStatus::Archived   => write!(f, "archived"),
        }
    }
} 

impl Task {

    pub fn new(name: &str, descr: Option<&str>) -> Self {
        let id = Uuid::new_v4().hyphenated().to_string();
        let description = descr.unwrap_or("").to_string();
        Self {
            id,
            name: name.to_string(),
            description, 
            status: TaskStatus::Created.to_string(),
            created_at: chrono::Local::now().naive_local(),
            updated_at: chrono::Local::now().naive_local()
        }
    }

    
    pub fn create(name: &str, description: Option<&str>, conn: &mut SqliteConnection) -> Option<Self> {
        //TODO restrict name to include no whitespaces
        if let Some(task) = Self::by_name(name, conn) {
            //TODO if exists let caller know that it has existed
            return Some(task)
        }

        let new_task = Self::new(name, description);

        diesel::insert_into(task_dsl)
            .values(&new_task)
            .execute(conn)
            .expect("Error saving new user");
        Self::by_id(&new_task.id.as_str(), conn)
    }

    pub fn list(conn: &mut SqliteConnection) -> Vec<Self> {
        task_dsl.load::<Task>(conn).expect("Error loading tasks") 
    }


    pub fn by_id(id: &str, conn: &mut SqliteConnection) -> Option<Self> {
        if let Ok(record) = task_dsl.find(id).first::<Task>(conn) {
            Some(record)
        } else {
            None
        }
    }

    pub fn by_name(name_query: &str, conn: &mut SqliteConnection) -> Option<Self> {
        use super::schema::tasks::dsl::name;
        if let Ok(record) = task_dsl.filter(name.eq(name_query)).first::<Task>(conn) {
            Some(record)
        } else {
            None
        }
    }

    pub fn update(tsk: Task, conn: &mut SqliteConnection) -> Option<Self> {
        use super::schema::tasks::dsl::{id, name, description, status};
        match diesel::update(task_dsl)
            .filter(id.eq(tsk.id.clone()))
            .set((name.eq(tsk.name), description.eq(tsk.description), status.eq(tsk.status)))
            .execute(conn) {
                Ok(_) => Self::by_id(tsk.id.as_str(), conn),
                Err(_) => None
            }
    }

    pub fn set_status(task_id: &str, new_status: &str, conn: &mut SqliteConnection) -> Option<Self> {
        use super::schema::tasks::dsl::{id, status};
        match diesel::update(task_dsl)
            .filter(id.eq(task_id))
            .set(status.eq(new_status))
            .execute(conn) {
                Ok(_) => Self::by_id(task_id, conn),
                Err(_) => None
            }
    }

    pub fn filter_by_status(filter_status: &str, conn: &mut SqliteConnection) -> Vec<Task> {
        use super::schema::tasks::dsl::status;
        if let Ok(records) = task_dsl.filter(status.eq(filter_status)).get_results(conn) {
            records
        } else {
            vec![]
        }
    }
}

#[cfg(test)]
mod task_tests;
