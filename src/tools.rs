use anyhow::Result;
use async_trait::async_trait;
use rmcp::handler::server::ToolHandler;
use rmcp::model::{CallToolRequest, CallToolResult};
use serde_json::json;
use std::sync::Arc;
use tracing::{debug, error};

use crate::helix_client::HelixClient;

macro_rules! structured_ok { ($val:expr) => { CallToolResult::structured($val) }; }
macro_rules! structured_err { ($e:expr) => { CallToolResult::structured_error(json!({"error": $e.to_string()})) }; }

pub struct InitHandler { pub helix_client: Arc<HelixClient> }
#[async_trait] impl ToolHandler for InitHandler {
    async fn call_tool(&self, _request: CallToolRequest) -> Result<CallToolResult> {
        debug!("MCP init called");
        match self.helix_client.init().await {
            Ok(conn_id) => Ok(structured_ok!(json!({"connection_id": conn_id}))),
            Err(e) => { error!("MCP init failed: {}", e); Ok(structured_err!(e)) }
        }
    }
}

pub struct NextHandler { pub helix_client: Arc<HelixClient> }
#[async_trait] impl ToolHandler for NextHandler {
    async fn call_tool(&self, request: CallToolRequest) -> Result<CallToolResult> {
        let conn = request.params.arguments.as_ref().and_then(|m| m.get("connection_id")).and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("Missing connection_id"))?;
        match self.helix_client.next(conn).await {
            Ok(res) => Ok(structured_ok!(res)),
            Err(e) => { error!("MCP next failed: {}", e); Ok(structured_err!(e)) }
        }
    }
}

pub struct CollectHandler { pub helix_client: Arc<HelixClient> }
#[async_trait] impl ToolHandler for CollectHandler {
    async fn call_tool(&self, request: CallToolRequest) -> Result<CallToolResult> {
        let conn = request.params.arguments.as_ref().and_then(|m| m.get("connection_id")).and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("Missing connection_id"))?;
        let range = request.params.arguments.as_ref().and_then(|m| m.get("range")).and_then(|r| r.as_object()).and_then(|o| { let start = o.get("start")?.as_u64()? as usize; let end = o.get("end")?.as_u64()? as usize; Some((start, end)) });
        let drop_flag = request.params.arguments.as_ref().and_then(|m| m.get("drop")).and_then(|v| v.as_bool()).unwrap_or(false);
        match self.helix_client.collect(conn, range, drop_flag).await {
            Ok(res) => Ok(structured_ok!(res)),
            Err(e) => { error!("MCP collect failed: {}", e); Ok(structured_err!(e)) }
        }
    }
}

pub struct ResetHandler { pub helix_client: Arc<HelixClient> }
#[async_trait] impl ToolHandler for ResetHandler {
    async fn call_tool(&self, request: CallToolRequest) -> Result<CallToolResult> {
        let conn = request.params.arguments.as_ref().and_then(|m| m.get("connection_id")).and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("Missing connection_id"))?;
        match self.helix_client.reset(conn).await {
            Ok(msg) => Ok(structured_ok!(json!({"message": msg}))),
            Err(e) => { error!("MCP reset failed: {}", e); Ok(structured_err!(e)) }
        }
    }
}

pub struct SchemaResourceHandler { pub helix_client: Arc<HelixClient> }
#[async_trait] impl ToolHandler for SchemaResourceHandler {
    async fn call_tool(&self, request: CallToolRequest) -> Result<CallToolResult> {
        let conn = request.params.arguments.as_ref().and_then(|m| m.get("connection_id")).and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("Missing connection_id"))?;
        match self.helix_client.schema_resource(conn).await {
            Ok(res) => Ok(structured_ok!(res)),
            Err(e) => { error!("MCP schema_resource failed: {}", e); Ok(structured_err!(e)) }
        }
    }
}

pub struct NFromTypeHandler { pub helix_client: Arc<HelixClient> }
#[async_trait] impl ToolHandler for NFromTypeHandler {
    async fn call_tool(&self, request: CallToolRequest) -> Result<CallToolResult> {
        let conn = request.params.arguments.as_ref().and_then(|m| m.get("connection_id")).and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("Missing connection_id"))?;
        let node_type = request.params.arguments.as_ref().and_then(|m| m.get("node_type")).and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("Missing node_type"))?;
        match self.helix_client.n_from_type(conn, node_type).await {
            Ok(res) => Ok(structured_ok!(res)),
            Err(e) => { error!("MCP n_from_type failed: {}", e); Ok(structured_err!(e)) }
        }
    }
}

pub struct EFromTypeHandler { pub helix_client: Arc<HelixClient> }
#[async_trait] impl ToolHandler for EFromTypeHandler {
    async fn call_tool(&self, request: CallToolRequest) -> Result<CallToolResult> {
        let conn = request.params.arguments.as_ref().and_then(|m| m.get("connection_id")).and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("Missing connection_id"))?;
        let edge_type = request.params.arguments.as_ref().and_then(|m| m.get("edge_type")).and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("Missing edge_type"))?;
        match self.helix_client.e_from_type(conn, edge_type).await {
            Ok(res) => Ok(structured_ok!(res)),
            Err(e) => { error!("MCP e_from_type failed: {}", e); Ok(structured_err!(e)) }
        }
    }
}

