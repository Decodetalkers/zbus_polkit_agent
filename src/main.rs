use std::collections::HashMap;

use zbus_polkit_agent::{Identity, PolkitError, RegisterFlags, polkit_agent_instance};
struct Agent;

fn authenticate(
    agent: &mut Agent,
    action_id: &str,
    msg: &str,
    icon_name: &str,
    details: &HashMap<&str, &str>,
    identifiers: &[Identity<'_>],
    cookie: &str,
) -> Result<(), PolkitError> {
    Ok(())
}

fn cancel_authentication(agent: &mut Agent, cookie: &str) -> Result<(), PolkitError> {
    Ok(())
}

#[tokio::main]
async fn main() {
    let connection = polkit_agent_instance(|| Agent, authenticate, cancel_authentication)
        .connect()
        .await
        .unwrap();
    println!("{:?}", connection.unique_name());
    std::future::pending::<()>().await;
}
