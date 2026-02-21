// Shared API server functions

use dioxus::prelude::*;
use crate::agents::{get_agent, ensure_agents_initialized};
use std::sync::OnceLock;
use tokio::sync::broadcast;

// MCP broadcast channel for streaming results to web clients
static MCP_BROADCASTER: OnceLock<broadcast::Sender<String>> = OnceLock::new();

fn get_mcp_broadcaster() -> broadcast::Sender<String> {
    MCP_BROADCASTER.get_or_init(|| {
        let (tx, _) = broadcast::channel(100);
        tx
    }).clone()
}

/// Echo the user input on the server.
#[post("/api/echo")]
pub async fn echo_server(input: String) -> Result<String, ServerFnError> {
    // Ensure agents are initialized
    let _ = ensure_agents_initialized().await;
    
    // For backward compatibility, send to Agent1
    if let Some(actor_ref) = get_agent(1) {
        use crate::agents::AgentMessage;
        let _ = actor_ref.send_message(AgentMessage::ProcessData {
            data: input.clone(),
        });
    }
    
    Ok(input)
}

// ============================================================================
// HTTP/REST API Endpoints for Multi-Agent System
// ============================================================================

/// Process data through Agent 1
#[post("/api/agents/1/process")]
pub async fn process_agent1(data: String) -> Result<String, ServerFnError> {
    ensure_agents_initialized().await
        .map_err(|e| ServerFnError::new(format!("Failed to initialize agents: {}", e)))?;
    
    if let Some(actor_ref) = get_agent(1) {
        use crate::agents::AgentMessage;
        actor_ref.send_message(AgentMessage::ProcessData {
            data: data.clone(),
        });
        Ok(format!("Message queued for Agent1: {}", data))
    } else {
        Err(ServerFnError::new("Agent1 is not available"))
    }
}

/// Process data through Agent 2
#[post("/api/agents/2/process")]
pub async fn process_agent2(data: String) -> Result<String, ServerFnError> {
    ensure_agents_initialized().await
        .map_err(|e| ServerFnError::new(format!("Failed to initialize agents: {}", e)))?;
    
    if let Some(actor_ref) = get_agent(2) {
        use crate::agents::AgentMessage;
        actor_ref.send_message(AgentMessage::ProcessData {
            data: data.clone(),
        });
        Ok(format!("Message queued for Agent2: {}", data))
    } else {
        Err(ServerFnError::new("Agent2 is not available"))
    }
}

/// Process data through Agent 3
#[post("/api/agents/3/process")]
pub async fn process_agent3(data: String) -> Result<String, ServerFnError> {
    ensure_agents_initialized().await
        .map_err(|e| ServerFnError::new(format!("Failed to initialize agents: {}", e)))?;
    
    if let Some(actor_ref) = get_agent(3) {
        use crate::agents::AgentMessage;
        actor_ref.send_message(AgentMessage::ProcessData {
            data: data.clone(),
        });
        Ok(format!("Message queued for Agent3: {}", data))
    } else {
        Err(ServerFnError::new("Agent3 is not available"))
    }
}

/// Process data through Agent 4
#[post("/api/agents/4/process")]
pub async fn process_agent4(data: String) -> Result<String, ServerFnError> {
    ensure_agents_initialized().await
        .map_err(|e| ServerFnError::new(format!("Failed to initialize agents: {}", e)))?;
    
    if let Some(actor_ref) = get_agent(4) {
        use crate::agents::AgentMessage;
        actor_ref.send_message(AgentMessage::ProcessData {
            data: data.clone(),
        });
        Ok(format!("Message queued for Agent4: {}", data))
    } else {
        Err(ServerFnError::new("Agent4 is not available"))
    }
}

/// Process data through Agent 5
#[post("/api/agents/5/process")]
pub async fn process_agent5(data: String) -> Result<String, ServerFnError> {
    ensure_agents_initialized().await
        .map_err(|e| ServerFnError::new(format!("Failed to initialize agents: {}", e)))?;
    
    if let Some(actor_ref) = get_agent(5) {
        use crate::agents::AgentMessage;
        actor_ref.send_message(AgentMessage::ProcessData {
            data: data.clone(),
        });
        Ok(format!("Message queued for Agent5: {}", data))
    } else {
        Err(ServerFnError::new("Agent5 is not available"))
    }
}

/// Get status of a specific agent
#[get("/api/agents/:id/status")]
pub async fn get_agent_status(id: u8) -> Result<String, ServerFnError> {
    ensure_agents_initialized().await
        .map_err(|e| ServerFnError::new(format!("Failed to initialize agents: {}", e)))?;
    
    if let Some(actor_ref) = get_agent(id) {
        use crate::agents::AgentMessage;
        actor_ref.send_message(AgentMessage::GetStatus);
        Ok(format!("Status request sent to Agent{}", id))
    } else {
        Err(ServerFnError::new(format!("Agent{} is not available", id)))
    }
}

