use core::fmt::Debug;

use error_stack::{Report, ResultExt as _};
use graph_types::account::AccountId;
use harpc_client::{connection::Connection, utils::invoke_call_discrete};
use harpc_codec::{decode::ReportDecoder, encode::Encoder};
use harpc_server::{
    error::DelegationError,
    session::Session,
    utils::{delegate_call_discrete, parse_procedure_id},
};
use harpc_service::delegate::SubsystemDelegate;
use harpc_tower::{body::Body, request::Request, response::Response};
use harpc_types::response_kind::ResponseKind;

use super::{role, session::Account};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, derive_more::Display, derive_more::Error)]
#[display("unable to authenticate user")]
pub struct AuthenticationError;

pub trait AuthenticationSystem<R>
where
    R: role::Role,
{
    async fn authenticate(
        &self,
        session: R::Session,
        actor_id: AccountId,
    ) -> Result<(), Report<AuthenticationError>>;
}

// TODO: this can be auto generated by the `harpc` crate
pub mod meta {
    //! The `meta` module contains the metadata for the account service.
    //! In the future this will be automatically generated by the `harpc` crate.

    use frunk::HList;
    use harpc_service::{
        Subsystem,
        metadata::Metadata,
        procedure::{Procedure, ProcedureIdentifier},
    };
    use harpc_types::{procedure::ProcedureId, version::Version};

    use crate::rpc::GraphSubsystemId;

    pub enum AuthenticationProcedureId {
        Authenticate,
    }

    impl ProcedureIdentifier for AuthenticationProcedureId {
        type Subsystem = AuthenticationSystem;

        fn from_id(id: ProcedureId) -> Option<Self> {
            match id.value() {
                0x00 => Some(Self::Authenticate),
                _ => None,
            }
        }

        fn into_id(self) -> ProcedureId {
            match self {
                Self::Authenticate => ProcedureId::new(0x00),
            }
        }
    }

    pub struct AuthenticationSystem;

    impl Subsystem for AuthenticationSystem {
        type ProcedureId = AuthenticationProcedureId;
        type Procedures = HList![ProcedureAuthenticate];
        type SubsystemId = GraphSubsystemId;

        const ID: GraphSubsystemId = GraphSubsystemId::Authentication;
        const VERSION: Version = Version {
            major: 0x00,
            minor: 0x00,
        };

        fn metadata() -> Metadata {
            Metadata {
                since: Version {
                    major: 0x00,
                    minor: 0x00,
                },
                deprecation: None,
            }
        }
    }

    pub struct ProcedureAuthenticate;

    impl Procedure for ProcedureAuthenticate {
        type Subsystem = AuthenticationSystem;

        const ID: <Self::Subsystem as Subsystem>::ProcedureId =
            AuthenticationProcedureId::Authenticate;

        fn metadata() -> Metadata {
            Metadata {
                since: Version {
                    major: 0x00,
                    minor: 0x00,
                },
                deprecation: None,
            }
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct AuthenticationServer;

impl AuthenticationSystem<role::Server> for AuthenticationServer {
    async fn authenticate(
        &self,
        session: Session<Account>,
        actor_id: AccountId,
    ) -> Result<(), Report<AuthenticationError>> {
        session
            .update(Account {
                actor_id: Some(actor_id),
            })
            .await;

        Ok(())
    }
}

// TODO: this can be auto generated by the `harpc` crate
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct AuthenticationDelegate<T> {
    inner: T,
}

impl<T, C> SubsystemDelegate<Session<Account>, C> for AuthenticationDelegate<T>
where
    T: AuthenticationSystem<role::Server, authenticate(..): Send> + Send,
    C: Encoder + ReportDecoder + Clone + Send,
{
    type Error = Report<DelegationError>;
    type Subsystem = meta::AuthenticationSystem;

    type Body<Source>
        = impl Body<Control: AsRef<ResponseKind>, Error = <C as Encoder>::Error>
    where
        Source: Body<Control = !, Error: Send + Sync> + Send + Sync;

    async fn call<B>(
        self,
        request: Request<B>,
        session: Session<Account>,
        codec: C,
    ) -> Result<Response<Self::Body<B>>, Self::Error>
    where
        B: Body<Control = !, Error: Send + Sync> + Send + Sync,
    {
        let id = parse_procedure_id(&request)?;

        match id {
            meta::AuthenticationProcedureId::Authenticate => {
                delegate_call_discrete(request, codec, |actor_id| async move {
                    self.inner.authenticate(session, actor_id).await
                })
                .await
            }
        }
    }
}

// TODO: this can be auto generated by the `harpc` crate
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct AuthenticationClient;

impl<Svc, C> AuthenticationSystem<role::Client<Svc, C>> for AuthenticationClient
where
    Svc: harpc_client::connection::ConnectionService<C>,
    C: harpc_client::connection::ConnectionCodec,
{
    async fn authenticate(
        &self,
        session: Connection<Svc, C>,
        actor_id: AccountId,
    ) -> Result<(), Report<AuthenticationError>> {
        invoke_call_discrete(session, meta::AuthenticationProcedureId::Authenticate, [
            actor_id,
        ])
        .await
        .change_context(AuthenticationError)
    }
}
