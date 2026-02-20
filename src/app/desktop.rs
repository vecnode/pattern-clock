// Desktop application component

#[cfg(feature = "desktop")]
use dioxus::prelude::*;
#[cfg(feature = "desktop")]
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
#[cfg(feature = "desktop")]
use std::time::Duration;
#[cfg(feature = "desktop")]
use burn::backend::{Autodiff, wgpu::Wgpu};

#[cfg(feature = "desktop")]
use crate::shared::{SystemInfo, echo_server};

// Global cognitive cycle state
#[cfg(feature = "desktop")]
static COGNITIVE_CYCLE_STATE: AtomicBool = AtomicBool::new(false);
// Global cognitive cycle counter
#[cfg(feature = "desktop")]
static COGNITIVE_CYCLE_COUNTER: AtomicU64 = AtomicU64::new(0);

#[cfg(feature = "desktop")]
const FAVICON: Asset = asset!("/assets/favicon.ico");
#[cfg(feature = "desktop")]
const MAIN_CSS: Asset = asset!("/assets/main.css");

/// Desktop application root component
#[cfg(feature = "desktop")]
#[component]
pub fn DesktopApp() -> Element {
    let mut cycle_state = use_signal(|| COGNITIVE_CYCLE_STATE.load(Ordering::SeqCst));
    
    use_effect(move || {
        spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(50));
            loop {
                interval.tick().await;
                if COGNITIVE_CYCLE_STATE.load(Ordering::SeqCst) {
                    COGNITIVE_CYCLE_COUNTER.fetch_add(1, Ordering::Relaxed);
                }
            }
        });
    });
    
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        DesktopHeader {}
        br {}
        SystemInfo {}
        br {}
        div {
            id: "app-header",
            width: "40%",
            button {
                onclick: move |_| {
                    let new_state = !COGNITIVE_CYCLE_STATE.load(Ordering::SeqCst);
                    COGNITIVE_CYCLE_STATE.store(new_state, Ordering::SeqCst);
                    cycle_state.set(new_state);
                    println!("cognitive_cycle_state={}", new_state);
                },
                if cycle_state() { "Stop CogCycle" } else { "Start CogCycle" }
            }
            div {
                width: "20px",
                height: "20px",
                background_color: if cycle_state() { "#006400" } else { "#8B0000" },
                margin_left: "10px",
            }
        }
        br {}
        div {
            id: "app-header",
            width: "40%",
            button {
                onclick: move |_| {
                    println!("Building LSTM model");
                    type Backend = Autodiff<Wgpu>;
                    let device = Default::default();
                    let config = crate::lstm::LstmConfig::default();
                    let lstm = crate::lstm::Lstm::<Backend>::new(config, &device);
                    println!("{:#?}", lstm);
                },
                "Build LSTM"
            }
        }
        br {}
        DesktopEcho {}
        br {}
        DesktopMCP {}
    }
}

#[cfg(feature = "desktop")]
#[component]
fn DesktopHeader() -> Element {
    rsx! {
        div {
            id: "app-header",
            "pattern-clock - Desktop"
        }
    }
}

/// Echo component that demonstrates fullstack server functions (Desktop)
#[cfg(feature = "desktop")]
#[component]
fn DesktopEcho() -> Element {
    let mut response = use_signal(|| String::new());

    rsx! {
        div {
            id: "echo",
            p { "ServerFn Echo" }
            br {}
            input {
                placeholder: "Type here to echo...",
                oninput:  move |event| async move {
                    let data = echo_server(event.value()).await.unwrap();
                    response.set(data);
                },
            }

            if !response().is_empty() {
                p {
                    "Server echoed: "
                    i { "{response}" }
                }
            }
        }
    }
}

/// MCP Server component for testing MCP tools
#[cfg(feature = "desktop")]
#[component]
fn DesktopMCP() -> Element {
    let mut mcp_response = use_signal(|| String::new());
    let mut is_loading = use_signal(|| false);

    rsx! {
        div {
            id: "mcp-test",
            p { "MCP Server Test" }
            br {}
            button {
                disabled: is_loading(),
                onclick: move |_| {
                    is_loading.set(true);
                    mcp_response.set(String::new());
                    spawn(async move {
                        let mcp_server = crate::mcp_server::PatternClockMCP::new();
                        match mcp_server.call_example_tool().await {
                            result => {
                                mcp_response.set(result);
                                is_loading.set(false);
                            }
                        }
                    });
                },
                if is_loading() { "Loading..." } else { "Call MCP Example Tool" }
            }
            br {}
            if !mcp_response().is_empty() {
                p {
                    "MCP Response: "
                    i { "{mcp_response}" }
                }
            }
        }
    }
}
