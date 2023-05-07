use actix_web::{
    App,
    web,
    test::{read_body_json, init_service, TestRequest}
};
use actix_rt;
use serde_json::json;
use crate::db::{models::{Task, TaskStatus}, establish_connection};

use super::task::{
    index, 
    create, 
    get_by_id, 
    task_update, 
    set_status, 
    filter_text
};


#[actix_rt::test]
async fn create_task_from_api() {
    let test_name = "endpoint_test_1";
    let test_description = "endpoint_test_1 description";
    let request_body = json!({"name": test_name, "description": test_description, "due": "2023-05-10T23:01:00.000Z"});
    let conn_pool = establish_connection();
    let mut app = init_service(App::new().app_data(web::Data::new(conn_pool)).service(create)).await;
    let resp = TestRequest::post()
        .uri("/create")
        .set_json(&request_body)
        .send_request(&mut app)
        .await;
    assert!(resp.status().is_success(), "Failed to create task");
    let task: Task = read_body_json(resp).await;
    assert_eq!(task.name, test_name);
    assert_eq!(task.description, test_description);
    assert_eq!(task.due, Some(chrono::NaiveDateTime::parse_from_str("2023-05-10T23:01:00.000Z", "%Y-%m-%dT%H:%M:%S%.fZ").unwrap()));
}

#[actix_rt::test]
async fn get_all_tasks_api() {
    let conn_pool = establish_connection();
    let mut app = init_service(App::new().app_data(web::Data::new(conn_pool)).service(index)).await;
    let resp = TestRequest::get()
        .uri("/")
        .send_request(&mut app)
        .await;

    assert!(resp.status().is_success(), "Failed to retrieve tasks");
    let tasks: Vec<Task> = read_body_json(resp).await;
    assert!(tasks.len() > 0);
}

#[actix_rt::test]
async fn retrieve_by_id_api() {
    let conn_pool = establish_connection();
    let mut app = init_service(App::new().app_data(web::Data::new(conn_pool)).service(index).service(get_by_id)).await;
    let resp = TestRequest::get()
        .uri("/")
        .send_request(&mut app)
        .await;

    let tasks: Vec<Task> = read_body_json(resp).await;
    let task = tasks.get(0).unwrap();
    let resp_task = TestRequest::get()
        .uri(format!("/{}", task.id).as_str())
        .send_request(&mut app)
        .await;
    assert!(resp_task.status().is_success(), "Failed to fetch task by id");
    let returned_task: Task = read_body_json(resp_task).await;
    assert_eq!(returned_task.id, task.id);
    assert_eq!(returned_task.name, task.name);
    assert_eq!(returned_task.description, task.description);
    assert_eq!(returned_task.created_at, task.created_at);
}


#[actix_rt::test]
async fn update_task() {
    let conn_pool = establish_connection();
    let mut app = init_service(App::new().app_data(web::Data::new(conn_pool)).service(create).service(task_update)).await;
    let test_name = "endpoint_test_4";
    let test_description = "endpoint_test_4 description";
    let request_body = json!({"name": test_name, "description": test_description, "due": null});
    let resp = TestRequest::post()
        .uri("/create")
        .set_json(&request_body)
        .send_request(&mut app)
        .await;
    let task: Task = read_body_json(resp).await;
    let tsk = json!({
        "id": task.id.clone(),
        "name": "endpoint_test_4_update".to_string(),
        "description": "endpoint_test_4 description update.".to_owned(),
        "due": null,
        "status": TaskStatus::Created.to_store(),
        "created_at":"2023-05-05T11:43:17.082Z",
        "updated_at": "2023-05-05T11:43:17.082Z"
    });

    let resp_upd = TestRequest::put()
        .uri("/")
        .set_json(tsk)
        .send_request(&mut app)
        .await;
    assert!(resp_upd.status().is_success(), "Error updating task");
    let updated_task: Task = read_body_json(resp_upd).await;
    assert_eq!(updated_task.id, task.id);
    assert_eq!(updated_task.name, "endpoint_test_4_update".to_string());
    assert_eq!(updated_task.description, "endpoint_test_4 description update.".to_string());
}

