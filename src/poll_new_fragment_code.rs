use std::time::SystemTime;

// check if the fragment code is updated every x ms, if it is, send a message to the main thread
pub async fn poll_new_fragment_code(
    file_path: String,
    channel: winit::event_loop::EventLoopProxy<(Option<SystemTime>, String)>,
    interval: u64,
) {
    let mut last_modified;

    loop {
        if let Ok(metadata) = std::fs::metadata(&file_path) {
            if let Ok(modified) = metadata.modified() {
                last_modified = modified;
                break;
            }
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(interval)).await;
    }

    let code = std::fs::read_to_string(&file_path).unwrap();
    send_code_update(None, code, &channel);

    loop {
        tokio::time::sleep(tokio::time::Duration::from_millis(interval)).await;

        if let Some(code) = is_code_updated(&file_path, &mut last_modified) {
            send_code_update(Some(last_modified), code, &channel);
        }
    }
}

fn is_code_updated(path: &str, last_modified: &mut std::time::SystemTime) -> Option<String> {
    let Ok(metadata) = std::fs::metadata(path) else {
        eprintln!("\n\nError when getting metadata: {}", path);
        eprintln!("File does not exist or permission denied.\n");
        std::process::exit(1);
    };

    let Ok(modified) = metadata.modified() else {
        return None;
    };

    if modified == *last_modified {
        return None;
    }

    *last_modified = modified;

    std::fs::read_to_string(path)
        .map_err(|e| eprintln!("\n\nError when reading file: {}\n\n{}", path, e))
        .ok()
}

fn send_code_update(
    time: Option<SystemTime>,
    string: String,
    channel: &winit::event_loop::EventLoopProxy<(Option<SystemTime>, String)>,
) {
    channel
        .send_event((time, string))
        .map_err(|e| eprintln!("\n\nError when sending event: {}\n\n", e))
        .ok();
}
