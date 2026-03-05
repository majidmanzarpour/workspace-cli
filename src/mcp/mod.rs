#[cfg(feature = "mcp")]
mod server;

#[cfg(feature = "mcp")]
pub use server::run;
