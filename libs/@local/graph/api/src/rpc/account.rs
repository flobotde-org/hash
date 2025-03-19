use alloc::{borrow::Cow, sync::Arc};
use core::{
    error::{self, Error},
    marker::PhantomData,
};

use error_stack::{Report, ResultExt as _};
use harpc_client::{connection::Connection, utils::invoke_call_discrete};
use harpc_codec::{decode::ReportDecoder, encode::Encoder};
use harpc_server::{
    error::{DelegationError, Forbidden},
    session::Session,
    utils::{delegate_call_discrete, parse_procedure_id},
};
use harpc_system::delegate::SubsystemDelegate;
use harpc_tower::{body::Body, either::Either, request::Request, response::Response};
use harpc_types::{error_code::ErrorCode, response_kind::ResponseKind};
use hash_graph_authorization::{
    AuthorizationApi as _, AuthorizationApiPool,
    backend::ModifyRelationshipOperation,
    schema::{
        AccountGroupMemberSubject, AccountGroupPermission, AccountGroupRelationAndSubject,
        WebOwnerSubject,
    },
    zanzibar::Consistency,
};
use hash_graph_store::{
    account::{AccountStore as _, InsertAccountGroupIdParams, InsertAccountIdParams},
    pool::StorePool,
};
use hash_temporal_client::TemporalClient;
use type_system::{
    provenance::ActorId,
    web::{ActorGroupId, OwnedById},
};

use super::session::Account;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct PermissionResponse {
    pub has_permission: bool,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, derive_more::Display)]
#[display("account {id} does not exist in the graph")]
pub struct AccountNotFoundError {
    id: ActorId,
}

impl Error for AccountNotFoundError {
    fn provide<'a>(&'a self, request: &mut error::Request<'a>) {
        request.provide_value(ErrorCode::RESOURCE_NOT_FOUND);
    }
}

#[must_use]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, derive_more::Display, derive_more::Error)]
#[display("unable to fullfil account request")]
pub struct AccountError;

pub trait AccountSystem {
    type ExecutionScope;

    async fn create_account(
        &self,
        scope: Self::ExecutionScope,
        params: InsertAccountIdParams,
    ) -> Result<ActorId, Report<AccountError>>;

    async fn create_account_group(
        &self,
        scope: Self::ExecutionScope,
        params: InsertAccountGroupIdParams,
    ) -> Result<ActorGroupId, Report<AccountError>>;

    async fn check_account_group_permission(
        &self,
        scope: Self::ExecutionScope,
        account_group_id: ActorGroupId,
        permission: AccountGroupPermission,
    ) -> Result<PermissionResponse, Report<AccountError>>;

    async fn add_account_group_member(
        &self,
        scope: Self::ExecutionScope,
        account_group_id: ActorGroupId,
        account_id: ActorId,
    ) -> Result<(), Report<AccountError>>;

    async fn remove_account_group_member(
        &self,
        scope: Self::ExecutionScope,
        account_group_id: ActorGroupId,
        account_id: ActorId,
    ) -> Result<(), Report<AccountError>>;
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

    pub enum AccountProcedureId {
        CreateAccount,
        CreateAccountGroup,
        CheckAccountGroupPermission,
        AddAccountGroupMember,
        RemoveAccountGroupMember,
    }

    impl ProcedureIdentifier for AccountProcedureId {
        type Subsystem = AccountSystem;

        fn from_id(id: ProcedureId) -> Option<Self> {
            match id.value() {
                0x00 => Some(Self::CreateAccount),
                0x01 => Some(Self::CreateAccountGroup),
                0x02 => Some(Self::CheckAccountGroupPermission),
                0x03 => Some(Self::AddAccountGroupMember),
                0x04 => Some(Self::RemoveAccountGroupMember),
                _ => None,
            }
        }

        fn into_id(self) -> ProcedureId {
            match self {
                Self::CreateAccount => ProcedureId::new(0x00),
                Self::CreateAccountGroup => ProcedureId::new(0x01),
                Self::CheckAccountGroupPermission => ProcedureId::new(0x02),
                Self::AddAccountGroupMember => ProcedureId::new(0x03),
                Self::RemoveAccountGroupMember => ProcedureId::new(0x04),
            }
        }
    }

    pub struct AccountSystem;

    impl Subsystem for AccountSystem {
        type ProcedureId = AccountProcedureId;
        type Procedures = HList![
            ProcedureCreateAccount,
            ProcedureCreateAccountGroup,
            ProcedureCheckAccountGroupPermission,
            ProcedureAddAccountGroupMember,
            ProcedureRemoveAccountGroupMember
        ];
        type SubsystemId = GraphSubsystemId;

