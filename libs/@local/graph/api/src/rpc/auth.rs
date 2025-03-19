use core::{fmt::Debug, marker::PhantomData};

use error_stack::{Report, ResultExt as _};
use harpc_client::{connection::Connection, utils::invoke_call_discrete};
use harpc_codec::{decode::ReportDecoder, encode::Encoder};
use harpc_server::{
    error::DelegationError,
    session::Session,
    utils::{delegate_call_discrete, parse_procedure_id},
};
use harpc_system::delegate::SubsystemDelegate;
use harpc_tower::{body::Body, request::Request, response::Response};
use harpc_types::response_kind::ResponseKind;
use type_system::provenance::ActorId;

use super::session::Account;

#[must_use]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, derive_more::Display, derive_more::Error)]
#[display("unable to authenticate user")]
pub struct AuthenticationError;

pub trait AuthenticationSystem {
    type ExecutionScope;

    async fn authenticate(
        &self,
        scope: Self::ExecutionScope,
        actor_id: ActorId,
    ) -> Result<(), Report<AuthenticationError>>;
}

// TODO: this can be auto generated by the `harpc` crate
pub mod meta {
    //! The `meta` module contains the metadata for the account service.
    //! In the future this will be automatically generated by the `harpc` crate.

    use frunk::HList;
    use harpc_system::{
        Subsystem,
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
    }

    pub struct ProcedureAuthenticate;

    impl Procedure for ProcedureAuthenticate {
        type Subsystem = AuthenticationSystem;

        const ID: <Self::Subsystem as Subsystem>::ProcedureId =
            AuthenticationProcedureId::Authenticate;
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct AuthenticationServer;

impl AuthenticationSystem for AuthenticationServer {
    type ExecutionScope = Session<Account>;

    async fn authenticate(
        &self,
        scope: Session<Account>,
        actor_id: ActorId,
    ) -> Result<(), Report<AuthenticationError>> {
        scope
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

impl<T> AuthenticationDelegate<T> {
    #[must_use]
    pub const fn new(inner: T) -> Self {
        Self { inner }
    }
}

impl<T, C> SubsystemDelegate<C> for AuthenticationDelegate<T>
where
    T: AuthenticationSystem<authenticate(..): Send, ExecutionScope: Send> + Send,
    C: Encoder + ReportDecoder + Clone + Send,
{
    type Error = Report<DelegationError>;
    type ExecutionScope = T::ExecutionScope;
    type Subsystem = meta::AuthenticationSystem;

    type Body<Source>
        = impl Body<Control: AsRef<ResponseKind>, Error = <C as Encoder>::Error>
    where
        Source: Body<Control = !, Error: Send + Sync> + Send;

    async fn call<B>(
        self,
        request: Request<B>,
        scope: T::ExecutionScope,
        codec: C,
    ) -> Result<Response<Self::Body<B>>, Self::Error>
    where
        B: Body<Control = !, Error: Send + Sync> + Send,
    {
        let id = parse_procedure_id(&request)?;

        match id {
            meta::AuthenticationProcedureId::Authenticate => {
                delegate_call_discrete(request, codec, |actor_id| async move {
                    self.inner.authenticate(scope, actor_id).await
                })
                .await
            }
        }
    }
}

// TODO: this can be auto generated by the `harpc` crate
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct AuthenticationClient<S, C> {
    _service: PhantomData<fn() -> *const S>,
    _codec: PhantomData<fn() -> *const C>,
}

impl<S, C> AuthenticationClient<S, C> {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            _service: PhantomData,
            _codec: PhantomData,
        }
    }
}

impl<S, C> Default for AuthenticationClient<S, C> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S, C> AuthenticationSystem for AuthenticationClient<S, C>
where
    S: harpc_client::connection::ConnectionService<C>,
    C: harpc_client::connection::ConnectionCodec,
{
    type ExecutionScope = Connection<S, C>;

    async fn authenticate(
        &self,
        scope: Connection<S, C>,
        actor_id: ActorId,
    ) -> Result<(), Report<AuthenticationError>> {
        invoke_call_discrete(
            scope,
            meta::AuthenticationProcedureId::Authenticate,
            [actor_id],
        )
        .await
        .change_context(AuthenticationError)
    }
}
