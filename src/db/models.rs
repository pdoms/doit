use std::fmt;

use diesel::sql_types::is_nullable::IsNullable;
use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use diesel::dsl::now;
use uuid::Uuid;
use super::schema::tasks;
use super::schema::tasks::dsl::tasks as task_dsl;

#[derive(Debug, Deserialize, Serialize, Queryable, Insertable, Identifiable)]
#[diesel(table_name = tasks)]
pub struct Task {
    pub id: String,
    pub name: String,
    pub description: String,
    pub status: String,
    pub due: Option<chrono::NaiveDateTime>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime
}

#[derive(Debug, Deserialize, Serialize)]
pub enum TaskStatus {
    Created,
    Overdue,
    Deleted,
    Done,
}

impl fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            TaskStatus::Created     => write!(f, "created"),
            TaskStatus::Overdue     => write!(f, "overdue"),
            TaskStatus::Done        => write!(f, "done"),
            TaskStatus::Deleted     => write!(f, "deleted"),

        }
    }
} 

impl Task {


    pub fn new(name: &str, descr: Option<&str>, due: Option<chrono::NaiveDateTime>) -> Self {
        let id = Uuid::new_v4().hyphenated().to_string();
        let description = descr.unwrap_or("").to_string();
        let ts = chrono::Local::now().naive_local();
        Self {
            id,
            name: name.to_string(),
            description, 
            status: TaskStatus::Created.to_string(),
            due,
            created_at: ts,
            updated_at: ts
        }
    }

    pub fn check_overdue(check_id: &str, conn: &mut PgConnection) -> Result<usize, diesel::result::Error> {
        use super::schema::tasks::dsl::{id, due, status};
        diesel::update(task_dsl)
            .filter(id.eq(check_id))
            .filter(due.lt(now))
            .set(status.eq("overdue"))
            .execute(conn)
    }

    pub fn create(name: &str, description: Option<&str>, due: Option<chrono::NaiveDateTime>, conn: &mut PgConnection) -> Option<Self> {
        if let Some(mut task) = Self::by_name(name, conn) {
            if task.due.is_some() {
                if task.due.unwrap().timestamp_millis() < chrono::Local::now().naive_local().timestamp_millis() {
                    task.status = "overdue".to_string();
                }
                return Some(task)
            } else {
                return Some(task)
            }
        }
        let new_task = Task::new(name, description, due);
        diesel::insert_into(task_dsl)
            .values(&new_task)
            .execute(conn)
            .expect("Error saving new user");
        Self::by_id(&new_task.id.as_str(), conn)
    }

    pub fn list(conn: &mut PgConnection) -> Vec<Self> {
        Task::set_overdues(conn);
        task_dsl.load::<Task>(conn).expect("Error loading tasks")
    }



    pub fn by_id(id: &str, conn: &mut PgConnection) -> Option<Self> {
        Task::check_overdue(id, conn).unwrap();
        if let Ok(record) = task_dsl.find(id).first::<Task>(conn) {
            Some(record)
        } else {
            None
        }
    }

    pub fn by_name(name_query: &str, conn: &mut PgConnection) -> Option<Self> {
        use super::schema::tasks::dsl::name;
        if let Ok(record) = task_dsl.filter(name.eq(name_query)).first::<Task>(conn) {
            Some(record)
        } else {
            None
        }
    }

    fn set_overdues(conn: &mut PgConnection) {
        use super::schema::tasks::dsl::{due, status};
        diesel::update(task_dsl)
            .filter(due.lt(now))
            .set(status.eq("overdue"))
            .execute(conn).expect("Failed to run overdue set");
    }

    pub fn update(tsk: Task, conn: &mut PgConnection) -> Option<Self> {
        use super::schema::tasks::dsl::{id, name, description, status, due};
        match diesel::update(task_dsl.find(&tsk.id))
            .set((name.eq(tsk.name), description.eq(tsk.description), status.eq(tsk.status), due.eq(tsk.due)))
            .execute(conn) {
                Ok(_) => Self::by_id(tsk.id.as_str(), conn),
                Err(_) => None
            }
    }

    pub fn set_status(task_id: &str, new_status: &str, conn: &mut PgConnection) -> Option<Self> {
        use super::schema::tasks::dsl::{id, status};
        match diesel::update(task_dsl.find(task_id))
            .set(status.eq(new_status))
            .execute(conn) {
                Ok(_) => {
                    let tsk = Self::by_id(task_id, conn).unwrap();
                    Some(tsk)
                },
                Err(_) => None
            }
    }

    pub fn filter_by_status(filter_status: &str, conn: &mut PgConnection) -> Vec<Task> {
        use super::schema::tasks::dsl::status;
        Task::set_overdues(conn);
        if let Ok(records) = task_dsl.filter(status.eq(filter_status)).get_results(conn) {
            records
        } else {
            vec![]
        }
    }

    pub fn delete_task(trg_id: &str, conn: &mut PgConnection) -> Result<usize, diesel::result::Error> {
        use super::schema::tasks::dsl::id;
        diesel::delete(task_dsl.find(trg_id))
            .execute(conn)
    }
}

#[cfg(test)]
mod task_tests;
