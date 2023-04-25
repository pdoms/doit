use crate::db::{establish_connection, models::Task};
use std::{thread, time};
use serial_test::serial;

#[test]
#[serial]
fn create_task_with_description() {
    let mut conn = establish_connection().get().unwrap();
    let name = "test_1";
    let description = Some("test 1 description");
    let dt = chrono::Local::now().naive_local();
    let task = Task::create(name, description, Some(dt), &mut conn);
    let result = task.unwrap();
    assert_eq!(result.name.as_str(), name);
    assert_eq!(result.description.as_str(), description.unwrap());
}

#[test]
#[serial]
fn create_task_without_description() {
    let mut conn = establish_connection().get().unwrap();
    let name = "test_2";
    let description = None;
    let task = Task::create(name, description, None, &mut conn);
    let result = task.unwrap();
    assert_eq!(result.description.as_str(), "");
}

#[test]
#[serial]
fn retrieve_by_name() {
    let mut conn = establish_connection().get().unwrap();
    let name = "test_3";
    let _task1 = Task::create(name, None, None, &mut conn);
    let task = Task::by_name(name, &mut conn);
    let result = task.unwrap();
    assert_eq!(result.name.as_str(), name);
    assert_eq!(result.description.as_str(), "");
}

#[test]
#[serial]
fn retrieve_by_id() {
    let mut conn = establish_connection().get().unwrap();
    let name= "test_4";
    let task_init = Task::create(name, None, None, &mut conn).unwrap();
    let task = Task::by_id(task_init.id.as_str(), &mut conn);
    let result = task.unwrap();
    assert_eq!(result.name.as_str(), name);
    assert_eq!(result.description.as_str(), "");
    
}
#[test]
#[serial]
fn retrieve_all() {
    let mut conn = establish_connection().get().unwrap();
    let name= "test_5";
    let _task_init = Task::create(name, None, None, &mut conn).unwrap();
    let tasks=Task::list(&mut conn);
    assert!(tasks.len() >= 1);
}

#[test]
#[serial]
fn do_update() {
    let mut conn = establish_connection().get().unwrap();
    let name= "test_6";
    let description = "test 6 description";
    let task_init = Task::create(name, Some(description), None, &mut conn).unwrap();
    let task = Task::by_id(task_init.id.as_str(), &mut conn).unwrap();
    let mut update = Task::new("test_6_upd", Some("test 6 description update."), None);
    update.id = task.id.clone();
    let result = Task::update(update, &mut conn).unwrap();
    assert_eq!(result.name.as_str(), "test_6_upd");
    assert_eq!(result.description.as_str(), "test 6 description update.");
}

#[test]
#[serial]
fn change_status() {
    let mut conn = establish_connection().get().unwrap();
    let task_init = Task::create("test_7",None, None, &mut conn).unwrap();
    let result = Task::set_status(&task_init.id, "done", &mut conn).unwrap();
    assert_eq!(result.status, "done".to_string());
}

#[test]
#[serial]
fn filter_by_status() {
    let mut conn = establish_connection().get().unwrap();
    let task_init_1 = Task::create("test_8", None, None, &mut conn).unwrap();
    let _task_init_2 = Task::create("test_9", None, None, &mut conn).unwrap();
    let _result = Task::set_status(&task_init_1.id, "test", &mut conn);
    let query_result = Task::filter_by_status("test", &mut conn);
    assert_eq!(query_result.len(), 1);
    assert_eq!(query_result[0].status, "test".to_string());
}

#[test]
#[serial]
fn test_overdue() {
    let due = chrono::Local::now().naive_local();
    let two_sec = time::Duration::from_secs(2);
    thread::sleep(two_sec);
    let mut conn = establish_connection().get().unwrap();
    let task_init = Task::create("test_11", None, Some(due), &mut conn).unwrap();
    assert_eq!(task_init.status, "overdue");
}

