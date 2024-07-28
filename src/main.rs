use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use mongodb::{bson::{doc, oid::ObjectId}, sync::Client};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct TodoItem {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<ObjectId>,
    title: String,
    completed: bool,
}

#[derive(Deserialize)]
struct TodoItemInput {
    title: String,
    completed: bool,
}

async fn get_todos(client: web::Data<Client>) -> impl Responder {
    let collection = client.database("todo_db").collection::<TodoItem>("todos");
    let cursor = collection.find(None, None).unwrap();
    let todos: Vec<TodoItem> = cursor.map(|doc| doc.unwrap()).collect();
    HttpResponse::Ok().json(todos)
}

async fn create_todo(
    client: web::Data<Client>,
    item: web::Json<TodoItemInput>,
) -> impl Responder {
    let collection = client.database("todo_db").collection::<TodoItem>("todos");
    let new_item = TodoItem {
        id: None,
        title: item.title.clone(),
        completed: item.completed,
    };
    let result = collection.insert_one(new_item, None).unwrap();
    let inserted_id = result.inserted_id.as_object_id().unwrap();
    HttpResponse::Ok().json(inserted_id)
}

async fn update_todo(
    client: web::Data<Client>,
    id: web::Path<String>,
    item: web::Json<TodoItemInput>,
) -> impl Responder {
    let collection = client.database("todo_db").collection::<TodoItem>("todos");
    let obj_id = ObjectId::with_string(&id).unwrap();
    let filter = doc! { "_id": obj_id };
    let update = doc! {
        "$set": {
            "title": &item.title,
            "completed": &item.completed,
        }
    };
    let update_result = collection.update_one(filter, update, None).unwrap();
    if update_result.matched_count == 1 {
        HttpResponse::Ok().body("Todo item updated")
    } else {
        HttpResponse::NotFound().body("Todo item not found")
    }
}

async fn delete_todo(
    client: web::Data<Client>,
    id: web::Path<String>,
) -> impl Responder {
    let collection = client.database("todo_db").collection::<TodoItem>("todos");
    let obj_id = ObjectId::with_string(&id).unwrap();
    let filter = doc! { "_id": obj_id };
    let delete_result = collection.delete_one(filter, None).unwrap();
    if delete_result.deleted_count == 1 {
        HttpResponse::Ok().body("Todo item deleted")
    } else {
        HttpResponse::NotFound().body("Todo item not found")
    }
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let client = Client::with_uri_str("mongodb://localhost:27017").unwrap();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(client.clone()))
            .route("/todos", web::get().to(get_todos))
            .route("/todos", web::post().to(create_todo))
            .route("/todos/{id}", web::put().to(update_todo))
            .route("/todos/{id}", web::delete().to(delete_todo))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
