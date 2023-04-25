use actix_web::{
    App,
    web,
    test::{read_body_json, init_service, TestRequest}
};
use actix_rt;
use serde_json::json;
use crate::{db::{models::Task, establish_connection}};

use super::task::{index, create, get_by_id, task_update, set_status, filter_by_status};

#[actix_rt::test]
async fn create_task_from_api() {
    let test_name = "endpoint_test_1";
    let test_description = "endpoint_test_1 description";
    let request_body = json!({"name": test_name, "description": test_description});
    let conn_pool = establish_connection();
    let mut app = init_service(App::new().app_data(web::Data::new(conn_pool)).service(create)).await;
    let resp = TestRequest::post()
        .uri("/")
        .set_json(&request_body)
        .send_request(&mut app)
        .await;
    assert!(resp.status().is_success(), "Failed to create task");
    let task: Task = read_body_json(resp).await;
    assert_eq!(task.name, test_name);
    assert_eq!(task.description, test_description);
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
    let request_body = json!({"name": test_name, "description": test_description});
    let resp = TestRequest::post()
        .uri("/")
        .set_json(&request_body)
        .send_request(&mut app)
        .await;
    let mut task: Task = read_body_json(resp).await;
    task.name = "endpoint_test_4_update".to_string();
    let ser = serde_json::to_string(&task).unwrap();
    let val: serde_json::Value = serde_json::from_str(ser.as_str()).unwrap();
    let resp_upd = TestRequest::put()
        .uri("/")
        .set_json(val)
        .send_request(&mut app)
        .await;
    assert!(resp_upd.status().is_success(), "Error updating task");
    let updated_task: Task = read_body_json(resp_upd).await;
    assert_eq!(updated_task.id, task.id);
    assert_eq!(updated_task.name, "endpoint_test_4_update".to_string());
    assert_eq!(updated_task.description, "endpoint_test_4 description".to_string());
}

#[actix_rt::test]
async fn set_status_task() {
    let conn_pool = establish_connection();
    let mut app = init_service(App::new().app_data(web::Data::new(conn_pool)).service(create).service(set_status)).await;
    let test_name = "endpoint_test_5";
    let request_body = json!({"name": test_name});
    let resp = TestRequest::post()
        .uri("/")
        .set_json(&request_body)
        .send_request(&mut app)
        .await;
    let task: Task = read_body_json(resp).await;
    let uri = format!("/set/{}/done", task.id);
    let resp_status = TestRequest::get()
        .uri(uri.as_str())
        .send_request(&mut app)
        .await;
    assert!(resp_status.status().is_success(), "Failed to update state");
    let t: Task = read_body_json(resp_status).await;
    assert_eq!(t.status, "done");
}

#[actix_rt::test]
async fn get_by_status() {
    let conn_pool = establish_connection();
    let mut app = init_service(App::new().app_data(web::Data::new(conn_pool)).service(create).service(set_status).service(filter_by_status)).await;
    let test_name = "endpoint_test_6";
    let request_body = json!({"name": test_name});
    let resp = TestRequest::post()
        .uri("/")
        .set_json(&request_body)
        .send_request(&mut app)
        .await;
    let task: Task = read_body_json(resp).await;
    let test_name_1 = "endpoint_test_7";
    let request_body = json!({"name": test_name_1});
    let resp = TestRequest::post()
        .uri("/")
        .set_json(&request_body)
        .send_request(&mut app)
        .await;

    let task_1: Task = read_body_json(resp).await;
    let uri = format!("/set/{}/endp_test", task.id);
    let uri_1 = format!("/set/{}/endp_test", task_1.id);
    TestRequest::get()
        .uri(&uri)
        .send_request(&mut app)
        .await;
    TestRequest::get()
        .uri(&uri_1)
        .send_request(&mut app)
        .await;

    let query = format!("/filter?status=endp_test");
    let resp_filtered = TestRequest::get()
        .uri(&query)
        .send_request(&mut app)
        .await;
    assert!(resp_filtered.status().is_success(), "Failed to filter by status");
    let body: Vec<Task> = read_body_json(resp_filtered).await;
    assert_eq!(body.len(), 2);
    assert_eq!(body[0].status, "endp_test");
}
