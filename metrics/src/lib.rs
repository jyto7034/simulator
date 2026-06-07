use lazy_static::lazy_static;
use prometheus::{opts, IntGauge, Registry};

lazy_static! {
    /// Active /game websocket connections.
    pub static ref ACTIVE_WS_CONNECTIONS: IntGauge =
        IntGauge::with_opts(opts!("active_ws_connections", "Number of active /game websocket connections")).unwrap();
}

/// Registers custom metrics used by the current /game server runtime.
pub fn register_custom_metrics(registry: &Registry) -> Result<(), prometheus::Error> {
    registry.register(Box::new(ACTIVE_WS_CONNECTIONS.clone()))?;

    Ok(())
}