#[actix_rt::test]
async fn set_status_task() {
    let conn_pool = establish_connection();
    let mut app = init_service(App::new().app_data(web::Data::new(conn_pool)).service(create).service(set_status)).await;
    let test_name = "endpoint_test_5";
    let request_body = json!({"name": test_name, "due": null});
    let resp = TestRequest::post()
        .uri("/create")
        .set_json(&request_body)
        .send_request(&mut app)
        .await;
    let task: Task = read_body_json(resp).await;
    let uri = format!("/set/{}/{}", task.id, TaskStatus::Done.to_store());
    let resp_status = TestRequest::get()
        .uri(uri.as_str())
        .send_request(&mut app)
        .await;
    assert!(resp_status.status().is_success(), "Failed to update state");
    let t: Task = read_body_json(resp_status).await;
    assert_eq!(t.status, TaskStatus::Done.to_store());
}

//#[actix_rt::test]
//async fn get_by_status() {
//    let conn_pool = establish_connection();
//    let mut app = init_service(App::new().app_data(web::Data::new(conn_pool)).service(create).service(set_status).service(filter_by_status)).await;
//    let test_name = "endpoint_test_6";
//    let request_body = json!({"name": test_name, "due": null});
//    let resp = TestRequest::post()
//        .uri("/create")
//        .set_json(&request_body)
//        .send_request(&mut app)
//        .await;
//    let task: Task = read_body_json(resp).await;
//    let test_name_1 = "endpoint_test_7";
//    let request_body = json!({"name": test_name_1, "due": null});
//    let resp = TestRequest::post()
//        .uri("/create")
//        .set_json(&request_body)
//        .send_request(&mut app)
//        .await;
//    
//    let task_1: Task = read_body_json(resp).await;
//    let uri = format!("/set/{}/{}", task.id, TaskStatus::Done.to_store());
//    let uri_1 = format!("/set/{}/{}", task_1.id, TaskStatus::Done.to_store());
//    TestRequest::get()
//        .uri(&uri)
//        .send_request(&mut app)
//        .await;
//    TestRequest::get()
//        .uri(&uri_1)
//        .send_request(&mut app)
//        .await;
//
//    let query = format!("/filter?status={}", TaskStatus::Done.to_store());
//    let resp_filtered = TestRequest::get()
//        .uri(&query)
//        .send_request(&mut app)
//        .await;
//    assert!(resp_filtered.status().is_success(), "Failed to filter by status");
//    let body: Vec<Task> = read_body_json(resp_filtered).await;
//    assert_eq!(body[0].status, TaskStatus::Done.to_store());
//    let mut conn = establish_connection().get().unwrap();
//    Task::delete_task(&task.id, &mut conn).unwrap();
//    Task::delete_task(&task_1.id, &mut conn).unwrap();
//
//
//}

#[actix_rt::test]
async fn text_filters() {
    let conn_pool = establish_connection();
    let mut app = init_service(App::new().app_data(web::Data::new(conn_pool)).service(create).service(set_status).service(filter_text)).await;
    let test_name = "aa";
    let request_body = json!({"name": test_name, "due": null});
    let resp = TestRequest::post()
        .uri("/create")
        .set_json(&request_body)
        .send_request(&mut app)
        .await;
    let task: Task = read_body_json(resp).await;
    let test_name_1 = "bb";
    let test_description_1 = "aloha from aaron";
    let request_body = json!({"name": test_name_1, "description": test_description_1, "due": null});
    let resp = TestRequest::post()
        .uri("/create")
        .set_json(&request_body)
        .send_request(&mut app)
        .await;

    let task_1: Task = read_body_json(resp).await;
    let test_name_2 = "c";
    let request_body = json!({"name": test_name_2, "due": null});
    let resp = TestRequest::post()
        .uri("/create")
        .set_json(&request_body)
        .send_request(&mut app)
        .await;

    let task_2: Task = read_body_json(resp).await;
    let query ="/filter?term=aa";
    let query_result = TestRequest::get()
        .uri(query)
        .send_request(&mut app)
        .await;

    assert!(query_result.status().is_success(), "Failed to filter by status");
    let body: Vec<Task> = read_body_json(query_result).await;
    
    assert_eq!(body[0].id, task.id);
    assert_eq!(body[1].id, task_1.id);
    assert_eq!(body.len(), 2);
    let mut conn = establish_connection().get().unwrap();
    Task::delete_task(&task.id, &mut conn).unwrap();
    Task::delete_task(&task_1.id, &mut conn).unwrap();
    Task::delete_task(&task_2.id, &mut conn).unwrap();
}
