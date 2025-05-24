use std::time::SystemTime;

use clap::Parser;
use window::App;
use winit::event_loop::{ControlFlow, EventLoop};

mod gpu;
mod poll_new_fragment_code;
mod window;

// args parsing
#[derive(Parser)]
#[command(name = "live-wgsl", version = "0.1.0")]
struct Args {
    #[arg(short, long, default_value = "live.wgsl")]
    path: String,
    // #[arg(short, long, default_value = "true")]
    // new: bool,
    #[arg(short, long, default_value = "200")]
    interval: u64,
    // #[arg(short, long, default_value = "false")]
    // code: bool,
}

#[tokio::main]
async fn main() {
    // assets
    let default_fragment_code = include_str!("./fragment_default.wgsl");

    // build event loop
    let event_loop: EventLoop<(Option<SystemTime>, String)> =
        EventLoop::with_user_event().build().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    let proxy = event_loop.create_proxy();

    // spawn a task to poll for new fragment code
    let args = Args::parse();

    let file_path = args.path;
    let interval = args.interval;

    // check if the file exists
    if !std::path::Path::new(&file_path).exists() {
        // copy from the default fragment code
        std::fs::write(&file_path, default_fragment_code).unwrap();
    }

    // show info

    println!();
    println!("Edit shader file with your favorite editor!");
    println!(
        "Shader file: {}",
        std::fs::canonicalize(&file_path).unwrap().display()
    );
    println!();
    println!("Polling interval: {} ms\n", interval);

    println!("-------------------------------\n");

    tokio::spawn(poll_new_fragment_code::poll_new_fragment_code(
        file_path, proxy, interval,
    ));

    event_loop
        .run_app(&mut App::new(default_fragment_code))
        .unwrap();
}
