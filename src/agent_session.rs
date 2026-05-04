use crate::error::Error;
use nix::unistd::{Uid, User};
use std::{
    io::{Read, Write},
    os::unix::net::UnixStream,
    path::Path,
};

const POLKIT_AGENT_HELPER_SOCKET: &str = "/run/polkit/agent-helper.socket";

#[derive(Debug)]
pub struct PolkitAgengSession {
    pub user: User,
    stream: UnixStream,
    complete: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    Request { echo_on: bool, message: String },
    Error(String),
    Info(String),
    Complete(bool),
}

pub enum Response<'a> {
    Password(&'a str),
}

const PAM_PROMPT_ECHO_OFF: &str = "PAM_PROMPT_ECHO_OFF";
const PAM_PROMPT_ECHO_ON: &str = "PAM_PROMPT_ECHO_ON";
const PAM_ERROR_MSG: &str = "PAM_ERROR_MSG";
const PAM_TEXT_INFO: &str = "PAM_TEXT_INFO";
const SUCCESS: &str = "SUCCESS";
const FAILURE: &str = "FAILURE";

impl PolkitAgengSession {
    pub fn user_name(&self) -> &str {
        self.user.name.as_ref()
    }
    pub fn new<'a>(uid: impl Into<Uid>, cookie: impl Into<Option<&'a str>>) -> Result<Self, Error> {
        let uid = uid.into();
        let user = nix::unistd::User::from_uid(uid)?.ok_or(Error::UserNotFound(uid.as_raw()))?;

        let agent_path = Path::new(POLKIT_AGENT_HELPER_SOCKET);
        if !agent_path.exists() {
            return Err(Error::PolkitFileNotFound);
        }

        let mut stream = UnixStream::connect(agent_path)?;
        stream.write_all(user.name.as_bytes())?;
        stream.write_all(b"\n")?;

        if let Some(cookie) = cookie.into() {
            stream.write_all(cookie.as_bytes())?;
            stream.write_all(b"\n")?;
        }

        Ok(Self {
            user,
            stream,
            complete: false,
        })
    }

    pub fn is_complete(&self) -> bool {
        self.complete
    }

    pub fn dispatch(&mut self) -> Result<Message, Error> {
        let mut data = vec![];
        loop {
            let mut exact = [0; 1];
            self.stream.read_exact(&mut exact)?;

            if exact[0] == b'\n' {
                data.extend(exact);
                break;
            }
            data.extend(exact);
        }
        let response = String::from_utf8_lossy(&data);
        if response.starts_with(PAM_PROMPT_ECHO_OFF) {
            let message = response[PAM_PROMPT_ECHO_OFF.len()..]
                .trim_start()
                .to_string();
            return Ok(Message::Request {
                echo_on: false,
                message,
            });
        }
        if response.starts_with(PAM_PROMPT_ECHO_ON) {
            let message = response[PAM_PROMPT_ECHO_ON.len()..]
                .trim_start()
                .to_string();
            return Ok(Message::Request {
                echo_on: true,
                message,
            });
        }

        if response.starts_with(PAM_ERROR_MSG) {
            let message = response[PAM_ERROR_MSG.len()..].trim_start().to_string();
            return Ok(Message::Error(message));
        }
        if response.starts_with(PAM_TEXT_INFO) {
            let message = response[PAM_TEXT_INFO.len()..].trim_start().to_string();
            return Ok(Message::Info(message));
        }

        self.complete = true;
        if response.starts_with(SUCCESS) {
            return Ok(Message::Complete(true));
        }
        if response.starts_with(FAILURE) {
            return Ok(Message::Complete(false));
        }
        Err(Error::UnknownMessage(response.to_string()))
    }

    pub fn response<'a>(&mut self, response: Response<'a>) -> Result<(), Error> {
        match response {
            Response::Password(password) => {
                self.stream.write_all(password.as_bytes())?;
                self.stream.write_all(b"\n")?;
            }
        }
        Ok(())
    }
}
