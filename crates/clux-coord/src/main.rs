use std::sync::Arc;

use clux_coord::mcp_bridge::{start_server, CoordState};

#[tokio::main]
async fn main() {
    let port: u16 = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    let state = Arc::new(CoordState::new());
    let addr = start_server(state, port).await.expect("Failed to start server");
    eprintln!("MCP server listening on http://{addr}/mcp");

    // Keep running forever
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
    }
}