        const ID: GraphSubsystemId = GraphSubsystemId::Account;
        const VERSION: Version = Version {
            major: 0x00,
            minor: 0x00,
        };
    }

    pub struct ProcedureCreateAccount;

    impl Procedure for ProcedureCreateAccount {
        type Subsystem = AccountSystem;

        const ID: <Self::Subsystem as Subsystem>::ProcedureId = AccountProcedureId::CreateAccount;
    }

    pub struct ProcedureCreateAccountGroup;

    impl Procedure for ProcedureCreateAccountGroup {
        type Subsystem = AccountSystem;

        const ID: <Self::Subsystem as Subsystem>::ProcedureId =
            AccountProcedureId::CreateAccountGroup;
    }

    pub struct ProcedureCheckAccountGroupPermission;

    impl Procedure for ProcedureCheckAccountGroupPermission {
        type Subsystem = AccountSystem;

        const ID: <Self::Subsystem as Subsystem>::ProcedureId =
            AccountProcedureId::CheckAccountGroupPermission;
    }

    pub struct ProcedureAddAccountGroupMember;

    impl Procedure for ProcedureAddAccountGroupMember {
        type Subsystem = AccountSystem;

        const ID: <Self::Subsystem as Subsystem>::ProcedureId =
            AccountProcedureId::AddAccountGroupMember;
    }

    pub struct ProcedureRemoveAccountGroupMember;

    impl Procedure for ProcedureRemoveAccountGroupMember {
        type Subsystem = AccountSystem;

        const ID: <Self::Subsystem as Subsystem>::ProcedureId =
            AccountProcedureId::RemoveAccountGroupMember;
    }
}

#[derive(Debug)]
#[derive_where::derive_where(Clone)]
pub struct AccountServer<S, A> {
    pub authorization_api_pool: Arc<A>,
    pub temporal_client: Option<Arc<TemporalClient>>,
    pub store_pool: Arc<S>,
}

impl<S, A> AccountServer<S, A>
where
    S: StorePool + Send + Sync,
    A: AuthorizationApiPool + Send + Sync,
{
    async fn authorization_api(&self) -> Result<A::Api<'_>, Report<AccountError>> {
        self.authorization_api_pool
            .acquire()
            .await
            .inspect_err(|error| {
                tracing::error!(?error, "Could not acquire access to the authorization API");
            })
            .change_context(AccountError)
    }

    async fn store(&self) -> Result<S::Store<'_, A::Api<'_>>, Report<AccountError>> {
        let authorization_api = self.authorization_api().await?;

        self.store_pool
            .acquire(authorization_api, self.temporal_client.clone())
            .await
            .inspect_err(|report| {
                tracing::error!(error=?report, "Could not acquire store");
            })
            .change_context(AccountError)
    }

    fn actor(session: &Session<Account>) -> Result<ActorId, Report<AccountError>> {
        let &Account {
            actor_id: Some(actor_id),
        } = session.get()
        else {
            let request_info = session.request_info();

            return Err(Report::new(Forbidden {
                subsystem: request_info.subsystem,
                procedure: request_info.procedure,
                reason: Cow::Borrowed("user authentication required"),
            })
            .change_context(AccountError));
        };

        Ok(actor_id)
    }
}

