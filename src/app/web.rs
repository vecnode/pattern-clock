// Web application component

#[cfg(any(feature = "web", feature = "server"))]
use dioxus::prelude::*;

#[cfg(any(feature = "web", feature = "server"))]
const FAVICON: Asset = asset!("/assets/favicon.ico");

/// Web application root component
/// Web app acts as MCP client - subscribes to MCP stream for real-time results
#[cfg(any(feature = "web", feature = "server"))]
#[component]
pub fn WebApp() -> Element {
    let mut mcp_results = use_signal(|| Vec::<String>::new());
    
    // Subscribe to MCP channel when component mounts (long-polling)
    use_effect(move || {
        spawn(async move {
            loop {
                match crate::shared::mcp_receive().await {
                    Ok(result) => {
                        if !result.is_empty() {
                            eprintln!("[Web] Received MCP result: {}", result);
                            mcp_results.with_mut(|results| {
                                results.push(result);
                                // Keep only last 10 results
                                if results.len() > 10 {
                                    results.remove(0);
                                }
                            });
                        }
                        // Immediately poll again for next result
                    }
                    Err(e) => {
                        eprintln!("[Web] MCP receive error: {}, retrying...", e);
                        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
                    }
                }
            }
        });
    });
    
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        div {
            font_family: "system-ui, sans-serif",
            header {
                h1 { "pattern-clock" }
                div { "Web Interface - MCP Client (Streaming)" }
            }
            div {
                margin_top: "20px",
                padding: "10px",
                background_color: "#e8f5e9",
                border_radius: "5px",
                p {
                    "Connected to MCP stream - waiting for results from desktop app..."
                }
            }
            div {
                margin_top: "20px",
                h3 { 
                    "MCP Results (from Desktop App): "
                    span {
                        color: if mcp_results().is_empty() { "#999" } else { "#4caf50" },
                        "({mcp_results().len()} received)"
                    }
                }
                if mcp_results().is_empty() {
                    div {
                        padding: "10px",
                        background_color: "#fff3cd",
                        border_radius: "5px",
                        border_left: "4px solid #ffc107",
                        p {
                            "No results yet. Click MCP buttons in the desktop app to see results here."
                        }
                    }
                } else {
                    for (idx, result) in mcp_results().iter().enumerate() {
                        div {
                            key: "{idx}",
                            margin_top: "10px",
                            padding: "10px",
                            background_color: "#f0f0f0",
                            border_radius: "5px",
                            border_left: "4px solid #4caf50",
                            p {
                                strong { "Result " }
                                "{idx + 1}"
                                strong { ": " }
                                "{result}"
                            }
                        }
                    }
                }
            }
        }
    }
}
