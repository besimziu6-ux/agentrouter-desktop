mod server;

use std::net::SocketAddr;
use std::path::PathBuf;

use clap::Parser;
use server::build_router;
use tauri::Manager;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long)]
    port: Option<u16>,
    #[arg(long, default_value_t = false)]
    serve: bool,
}

fn resolve_dist_dir() -> PathBuf {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let cwd_dist = cwd.join("dist");
    if cwd_dist.join("index.html").exists() {
        return cwd_dist;
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let exe_dist = dir.join("dist");
            if exe_dist.join("index.html").exists() {
                return exe_dist;
            }
            if let Some(root) = dir.parent() {
                for cand in [
                    root.join("lib/agentrouter-desktop"),
                    root.join("lib/AgentRouter"),
                    root.join("share/agentrouter-desktop"),
                ] {
                    if cand.join("index.html").exists() {
                        return cand;
                    }
                    let nested = cand.join("dist");
                    if nested.join("index.html").exists() {
                        return nested;
                    }
                }
            }
        }
    }
    cwd_dist
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let dist_dir = resolve_dist_dir();
    if args.serve {
        let port = args.port.unwrap_or(8787);
        run_server(port, dist_dir).await;
        return;
    }
    let listener = match tokio::net::TcpListener::bind("127.0.0.1:0").await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("failed to bind ephemeral port: {e}");
            std::process::exit(1);
        }
    };
    let port = listener.local_addr().map(|a| a.port()).unwrap_or(8787);
    let router = build_router(dist_dir);
    tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, router).await {
            eprintln!("server error: {e}");
        }
    });
    let url = format!("http://127.0.0.1:{port}");
    println!("serving on {url}");
    if let Err(e) = tauri::Builder::default()
        .setup(move |app| {
            if let Some(w) = app.handle().get_webview_window("main") {
                let parsed: tauri::Url = url.parse().expect("valid localhost url");
                w.navigate(parsed)?;
            }
            Ok(())
        })
        .run(tauri::generate_context!())
    {
        eprintln!("tauri error: {e}");
        std::process::exit(1);
    }
}

async fn run_server(port: u16, dist_dir: PathBuf) {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("failed to bind {addr}: {e} (is port {port} already in use?)");
            std::process::exit(1);
        }
    };
    println!("serving on http://127.0.0.1:{port}");
    let router = build_router(dist_dir);
    if let Err(e) = axum::serve(listener, router).await {
        eprintln!("server error: {e}");
        std::process::exit(1);
    }
}