pub struct OutStepHandler { pub helix_client: Arc<HelixClient> }
#[async_trait] impl ToolHandler for OutStepHandler {
    async fn call_tool(&self, request: CallToolRequest) -> Result<CallToolResult> {
        let conn = request.params.arguments.as_ref().and_then(|m| m.get("connection_id")).and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("Missing connection_id"))?;
        let edge_label = request.params.arguments.as_ref().and_then(|m| m.get("edge_label")).and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("Missing edge_label"))?;
        let edge_type = request.params.arguments.as_ref().and_then(|m| m.get("edge_type")).and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("Missing edge_type"))?;
        match self.helix_client.out_step(conn, edge_label, edge_type).await {
            Ok(res) => Ok(structured_ok!(res)),
            Err(e) => { error!("MCP out_step failed: {}", e); Ok(structured_err!(e)) }
        }
    }
}

pub struct OutEStepHandler { pub helix_client: Arc<HelixClient> }
#[async_trait] impl ToolHandler for OutEStepHandler {
    async fn call_tool(&self, request: CallToolRequest) -> Result<CallToolResult> {
        let conn = request.params.arguments.as_ref().and_then(|m| m.get("connection_id")).and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("Missing connection_id"))?;
        let edge_label = request.params.arguments.as_ref().and_then(|m| m.get("edge_label")).and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("Missing edge_label"))?;
        match self.helix_client.out_e_step(conn, edge_label).await {
            Ok(res) => Ok(structured_ok!(res)),
            Err(e) => { error!("MCP out_e_step failed: {}", e); Ok(structured_err!(e)) }
        }
    }
}

pub struct InStepHandler { pub helix_client: Arc<HelixClient> }
#[async_trait] impl ToolHandler for InStepHandler {
    async fn call_tool(&self, request: CallToolRequest) -> Result<CallToolResult> {
        let conn = request.params.arguments.as_ref().and_then(|m| m.get("connection_id")).and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("Missing connection_id"))?;
        let edge_label = request.params.arguments.as_ref().and_then(|m| m.get("edge_label")).and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("Missing edge_label"))?;
        let edge_type = request.params.arguments.as_ref().and_then(|m| m.get("edge_type")).and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("Missing edge_type"))?;
        match self.helix_client.in_step(conn, edge_label, edge_type).await {
            Ok(res) => Ok(structured_ok!(res)),
            Err(e) => { error!("MCP in_step failed: {}", e); Ok(structured_err!(e)) }
        }
    }
}

pub struct InEStepHandler { pub helix_client: Arc<HelixClient> }
#[async_trait] impl ToolHandler for InEStepHandler {
    async fn call_tool(&self, request: CallToolRequest) -> Result<CallToolResult> {
        let conn = request.params.arguments.as_ref().and_then(|m| m.get("connection_id")).and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("Missing connection_id"))?;
        let edge_label = request.params.arguments.as_ref().and_then(|m| m.get("edge_label")).and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("Missing edge_label"))?;
        match self.helix_client.in_e_step(conn, edge_label).await {
            Ok(res) => Ok(structured_ok!(res)),
            Err(e) => { error!("MCP in_e_step failed: {}", e); Ok(structured_err!(e)) }
        }
    }
}

pub struct FilterItemsHandler { pub helix_client: Arc<HelixClient> }
#[async_trait] impl ToolHandler for FilterItemsHandler {
    async fn call_tool(&self, request: CallToolRequest) -> Result<CallToolResult> {
        let conn = request.params.arguments.as_ref().and_then(|m| m.get("connection_id")).and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("Missing connection_id"))?;
        let filter = request.params.arguments.as_ref().and_then(|m| m.get("filter")).cloned().ok_or_else(|| anyhow::anyhow!("Missing filter"))?;
        match self.helix_client.filter_items(conn, filter).await {
            Ok(res) => Ok(structured_ok!(res)),
            Err(e) => { error!("MCP filter_items failed: {}", e); Ok(structured_err!(e)) }
        }
    }
}

pub struct SearchVectorTextHandler { pub helix_client: Arc<HelixClient> }
#[async_trait] impl ToolHandler for SearchVectorTextHandler {
    async fn call_tool(&self, request: CallToolRequest) -> Result<CallToolResult> {
        let conn = request.params.arguments.as_ref().and_then(|m| m.get("connection_id")).and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("Missing connection_id"))?;
        let query = request.params.arguments.as_ref().and_then(|m| m.get("query")).and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("Missing query"))?;
        let label = request.params.arguments.as_ref().and_then(|m| m.get("label")).and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("Missing label"))?;
        match self.helix_client.search_vector_text(conn, query, label).await {
            Ok(res) => Ok(structured_ok!(res)),
            Err(e) => { error!("MCP search_vector_text failed: {}", e); Ok(structured_err!(e)) }
        }
    }
}

pub struct SearchKeywordHandler { pub helix_client: Arc<HelixClient> }
#[async_trait] impl ToolHandler for SearchKeywordHandler {
    async fn call_tool(&self, request: CallToolRequest) -> Result<CallToolResult> {
        let conn = request.params.arguments.as_ref().and_then(|m| m.get("connection_id")).and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("Missing connection_id"))?;
        let query = request.params.arguments.as_ref().and_then(|m| m.get("query")).and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("Missing query"))?;
        let label = request.params.arguments.as_ref().and_then(|m| m.get("label")).and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("Missing label"))?;
        let limit = request.params.arguments.as_ref().and_then(|m| m.get("limit")).and_then(|v| v.as_u64()).unwrap_or(10) as usize;
        match self.helix_client.search_keyword(conn, query, label, limit).await {
            Ok(res) => Ok(structured_ok!(res)),
            Err(e) => { error!("MCP search_keyword failed: {}", e); Ok(structured_err!(e)) }
        }
    }
}
