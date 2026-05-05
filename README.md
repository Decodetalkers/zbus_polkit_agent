# zbus-polkit-agent

You can use this library to build you polkit agent in rust

## min example

```rust
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
    let identify: UnixUser = identifiers.remove(0).try_into()?;
    let mut session = PolkitAgengSession::new(identify, cookie)?;
    let mut retry_count = 3;
    while retry_count >= 0 {
        while !session.is_complete() {
            let message = session.dispatch()?;
            if let Message::Request { prompt, .. } = message {
                let Ok(password) = prompt_password(format!("{} {prompt} ", session.user_name()))
                else {
                    return Err(PolkitError::Cancelled);
                };
                session
                    .response(Response {
                        password: &password,
                    })?;
            }
        }

        if session.succeeded() {
            return Ok(());
        }
        session.restart()?;
        retry_count -= 1;
    }
    if !session.succeeded() {
        return Err(PolkitError::Failed);
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
```

The interface design is inspired by iced, but then it makes it hard for me to use async, because until now, async is still hard to use in trait

Now you can use iced to build your own polkit agent

## TODO
* Async
* Documents
* Etc

## Info
I use GPL-v3 on this project, and from now, I will also prefer GPL-v3. Maybe it will still be used to train LLM, and maybe you do not think this project worth that much. But I want to own the copyright, at least I want keep the copyright to human, not robots.

I have wanted to create such project for a very long time. Before I was bad at reading glib coding, maybe I am just bad at reading C code. But I still learn a lot, find a lot of documents. In 2025, I forked polkit-rs, and implemented the polkit-agent in the way of ffi, but that relays a lot on glibc runtime. Then I started considering to make a pure zbus version. And these days I realized the problem that I cannot see the dbus interface of agent in d-spy, is caused by agent is runed in user's session, but registered with system service. Finally I understand what is happening. This was such a good thing happened to me recently.

Maybe nobody will care this project, but I finally finished it. This the present in 2026 for me
