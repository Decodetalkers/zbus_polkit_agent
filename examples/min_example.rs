use std::collections::HashMap;

use rpassword::prompt_password;
use zbus_polkit_agent::{
    Identity, PolkitError, UnixUser,
    agent_session::{Message, PolkitAgengSession, Response},
    polkit_agent_instance,
};
struct Agent;

fn authenticate(
    _agent: &mut Agent,
    _action_id: &str,
    _msg: &str,
    _icon_name: &str,
    _details: HashMap<&str, &str>,
    cookie: &str,
    mut identifiers: Vec<Identity<'_>>,
) -> Result<(), PolkitError> {
    let identify: UnixUser = identifiers.remove(0).try_into().unwrap();
    let mut session = PolkitAgengSession::new(identify, cookie).unwrap();
    while !session.is_complete() {
        let message = session.dispatch().unwrap();
        if let Message::Request { message, .. } = message {
            if message.starts_with("Password:") {
                let Ok(password) = prompt_password(format!("{} password: ", session.user_name()))
                else {
                    return Err(PolkitError::Failed);
                };
                session.response(Response::Password(&password)).unwrap();
            }
        }
    }
    Ok(())
}

fn cancel_authentication(_agent: &mut Agent, _cookie: &str) -> Result<(), PolkitError> {
    Ok(())
}
const OBJECT_PATH: &str = "/org/waycrate/PolicyKit1/AuthenticationAgent";

#[tokio::main]
async fn main() {
    let _connection = polkit_agent_instance(|| Agent, authenticate, cancel_authentication)
        .connect(OBJECT_PATH)
        .await
        .unwrap();
    std::future::pending::<()>().await;
}