impl<S, A> AccountSystem for AccountServer<S, A>
where
    S: StorePool + Send + Sync,
    A: AuthorizationApiPool + Send + Sync,
{
    type ExecutionScope = Session<Account>;

    async fn create_account(
        &self,
        scope: Session<Account>,
        params: InsertAccountIdParams,
    ) -> Result<ActorId, Report<AccountError>> {
        let actor_id = Self::actor(&scope)?;

        let mut store = self.store().await?;

        let account_id = params.account_id;
        store
            .insert_account_id(actor_id, params)
            .await
            .change_context(AccountError)?;

        Ok(account_id)
    }

    async fn create_account_group(
        &self,
        scope: Session<Account>,
        params: InsertAccountGroupIdParams,
    ) -> Result<ActorGroupId, Report<AccountError>> {
        let actor_id = Self::actor(&scope)?;

        let mut store = self.store().await?;

        let account = store
            .identify_owned_by_id(OwnedById::from(actor_id))
            .await
            .inspect_err(|report| {
                tracing::error!(error=?report, "Could not identify account");
            })
            .change_context(AccountError)?;

        if account != (WebOwnerSubject::Account { id: actor_id }) {
            tracing::error!("Account does not exist in the graph");
            return Err(
                Report::new(AccountNotFoundError { id: actor_id }).change_context(AccountError)
            );
        }

        let account_group_id = params.account_group_id;
        store
            .insert_account_group_id(actor_id, params)
            .await
            .inspect_err(|report| {
                tracing::error!(error=?report, "Could not create account id");
            })
            .change_context(AccountError)?;

        Ok(account_group_id)
    }

    async fn check_account_group_permission(
        &self,
        scope: Session<Account>,
        account_group_id: ActorGroupId,
        permission: AccountGroupPermission,
    ) -> Result<PermissionResponse, Report<AccountError>> {
        let actor_id = Self::actor(&scope)?;

        let auth = self.authorization_api().await?;

        let check = auth
            .check_account_group_permission(
                actor_id,
                permission,
                account_group_id,
                Consistency::FullyConsistent,
            )
            .await
            .inspect_err(|error| {
                tracing::error!(
                    ?error,
                    "Could not check if permission on the account group is granted to the \
                     specified actor"
                );
            })
            .change_context(AccountError)?;

        Ok(PermissionResponse {
            has_permission: check.has_permission,
        })
    }

    async fn add_account_group_member(
        &self,
        scope: Session<Account>,
        account_group_id: ActorGroupId,
        account_id: ActorId,
    ) -> Result<(), Report<AccountError>> {
        let actor_id = Self::actor(&scope)?;

        let mut auth = self.authorization_api().await?;

        let check = auth
            .check_account_group_permission(
                actor_id,
                AccountGroupPermission::AddMember,
                account_group_id,
                Consistency::FullyConsistent,
            )
            .await
            .inspect_err(|error| {
                tracing::error!(
                    ?error,
                    "Could not check if account group member can be added"
                );
            })
            .change_context(AccountError)?;

        if !check.has_permission {
            return Err(Report::new(Forbidden {
                subsystem: scope.request_info().subsystem,
                procedure: scope.request_info().procedure,
                reason: Cow::Borrowed("actor does not have permission to add account group member"),
            })
            .change_context(AccountError));
        }

        auth.modify_account_group_relations([(
            ModifyRelationshipOperation::Create,
            account_group_id,
            AccountGroupRelationAndSubject::Member {
                subject: AccountGroupMemberSubject::Account { id: account_id },
                level: 0,
            },
        )])
        .await
        .inspect_err(|error| {
            tracing::error!(?error, "Could not add account group member");
        })
        .change_context(AccountError)?;

        Ok(())
    }

    async fn remove_account_group_member(
        &self,
        scope: Session<Account>,
        account_group_id: ActorGroupId,
        account_id: ActorId,
    ) -> Result<(), Report<AccountError>> {
        let actor_id = Self::actor(&scope)?;

        let mut auth = self.authorization_api().await?;

        let check = auth
            .check_account_group_permission(
                actor_id,
                AccountGroupPermission::RemoveMember,
                account_group_id,
                Consistency::FullyConsistent,
            )
            .await
            .inspect_err(|error| {
                tracing::error!(
                    ?error,
                    "Could not check if account group member can be removed"
                );
            })
            .change_context(AccountError)?;

        if !check.has_permission {
            let request_info = scope.request_info();

            return Err(Report::new(Forbidden {
                subsystem: request_info.subsystem,
                procedure: request_info.procedure,
                reason: Cow::Borrowed(
                    "actor does not have permission to remove account group member",
                ),
            })
            .change_context(AccountError));
        }

        auth.modify_account_group_relations([(
            ModifyRelationshipOperation::Delete,
            account_group_id,
            AccountGroupRelationAndSubject::Member {
                subject: AccountGroupMemberSubject::Account { id: account_id },
                level: 0,
            },
        )])
        .await
        .inspect_err(|error| {
            tracing::error!(?error, "Could not remove account group member");
        })
        .change_context(AccountError)?;

        Ok(())
    }
}

// TODO: this can be auto generated by the `harpc` crate
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct AccountDelegate<T> {
    inner: T,
}

impl<T> AccountDelegate<T> {
    #[must_use]
    pub const fn new(inner: T) -> Self {
        Self { inner }
    }
}

