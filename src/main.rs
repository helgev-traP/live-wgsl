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
    #[arg(default_value = "live.wgsl")]
    path: String,
    #[arg(short, long, default_value = "false")]
    n: bool,
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
    let event_loop: EventLoop<String> = EventLoop::with_user_event().build().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    let proxy = event_loop.create_proxy();

    // spawn a task to poll for new fragment code
    let args = Args::parse();

    let file_path = args.path;
    let interval = args.interval;

    // check if the file exists
    if !std::path::Path::new(&file_path).exists() {
        if args.n {
            // copy from the default fragment code
            std::fs::write(&file_path, default_fragment_code).unwrap();
        } else {
            eprintln!("File does not exist: {}", file_path);
            std::process::exit(1);
        }
    }

    // execute vscode to open the file(this do not works now)

    // #[cfg(not(target_arch = "wasm32"))]
    // if args.code {
    //     // wgslへのフルパスを構築
    //     let current = std::env::current_dir().unwrap();
    //     let file_path = current.join(&file_path);

    //     std::process::Command::new("code")
    //         .arg(&file_path)
    //         .spawn()
    //         .expect("Failed to open the file in vscode");
    // }

    // show info
    println!("Shader file: {}", file_path);
    println!("Polling interval: {} ms", interval);

    tokio::spawn(poll_new_fragment_code::poll_new_fragment_code(
        file_path, proxy, interval,
    ));

    event_loop
        .run_app(&mut App::new(default_fragment_code))
        .unwrap();
}
