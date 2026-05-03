mod flags;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, marker::PhantomData};
use zbus::{connection, zvariant::Type};

pub use flags::RegisterFlags;

pub trait PolkitCore: Sync + Send {
    type State;
    fn boot(&self) -> Self::State;
    fn authenticate(
        &mut self,
        state: &mut Self::State,
        action_id: &str,
        msg: &str,
        icon_name: &str,
        details: &HashMap<&str, &str>,
        identifies: &[Identity<'_>],
        cookie: &str,
    ) -> Result<(), PolkitError>;

    fn cancel_authentication(
        &mut self,
        state: &mut Self::State,
        cookie: &str,
    ) -> Result<(), PolkitError>;
}

pub struct PolkitAgentBuilder<C: PolkitCore> {
    agent: C,
}

pub struct PolkitAgent<C: PolkitCore<State = State>, State> {
    agent: C,
    state: State,
}

#[derive(Clone, Debug, zbus::DBusError)]
#[zbus(prefix = "org.freedesktop.PolicyKit1.Error")]
pub enum PolkitError {
    Failed,
    Cancelled,
    NotSupported,
    NotAuthorized,
    CancellationIdNotUnique,
}

#[derive(Debug, Serialize, Deserialize, Type)]
pub struct Identity<'a> {
    identity_kind: &'a str,
    identity_details: HashMap<&'a str, zbus::zvariant::Value<'a>>,
}

#[zbus::interface(name = "org.freedesktop.PolicyKit1.AuthenticationAgent")]
impl<C: PolkitCore<State = State> + 'static, State> PolkitAgent<C, State>
where
    State: 'static + Sync + Send,
{
    fn begin_authentitation(
        &mut self,
        action_id: &str,
        msg: &str,
        icon_name: &str,
        details: HashMap<&str, &str>,
        cookie: &str,
        identifies: Vec<Identity<'_>>,
    ) -> Result<(), PolkitError> {
        self.agent.authenticate(
            &mut self.state,
            action_id,
            msg,
            icon_name,
            &details,
            &identifies,
            cookie,
        )
    }
    fn cancel_authentication(&mut self, cookie: &str) -> Result<(), PolkitError> {
        self.agent.cancel_authentication(&mut self.state, cookie)
    }
}

mod seal {
    use super::*;
    use std::future::Future;
    pub trait Authenticate<State> {
        fn authenticate(
            &self,
            state: &mut State,
            action_id: &str,
            msg: &str,
            icon_name: &str,
            details: &HashMap<&str, &str>,
            identifies: &[Identity<'_>],
            cookie: &str,
        ) -> Result<(), PolkitError>;
    }
    impl<F, State> Authenticate<State> for F
    where
        F: Fn(
            &mut State,
            &str,
            &str,
            &str,
            &HashMap<&str, &str>,
            &[Identity<'_>],
            &str,
        ) -> Result<(), PolkitError>,
    {
        fn authenticate(
            &self,
            state: &mut State,
            action_id: &str,
            msg: &str,
            icon_name: &str,
            details: &HashMap<&str, &str>,
            identifies: &[Identity<'_>],
            cookie: &str,
        ) -> Result<(), PolkitError> {
            self(
                state, action_id, msg, icon_name, details, identifies, cookie,
            )
        }
    }
    pub trait CancelAuthentication<State> {
        fn cancel_authentication(&self, state: &mut State, cookie: &str)
        -> Result<(), PolkitError>;
    }

    impl<F, State> CancelAuthentication<State> for F
    where
        F: Fn(&mut State, &str) -> Result<(), PolkitError>,
    {
        fn cancel_authentication(
            &self,
            state: &mut State,
            cookie: &str,
        ) -> Result<(), PolkitError> {
            self(state, cookie)
        }
    }

    pub trait Boot<State> {
        fn boot(&self) -> State;
    }
    impl<F, State> Boot<State> for F
    where
        F: Fn() -> State,
    {
        fn boot(&self) -> State {
            self()
        }
    }
}

use seal::*;

pub fn polkit_agent_instance<Authenticate, CancelAuthentication, State, Boot>(
    boot: Boot,
    authenticate: Authenticate,
    cancel_authentication: CancelAuthentication,
) -> PolkitAgentBuilder<impl PolkitCore<State = State>>
where
    Boot: self::Boot<State> + Send + Sync,
    Authenticate: self::Authenticate<State> + Send + Sync,
    CancelAuthentication: self::CancelAuthentication<State> + Send + Sync,
    State: 'static + Send + Sync,
{
    struct Instance<State, Boot, Authenticate, CancelAuthentication> {
        boot: Boot,
        authenticate: Authenticate,
        cancel_authentication: CancelAuthentication,
        _state: PhantomData<State>,
    }
    impl<State, Boot, Authenticate, CancelAuthentication> PolkitCore
        for Instance<State, Boot, Authenticate, CancelAuthentication>
    where
        Boot: self::Boot<State> + Sync + Send,
        Authenticate: self::Authenticate<State> + Sync + Send,
        CancelAuthentication: self::CancelAuthentication<State> + Send + Sync,
        State: 'static + Send + Sync,
    {
        type State = State;
        fn boot(&self) -> Self::State {
            self.boot.boot()
        }
        fn authenticate(
            &mut self,
            state: &mut State,
            action_id: &str,
            msg: &str,
            icon_name: &str,
            details: &HashMap<&str, &str>,
            identifies: &[Identity<'_>],
            cookie: &str,
        ) -> Result<(), PolkitError> {
            self.authenticate.authenticate(
                state, action_id, msg, icon_name, details, identifies, cookie,
            )
        }

        fn cancel_authentication(
            &mut self,
            state: &mut State,
            cookie: &str,
        ) -> Result<(), PolkitError> {
            self.cancel_authentication
                .cancel_authentication(state, cookie)
        }
    }
    PolkitAgentBuilder {
        agent: Instance {
            boot,
            authenticate,
            cancel_authentication,
            _state: PhantomData,
        },
    }
}

impl<State, C: PolkitCore<State = State> + 'static> PolkitAgentBuilder<C>
where
    State: 'static + Send + Sync,
{
    pub async fn connect(self) -> zbus::Result<zbus::Connection> {
        let agent = PolkitAgent {
            state: self.agent.boot(),
            agent: self.agent,
        };
        connection::Builder::system()?
            .serve_at("/org/freedesktop/PolicyKit1/AuthenticationAgent", agent)?
            .build()
            .await
    }
}
