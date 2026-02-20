// Web application component - simple placeholder

#[cfg(any(feature = "web", feature = "server"))]
use dioxus::prelude::*;

#[cfg(any(feature = "web", feature = "server"))]
const FAVICON: Asset = asset!("/assets/favicon.ico");

/// Web application root component
#[cfg(any(feature = "web", feature = "server"))]
#[component]
pub fn WebApp() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        div {
            font_family: "system-ui, sans-serif",
            header {
                h1 { "pattern-clock" }
                div { "Web Interface" }
            }
        }
    }
}
