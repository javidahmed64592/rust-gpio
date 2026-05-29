//! Runner - System Orchestrator
//!
//! Main executable that:
//! - Bootstraps the entire application
//! - Loads configuration
//! - Initializes communication channels
//! - Spawns all sensor tasks
//! - Spawns controller task
//! - Spawns all actuator tasks
//! - Wires everything together
//!
//! Contains minimal business logic - primarily dependency injection and orchestration.

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Runner starting - bootstrapping GPIO system...");

    // TODO: Load config from config/config.yaml
    // TODO: Create event channels (sensors -> controller)
    // TODO: Create command channels (controller -> actuators)
    // TODO: Spawn PIR sensor task
    // TODO: Spawn lighting override button task
    // TODO: Spawn brightness button task
    // TODO: Spawn MPU6050 sensor task
    // TODO: Spawn controller task
    // TODO: Spawn LED actuator task
    // TODO: Spawn LCD actuator task
    // TODO: Wait for all tasks or handle shutdown

    println!("Runner initialized (placeholder)");
    println!("All components would be spawned here");
    println!("Press Ctrl+C to shut down");

    // Keep running
    tokio::signal::ctrl_c().await?;
    println!("Runner shutting down - stopping all components...");

    Ok(())
}
