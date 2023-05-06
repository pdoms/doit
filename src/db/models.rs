use std::fmt;
use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use diesel::dsl::{now, not};
use uuid::Uuid;

use super::schema::tasks;
use super::schema::tasks::dsl::tasks as task_dsl;
use crate::services::task::TaskUpdate;
use crate::utils::sort::sort_by_score;

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, Queryable, Insertable, Identifiable)]
#[diesel(table_name = tasks)]
pub struct Task {
    pub id: String,
    pub name: String,
    pub description: String,
    pub status: i32,
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

impl TaskStatus {
    pub fn to_store(&self) -> i32 {
        match *self {
            TaskStatus::Created     => 1,
            TaskStatus::Overdue     => 0,
            TaskStatus::Done        => 2,
            TaskStatus::Deleted     => 3,
        }
    }
    pub fn from_store(status: i32) -> Self {
        match status {
            1 => TaskStatus::Created,
            0 => TaskStatus::Overdue,
            2 => TaskStatus::Done,
            3 => TaskStatus::Deleted,
            _ => unreachable!("Bug in TaskStatus::from_store()")
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
            status: TaskStatus::Created.to_store(),
            due,
            created_at: ts,
            updated_at: ts
        }
    }

    pub fn check_overdue(check_id: &str, conn: &mut PgConnection) -> Result<usize, diesel::result::Error> {
        use super::schema::tasks::dsl::{id, due, status};
        diesel::update(task_dsl)
            .filter(id.eq(check_id))
            .filter(not(status.eq(TaskStatus::Done.to_store())))
            .filter(not(status.eq(TaskStatus::Deleted.to_store())))
            .filter(due.lt(now))
            .set(status.eq(TaskStatus::Overdue.to_store()))
            .execute(conn)
    }

    pub fn create(name: &str, description: Option<&str>, due: Option<chrono::NaiveDateTime>, conn: &mut PgConnection) -> Option<Self> {
        if let Some(mut task) = Self::by_name(name, conn) {
            if task.due.is_some() {
                if task.due.unwrap().timestamp_millis() < chrono::Local::now().naive_local().timestamp_millis() {
                    task.status = TaskStatus::Overdue.to_store();
                }
                return Some(task)
            } else {
                return Some(task)
            }
        }
        let mut new_task = Task::new(name, description, due);
        if new_task.due.is_some() && new_task.due.unwrap().timestamp_millis() < chrono::Local::now().naive_local().timestamp_millis() {
            new_task.status = TaskStatus::Overdue.to_store();
        }
        diesel::insert_into(task_dsl)
            .values(&new_task)
            .execute(conn)
            .expect("Error saving new user");
        Self::by_id(&new_task.id.as_str(), conn)
    }

    pub fn list(conn: &mut PgConnection) -> Vec<Self> {
        Task::set_overdues(conn);
        use super::schema::tasks::dsl::{due, updated_at, status};
        task_dsl
            .filter(not(status.eq(TaskStatus::Deleted.to_store())))
            .order_by((due.asc(), status.asc(), updated_at.desc()))
            .load::<Task>(conn).expect("Error loading tasks")
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
            .set(status.eq(TaskStatus::Overdue.to_store()))
            .execute(conn).expect("Failed to run overdue set");
    }

    pub fn update(mut tsk: TaskUpdate, conn: &mut PgConnection) -> Option<Self> {
        use super::schema::tasks::dsl::{name, description, status, due};

        match tsk.due {
            Some(d) => {
                let ts_now = chrono::Local::now().naive_local();
                if d.timestamp_millis() < ts_now.timestamp_millis() {
                    tsk.status = TaskStatus::Overdue.to_store();
                } else {
                    tsk.status = TaskStatus::Created.to_store();
                }
            }
            None => {}
        }

        match diesel::update(task_dsl.find(&tsk.id))
            .set((name.eq(tsk.name), description.eq(tsk.description), status.eq(tsk.status), due.eq(tsk.due)))
            .execute(conn) {
                Ok(_) => Self::by_id(tsk.id.as_str(), conn),
                Err(_) => None
            }
    }

    pub fn set_status(task_id: &str, new_status: i32, conn: &mut PgConnection) -> Option<Self> {
        use super::schema::tasks::dsl::status;
        
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

    pub fn filter_by_status(filter_status: i32, conn: &mut PgConnection) -> Vec<Task> {
        use super::schema::tasks::dsl::{status, due, updated_at};
        Task::set_overdues(conn);
        if let Ok(records) = task_dsl
            .filter(status.eq(filter_status))
            .order_by((due.asc(), status.asc(), updated_at.desc()))
            .get_results(conn) {
            records
        } else {
            vec![]
        }
    }


    //global search
    pub fn text_filter(text: &str, conn: &mut PgConnection) -> Vec<Task> {
        use super::schema::tasks::dsl::{name, description};
        let term = format!("%{}%", text);
        let result = task_dsl
            .filter(name.ilike(&term))
            .or_filter(description.ilike(&term))
            .load::<Task>(conn)
            .expect("could not query db");
        sort_by_score(result, text)
    }
    
    pub fn delete_task(trg_id: &str, conn: &mut PgConnection) -> Result<usize, diesel::result::Error> {
        diesel::delete(task_dsl.find(trg_id))
            .execute(conn)
    }
}

#[cfg(test)]
mod task_tests;
