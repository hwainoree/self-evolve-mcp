use super::McpServer;
use rmcp::{tool, tool_router};

#[tool_router(router = marco_router, vis = "pub(super)")]
impl McpServer {
    #[tool(description = "Return polo. No arguments required.")]
    fn marco(&self) -> String {
        "polo".to_owned()
    }
}