impl<T, C> SubsystemDelegate<C> for AccountDelegate<T>
where
    T: AccountSystem<
            create_account(..): Send,
            create_account_group(..): Send,
            check_account_group_permission(..): Send,
            add_account_group_member(..): Send,
            remove_account_group_member(..): Send,
            ExecutionScope: Send,
        > + Send,
    C: Encoder + ReportDecoder + Clone + Send,
{
    type Error = Report<DelegationError>;
    type ExecutionScope = T::ExecutionScope;
    type Subsystem = meta::AccountSystem;

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

        // The Either chain here isn't... great, but the only other way would be to box. To box we'd
        // need to require that the `Decoder<Output>` is both `Send` and `Sync`, which it can be,
        // but to completely write out the trait bound is a bit of a pain.
        // We would instead most likely need to add `+ Sync` to the GAT, which would over-constrain
        // it unnecessarily, but would _in theory_ allow us to remove the `Either` chain.
        match id {
            meta::AccountProcedureId::CreateAccount => {
                delegate_call_discrete(request, codec, |params| async move {
                    self.inner.create_account(scope, params).await
                })
                .await
                .map(|response| response.map_body(Either::Left))
            }
            meta::AccountProcedureId::CreateAccountGroup => {
                delegate_call_discrete(request, codec, |params| async move {
                    self.inner.create_account_group(scope, params).await
                })
                .await
                .map(|response| response.map_body(Either::Left).map_body(Either::Right))
            }
            meta::AccountProcedureId::CheckAccountGroupPermission => delegate_call_discrete(
                request,
                codec,
                |(account_group_id, permission)| async move {
                    self.inner
                        .check_account_group_permission(scope, account_group_id, permission)
                        .await
                },
            )
            .await
            .map(|response| {
                response
                    .map_body(Either::Left)
                    .map_body(Either::Right)
                    .map_body(Either::Right)
            }),
            meta::AccountProcedureId::AddAccountGroupMember => delegate_call_discrete(
                request,
                codec,
                |(account_group_id, account_id)| async move {
                    self.inner
                        .add_account_group_member(scope, account_group_id, account_id)
                        .await
                },
            )
            .await
            .map(|response| {
                response
                    .map_body(Either::Left)
                    .map_body(Either::Right)
                    .map_body(Either::Right)
                    .map_body(Either::Right)
            }),
            meta::AccountProcedureId::RemoveAccountGroupMember => delegate_call_discrete(
                request,
                codec,
                |(account_group_id, account_id)| async move {
                    self.inner
                        .remove_account_group_member(scope, account_group_id, account_id)
                        .await
                },
            )
            .await
            .map(|response| {
                response
                    .map_body(Either::Right)
                    .map_body(Either::Right)
                    .map_body(Either::Right)
                    .map_body(Either::Right)
            }),
        }
    }
}

// TODO: this can be auto generated by the `harpc` crate
#[derive_where::derive_where(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct AccountClient<S, C> {
    _session: PhantomData<fn() -> *const S>,
    _codec: PhantomData<fn() -> *const C>,
}

impl<S, C> AccountClient<S, C> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            _session: PhantomData,
            _codec: PhantomData,
        }
    }
}

impl<S, C> Default for AccountClient<S, C> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S, C> AccountSystem for AccountClient<S, C>
where
    S: harpc_client::connection::ConnectionService<C>,
    C: harpc_client::connection::ConnectionCodec,
{
    type ExecutionScope = Connection<S, C>;

    async fn create_account(
        &self,
        scope: Connection<S, C>,
        params: InsertAccountIdParams,
    ) -> Result<ActorId, Report<AccountError>> {
        invoke_call_discrete(scope, meta::AccountProcedureId::CreateAccount, [params])
            .await
            .change_context(AccountError)
    }

    async fn create_account_group(
        &self,
        scope: Connection<S, C>,
        params: InsertAccountGroupIdParams,
    ) -> Result<ActorGroupId, Report<AccountError>> {
        invoke_call_discrete(
            scope,
            meta::AccountProcedureId::CreateAccountGroup,
            [params],
        )
        .await
        .change_context(AccountError)
    }

    async fn check_account_group_permission(
        &self,
        scope: Connection<S, C>,
        account_group_id: ActorGroupId,
        permission: AccountGroupPermission,
    ) -> Result<PermissionResponse, Report<AccountError>> {
        invoke_call_discrete(
            scope,
            meta::AccountProcedureId::CheckAccountGroupPermission,
            [(account_group_id, permission)],
        )
        .await
        .change_context(AccountError)
    }

    async fn add_account_group_member(
        &self,
        scope: Connection<S, C>,
        account_group_id: ActorGroupId,
        account_id: ActorId,
    ) -> Result<(), Report<AccountError>> {
        invoke_call_discrete(
            scope,
            meta::AccountProcedureId::AddAccountGroupMember,
            [(account_group_id, account_id)],
        )
        .await
        .change_context(AccountError)
    }

    async fn remove_account_group_member(
        &self,
        scope: Connection<S, C>,
        account_group_id: ActorGroupId,
        account_id: ActorId,
    ) -> Result<(), Report<AccountError>> {
        invoke_call_discrete(
            scope,
            meta::AccountProcedureId::RemoveAccountGroupMember,
            [(account_group_id, account_id)],
        )
        .await
        .change_context(AccountError)
    }
}
