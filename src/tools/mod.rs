pub mod marco;

use rmcp::{
    ServerHandler,
    handler::server::router::tool::ToolRouter,
    model::{Implementation, ServerCapabilities, ServerConfig},
    tool_handler,
};

#[derive(Clone)]
pub struct McpServer {
    tool_router: ToolRouter<Self>,
}

impl Default for McpServer {
    fn default() -> Self {
        Self {
            tool_router: Self::marco_router(),
        }
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for McpServer {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build()).with_server_info(
            Implementation::new("self-evolve-mcp", env!("CARGO_PKG_VERSION")),
        )
    }
}
