use rmcp::{
    ServerHandler,
    handler::server::router::tool::ToolRouter,
    model::*,
    tool, tool_handler, tool_router,
};
use crate::agents::{get_agent, ensure_agents_initialized, AgentMessage};

pub struct PatternClockMCP {
    tool_router: ToolRouter<PatternClockMCP>,
}

#[tool_router]
impl PatternClockMCP {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    /// Example tool that returns a greeting
    #[tool(description = "A simple example tool that returns a greeting message")]
    pub async fn example_tool(&self) -> String {
        "Hello from pattern-clock MCP server!".to_string()
    }
}

#[tool_handler]
impl ServerHandler for PatternClockMCP {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}

// Public functions that can be called directly from the app (not just via MCP protocol)
impl PatternClockMCP {
    /// Call the example tool directly (for use by desktop app)
    pub async fn call_example_tool(&self) -> String {
        self.example_tool().await
    }

    /// Process agent directly (for use by desktop app)
    pub async fn call_process_agent(&self, agent_id: u8, data: String) -> String {
        if agent_id < 1 || agent_id > 5 {
            return format!("Error: agent_id must be between 1 and 5, got {}", agent_id);
        }

        if let Err(e) = ensure_agents_initialized().await {
            return format!("Error: Failed to initialize agents: {}", e);
        }

        if let Some(actor_ref) = get_agent(agent_id) {
            actor_ref.send_message(AgentMessage::ProcessData {
                data: data.clone(),
            });
            format!("Message queued for Agent{}: {}", agent_id, data)
        } else {
            format!("Error: Agent{} is not available", agent_id)
        }
    }
}
