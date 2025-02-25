// check if the fragment code is updated every x ms, if it is, send a message to the main thread
pub async fn poll_new_fragment_code(
    file_path: String,
    channel: winit::event_loop::EventLoopProxy<String>,
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

    // first event
    get_and_send_code(&file_path, &mut last_modified, &channel, true);

    loop {
        tokio::time::sleep(tokio::time::Duration::from_millis(interval)).await;

        get_and_send_code(&file_path, &mut last_modified, &channel, false);
    }
}

fn get_and_send_code(
    path: &str,
    last_modified: &mut std::time::SystemTime,
    channel: &winit::event_loop::EventLoopProxy<String>,
    force: bool,
) {
    let Ok(metadata) = std::fs::metadata(path) else {
        return;
    };

    let Ok(modified) = metadata.modified() else {
        return;
    };

    if modified == *last_modified && !force {
        return;
    }

    *last_modified = modified;

    let Ok(code) = std::fs::read_to_string(path) else {
        return;
    };

    if let Err(e) = channel.send_event(code) {
        eprintln!("Error when sending event: {}", e);
        panic!();
    }
}
