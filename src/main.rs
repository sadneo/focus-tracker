use std::collections::HashMap;
use std::io::SeekFrom;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;
use std::time::SystemTime;

use tokio::fs::{self, OpenOptions};
use tokio::io::{AsyncSeekExt, AsyncWriteExt};
use tokio::sync::Mutex;

use axum::{Json, Router, extract::State, routing::get};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tower_http::services::ServeDir;
use tower_http::cors::CorsLayer;

#[derive(Deserialize, Serialize)]
struct EventObject {
    change_type: String,
    timestamp: SystemTime,
    id: i64,
    app_id: String,
}

type EventState = Arc<Mutex<Vec<EventObject>>>;

#[tokio::main]
async fn main() {
    let log_path = PathBuf::from(std::env::var("XDG_STATE_HOME").expect("$XDG_STATE_HOME invalid"))
        .join("focus-tracker.json");

    let events: Vec<EventObject> = match fs::try_exists(&log_path).await {
        Ok(true) => {
            let file_contents = fs::read(&log_path).await.unwrap();
            let events: Vec<EventObject> =
                serde_json::from_slice(&file_contents).unwrap_or_default();
            events
        }
        Ok(false) => {
            if let Err(e) = fs::write(&log_path, b"[]").await {
                eprintln!("error: {}", e);
                return;
            }
            vec![]
        }
        Err(e) => {
            eprintln!("error: {}", e);
            return;
        }
    };
    let events = Arc::new(Mutex::new(events));

    let app: Router<()> = Router::new()
        .route(
            "/api/data",
            get(|State(state): State<EventState>| async move {
                let events_guard = state.lock().await;
                let ser_contents = serde_json::to_string_pretty(&*events_guard).unwrap();
                Json(ser_contents)
            }),
        )
        .fallback_service(ServeDir::new("frontend/dist"))
        .layer(CorsLayer::very_permissive())
        .with_state(events.clone());

    let listener = tokio::net::TcpListener::bind("localhost:3000")
        .await
        .unwrap();
    tokio::task::spawn(async move { axum::serve(listener, app).await });

    loop {
        let sub_result = Command::new("swaymsg")
            .arg("-t")
            .arg("subscribe")
            .arg("[\"window\"]")
            .output()
            .expect("swaymsg subscribe failed");

        let object: HashMap<String, Value> =
            serde_json::from_slice(&sub_result.stdout).expect("serde from_json failed");
        add_log(object, &log_path, events.clone())
            .await
            .expect("add log failed");
    }
}

async fn add_log(
    object: HashMap<String, Value>,
    path: &PathBuf,
    events: EventState,
) -> std::io::Result<()> {
    let change_type = object.get("change").unwrap().as_str().unwrap().to_owned();
    if !matches!(change_type.as_str(), "new" | "close" | "focus") {
        return Ok(());
    }

    let timestamp = SystemTime::now();
    let container = object.get("container").unwrap().as_object().unwrap();
    let id = container.get("id").unwrap().as_i64().unwrap();
    let app_id = match container.get("app_id").unwrap() {
        Value::String(value) => value.to_owned(),
        Value::Null => container.get("window_properties").unwrap().get("class").unwrap().as_str().unwrap().to_owned(),
        _ => panic!(),
    };

    println!("{change_type} {id} {app_id}");
    let new_event = EventObject {
        change_type,
        timestamp,
        id,
        app_id,
    };

    #[allow(clippy::suspicious_open_options)]
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(path)
        .await?;

    let mut events_guard = events.lock().await;
    events_guard.push(new_event);
    file.set_len(0).await?;
    file.seek(SeekFrom::Start(0)).await?;
    let ser_contents = serde_json::to_vec_pretty(&*events_guard)?;
    file.write_all(&ser_contents).await?;

    Ok(())
}
