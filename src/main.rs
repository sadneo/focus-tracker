use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::{Seek, SeekFrom};
use std::path::PathBuf;
use std::process::Command;
use std::time::SystemTime;

use tokio::fs;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Deserialize, Serialize)]
struct EventObject {
    change_type: String,
    timestamp: SystemTime,
    id: i64,
    app_id: String,
}

#[tokio::main]
async fn main() {
    let log_path = PathBuf::from(std::env::var("XDG_STATE_HOME").expect("$XDG_STATE_HOME invalid"))
        .join("focus-tracker.json");
    println!("{}", log_path.display());

    match fs::try_exists(&log_path).await {
        Ok(true) => (),
        Ok(false) => {
            if let Err(e) = fs::write(&log_path, b"[]").await {
                eprintln!("error: {}", e);
            }
        },
        Err(e) => eprintln!("error: {}", e),
    }

    loop {
        let sub_result = Command::new("swaymsg")
            .arg("-t")
            .arg("subscribe")
            .arg("[\"window\"]")
            .output()
            .expect("swaymsg subscribe failed");

        let object: HashMap<String, Value> =
            serde_json::from_slice(&sub_result.stdout).expect("serde from_json failed");
        add_log(object, &log_path).await.expect("add log failed");
    }
}

async fn add_log(object: HashMap<String, Value>, path: &PathBuf) -> std::io::Result<()> {
    let change_type = object.get("change").unwrap().as_str().unwrap().to_owned();
    if !matches!(change_type.as_str(), "new" | "close" | "focus") {
        return Ok(());
    }

    let timestamp = SystemTime::now();
    let container = object
        .get("container")
        .unwrap()
        .as_object()
        .unwrap();
    let id = container.get("id")
        .unwrap()
        .as_i64()
        .unwrap();
    let app_id = container.get("app_id")
        .unwrap()
        .as_str()
        .unwrap()
        .to_owned();

    println!("{change_type} {id} {app_id}");
    let new_event = EventObject {
        change_type,
        timestamp,
        id,
        app_id,
    };

    // tokio here
    tokio::task::spawn_blocking(move || {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;

        let mut events: Vec<EventObject> = serde_json::from_reader(&file).unwrap_or_default();
        events.push(new_event);
        file.set_len(0)?;
        file.seek(SeekFrom::Start(0))?;
        serde_json::to_writer_pretty(&file, &events)?;
    }).await.unwrap();
    Ok(())
}