/// Process data through any agent (dynamic routing)
#[post("/api/agents/:id/process")]
pub async fn process_agent_dynamic(id: u8, data: String) -> Result<String, ServerFnError> {
    ensure_agents_initialized().await
        .map_err(|e| ServerFnError::new(format!("Failed to initialize agents: {}", e)))?;
    
    if let Some(actor_ref) = get_agent(id) {
        use crate::agents::AgentMessage;
        actor_ref.send_message(AgentMessage::ProcessData {
            data: data.clone(),
        });
        Ok(format!("Message queued for Agent{}: {}", id, data))
    } else {
        Err(ServerFnError::new(format!("Agent{} is not available", id)))
    }
}

// ============================================================================
// MCP Server Functions - Desktop app triggers, results streamed to web clients
// ============================================================================

/// Call MCP example tool - Desktop app triggers, broadcasts to web clients via MCP channel
#[post("/api/mcp/example_tool")]
pub async fn mcp_example_tool() -> Result<String, ServerFnError> {
    eprintln!("[MCP] example_tool triggered from desktop app");
    let mcp_server = crate::mcp_server::PatternClockMCP::new();
    let result = mcp_server.call_example_tool().await;
    eprintln!("[MCP] example_tool result: {}", result);
    
    // Broadcast result through MCP channel to web clients
    let _ = get_mcp_broadcaster().send(result.clone());
    
    Ok(result)
}

/// Call MCP random number tool - Desktop app triggers, broadcasts to web clients via MCP channel
#[post("/api/mcp/random_number")]
pub async fn mcp_random_number() -> Result<String, ServerFnError> {
    eprintln!("[MCP] random_number triggered from desktop app");
    let mcp_server = crate::mcp_server::PatternClockMCP::new();
    let result = mcp_server.call_get_random_number().await;
    eprintln!("[MCP] random_number result: {}", result);
    
    // Broadcast result through MCP channel to web clients
    let _ = get_mcp_broadcaster().send(result.clone());
    
    Ok(result)
}

/// Call MCP process agent tool - Desktop app triggers, broadcasts to web clients via MCP channel
#[post("/api/mcp/process_agent")]
pub async fn mcp_process_agent(agent_id: u8, data: String) -> Result<String, ServerFnError> {
    eprintln!("[MCP] process_agent triggered from desktop app: agent_id={}, data={}", agent_id, data);
    let mcp_server = crate::mcp_server::PatternClockMCP::new();
    let result = mcp_server.call_process_agent(agent_id, data).await;
    eprintln!("[MCP] process_agent result: {}", result);
    
    // Broadcast result through MCP channel to web clients
    let _ = get_mcp_broadcaster().send(result.clone());
    
    Ok(result)
}

// ============================================================================
// MCP Stream Endpoint - Web clients subscribe to MCP results
// ============================================================================

/// Get next MCP result (long-polling endpoint for web clients)
/// This is the MCP communication channel - web app polls this endpoint
#[get("/api/mcp/receive")]
pub async fn mcp_receive() -> Result<String, ServerFnError> {
    eprintln!("[MCP] Web client requesting MCP result");
    
    let mut rx = get_mcp_broadcaster().subscribe();
    
    // Wait up to 60 seconds for a result
    match tokio::time::timeout(
        tokio::time::Duration::from_secs(60),
        rx.recv()
    ).await {
        Ok(Ok(result)) => {
            eprintln!("[MCP] Sending result to web client: {}", result);
            Ok(result)
        }
        Ok(Err(broadcast::error::RecvError::Closed)) => {
            eprintln!("[MCP] Broadcast channel closed");
            Ok(String::new())
        }
        Ok(Err(broadcast::error::RecvError::Lagged(skipped))) => {
            eprintln!("[MCP] Web client lagged, skipped {} messages", skipped);
            // Try to get the latest message
            match rx.try_recv() {
                Ok(result) => Ok(result),
                Err(_) => Ok(String::new()),
            }
        }
        Err(_) => {
            // Timeout - return empty string (normal for long-polling)
            Ok(String::new())
        }
    }
}

// Signal polling/queuing system removed - web app now calls MCP tools directly

/// Return 404 for removed signal endpoints (prevents cached browser requests)
#[get("/api/signals/receive")]
pub async fn receive_signal_removed() -> Result<String, ServerFnError> {
    Err(ServerFnError::new("Endpoint removed - use MCP endpoints instead"))
}

#[post("/api/signals/send")]
pub async fn send_signal_removed(_data: String) -> Result<String, ServerFnError> {
    Err(ServerFnError::new("Endpoint removed - use MCP endpoints instead"))
}
