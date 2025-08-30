use crate::application::commands::transaction::{
    BalanceQuery, CancelOrderCommand, GenerateDebugTransactionCommand, PayTransactionCommand,
    RechargeCommand, SetPaymentPasswordCommand, TransactionDetailQuery, TransactionQuery,
};
use crate::application::service::transaction::{
    BalanceInfoDTO, TransactionApplicationService, TransactionApplicationServiceError,
    TransactionInfoDTO,
};
use crate::application::{ApplicationError, GeneralError, ModeError};
use crate::domain::Identifiable;
use crate::domain::model::session::SessionId;
use crate::domain::model::transaction::{Transaction, TransactionAmountAbs};
use crate::domain::model::user::{PaymentPassword, User, UserId};
use crate::domain::repository::transaction::TransactionRepository;
use crate::domain::repository::user::UserRepository;
use crate::domain::service::order::order_dto::TransactionDataDto;
use crate::domain::service::session::SessionManagerService;
use crate::domain::service::transaction::{TransactionService, TransactionServiceError};
use crate::domain::service::user::{UserService, UserServiceError};
use async_trait::async_trait;
use rust_decimal::Decimal;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use shared::utils::TimeMeter;
use std::sync::Arc;
use tracing::{error, info, instrument, warn};

pub struct TransactionApplicationServiceImpl<S, T, R, U, UR>
where
    S: SessionManagerService,
    T: TransactionService,
    R: TransactionRepository,
    U: UserService,
    UR: UserRepository,
{
    debug_mode: bool,
    session_manager: Arc<S>,
    transaction_service: Arc<T>,
    transaction_repository: Arc<R>,
    user_service: Arc<U>,
    user_repository: Arc<UR>,
}

impl<S, T, R, U, UR> TransactionApplicationServiceImpl<S, T, R, U, UR>
where
    S: SessionManagerService,
    T: TransactionService,
    R: TransactionRepository,
    U: UserService,
    UR: UserRepository,
{
    pub fn new(
        debug_mode: bool,
        session_manager: Arc<S>,
        transaction_service: Arc<T>,
        transaction_repository: Arc<R>,
        user_service: Arc<U>,
        user_repository: Arc<UR>,
    ) -> Self {
        Self {
            debug_mode,
            session_manager,
            transaction_service,
            transaction_repository,
            user_service,
            user_repository,
        }
    }
    async fn get_user_id_by_session_id(
        &self,
        session_id: &str,
    ) -> Result<UserId, Box<dyn ApplicationError>> {
        let session_id = SessionId::try_from(session_id)
            .map_err(|_for_super_earth| GeneralError::InvalidSessionId)?;

        let user_id = self
            .session_manager
            .get_user_id_by_session(session_id)
            .await
            .map_err(|_for_super_earth| GeneralError::InternalServerError)?
            .ok_or(GeneralError::InvalidSessionId)?;

        Ok(user_id)
    }

    async fn get_user_by_session_id(
        &self,
        session_id: &str,
    ) -> Result<User, Box<dyn ApplicationError>> {
        let user_id = self.get_user_id_by_session_id(session_id).await?;

        let user = self
            .user_repository
            .find(user_id)
            .await
            .map_err(|_for_super_earth| GeneralError::InternalServerError)?
            .ok_or(GeneralError::InvalidSessionId)?;

        Ok(user)
    }

    async fn verify_user_password(
        &self,
        user: &User,
        user_password: String,
    ) -> Result<(), Box<dyn ApplicationError>> {
        self.user_service
            .verify_password(user, user_password)
            .await
            .map_err(|e| match e {
                UserServiceError::InvalidPassword => {
                    Box::new(TransactionApplicationServiceError::WrongUserPassword)
                        as Box<dyn ApplicationError>
                }
                _ => Box::new(GeneralError::InternalServerError) as Box<dyn ApplicationError>,
            })?;

        Ok(())
    }

    async fn verify_payment_password(
        &self,
        user: &User,
        payment_password: String,
    ) -> Result<(), Box<dyn ApplicationError>> {
        self.user_service
            .verify_payment_password(user, payment_password)
            .await
            .map_err(|e| match e {
                UserServiceError::InvalidPassword => {
                    Box::new(TransactionApplicationServiceError::WrongUserPassword)
                        as Box<dyn ApplicationError>
                }
                _ => Box::new(GeneralError::InternalServerError) as Box<dyn ApplicationError>,
            })?;

        Ok(())
    }
}

#[async_trait]
impl<S, T, R, U, UR> TransactionApplicationService
    for TransactionApplicationServiceImpl<S, T, R, U, UR>
where
    S: SessionManagerService,
    T: TransactionService,
    R: TransactionRepository,
    U: UserService,
    UR: UserRepository,
{
    #[instrument(skip(self))]
    async fn recharge(&self, command: RechargeCommand) -> Result<(), Box<dyn ApplicationError>> {
        let user_id = self
            .get_user_id_by_session_id(&command.session_id)
            .await
            .inspect_err(|e| {
                error!(
                    "failed to get user id by session id {}: {}",
                    command.session_id, e
                );
            })?;

        self.transaction_service
            .recharge(
                user_id,
                TransactionAmountAbs::from(Decimal::from_f64(command.amount).ok_or(
                    GeneralError::BadRequest(format!("Invalid amount: {}", command.amount)),
                )?),
            )
            .await
            .inspect_err(|e| {
                error!("failed to recharge user {}: {}", user_id, e);
            })?;

        Ok(())
    }

    #[instrument(skip(self))]
    async fn query_balance(
        &self,
        query: BalanceQuery,
    ) -> Result<BalanceInfoDTO, Box<dyn ApplicationError>> {
        let user_id = self.get_user_id_by_session_id(&query.session_id).await?;

        let balance = self.transaction_service.get_balance(user_id).await?;

        Ok(BalanceInfoDTO {
            balance: balance.to_f64().unwrap(),
        })
    }

    #[instrument(skip(self))]
    async fn query_transactions(
        &self,
        query: TransactionQuery,
    ) -> Result<Vec<TransactionInfoDTO>, Box<dyn ApplicationError>> {
        let user_id = self.get_user_id_by_session_id(&query.session_id).await?;

        let tx_list = self
            .transaction_repository
            .find_by_user_id(user_id)
            .await
            .map_err(|e| {
                error!("failed to find tx list for user_id {}: {}", user_id, e);
                GeneralError::InternalServerError
            })?;

        Ok(tx_list.into_iter().map(|item| item.into()).collect())
    }

    #[instrument(skip(self, command))]
    async fn set_payment_password(
        &self,
        command: SetPaymentPasswordCommand,
    ) -> Result<(), Box<dyn ApplicationError>> {
        let user = self.get_user_by_session_id(&command.session_id).await?;

        self.verify_user_password(&user, command.user_password)
            .await?;

        let payment_password = PaymentPassword::try_from(command.payment_password.as_str())
            .map_err(|_for_super_earth| {
                TransactionApplicationServiceError::InvalidPaymentPasswordFormat
            })?;

        self.user_service
            .set_payment_password(user.get_id().unwrap(), Some(payment_password))
            .await
            .map_err(|e| {
                error!(
                    "failed to set payment password for user: {}: {}",
                    user.get_id().unwrap(),
                    e
                );
                GeneralError::InternalServerError
            })?;

        self.user_service
            .clear_wrong_payment_password_tried(user.get_id().unwrap())
            .await
            .map_err(|e| {
                error!(
                    "failed to clear wrong payment password tried times for user: {}: {}",
                    user.get_id().unwrap(),
                    e
                );
                GeneralError::InternalServerError
            })?;

        Ok(())
    }

    #[instrument(skip(self))]
    async fn pay_transaction(
        &self,
        command: PayTransactionCommand,
    ) -> Result<(), Box<dyn ApplicationError>> {
        let user = self.get_user_by_session_id(&command.session_id).await?;

        if let Some(user_password) = command.user_password {
            self.verify_user_password(&user, user_password).await?;
        } else if let Some(payment_password) = command.payment_password {
            self.verify_payment_password(&user, payment_password)
                .await?;
        } else {
            return Err(Box::new(GeneralError::BadRequest(
                "Neither user password nor payment password was set".to_string(),
            )));
        }

        self.transaction_service
            .pay_transaction(command.transaction_id)
            .await?;

        Ok(())
    }

    #[instrument(skip(self))]
    async fn generate_debug_transaction(
        &self,
        command: GenerateDebugTransactionCommand,
    ) -> Result<TransactionInfoDTO, Box<dyn ApplicationError>> {
        if !self.debug_mode {
            return Err(Box::new(ModeError));
        }

        let user_id = self.get_user_id_by_session_id(&command.session_id).await?;

        let amount = TransactionAmountAbs::from_f64_checked(command.amount)
            .map_err(|e| GeneralError::BadRequest(e.to_string()))?;

        let mut tx = Transaction::new_debug(user_id, amount);

        self.transaction_repository
            .save(&mut tx)
            .await
            .map_err(|e| {
                error!("failed to save transaction: {}", e);

                GeneralError::InternalServerError
            })?;

        Ok(tx.into())
    }

    #[instrument(skip(self))]
    async fn query_transaction_details(
        &self,
        query: TransactionDetailQuery,
    ) -> Result<Vec<TransactionDataDto>, Box<dyn ApplicationError>> {
        let mut meter = TimeMeter::new("query_transaction_details");

        let user_id = self.get_user_id_by_session_id(&query.session_id).await?;

        let transaction_list = self
            .transaction_repository
            .find_by_user_id(user_id)
            .await
            .inspect_err(|e| error!("failed to find transactions for user_id {}: {}", user_id, e))
            .map_err(|e| {
                error!("failed to query transaction for user_id {}: {}", user_id, e);

                GeneralError::InternalServerError
            })?;

        meter.meter("load transaction list");

        let mut result = Vec::with_capacity(transaction_list.len());

        for transaction in transaction_list {
            let txid = transaction.get_id();
            result.push(
                self.transaction_service
                    .convert_transaction_to_dto(transaction)
                    .await
                    .inspect_err(|e| {
                        error!("failed to convert transaction {:?} to dto: {}", txid, e);
                    })
                    .map_err(|e| {
                        error!("failed to convert transaction {:?} to dto {}", txid, e);

                        GeneralError::InternalServerError
                    })?,
            )
        }

        meter.meter("convert transactions to DTO");

        info!("{}", meter);

        Ok(result)
    }

    #[instrument(skip(self))]
    async fn cancel_order(
        &self,
        command: CancelOrderCommand,
    ) -> Result<(), Box<dyn ApplicationError>> {
        let user_id = self.get_user_id_by_session_id(&command.session_id).await?;

        let tx_list = self
            .transaction_repository
            .find_by_user_id(user_id)
            .await
            .map_err(|e| {
                error!("failed to find tx list for user_id {}: {}", user_id, e);
                GeneralError::InternalServerError
            })?;

        let target_order_uuid = command.order_id;

        let mut target_tx = None;
        let mut target_order = None;

        for tx in &tx_list {
            for order in tx.orders() {
                if order.uuid() == target_order_uuid {
                    target_order = Some(order);
                    target_tx = Some(tx);
                    break;
                }
            }
        }

        let target_tx = target_tx.ok_or_else(|| {
            warn!("No transaction found for order id {}", target_order_uuid);
            GeneralError::NotFound(format!(
                "No transaction found for order id {}",
                target_order_uuid
            ))
        })?;

        let target_order = target_order.unwrap();

        self.transaction_service
            .refund_transaction(target_tx.uuid(), std::slice::from_ref(target_order))
            .await
            .map_err(|e| match e {
                TransactionServiceError::RefundError(e) => Box::new(
                    TransactionApplicationServiceError::RefundError(e.to_string()),
                )
                    as Box<dyn ApplicationError>,
                x => {
                    error!("Failed to refund order {}: {}", target_order_uuid, x);
                    Box::new(GeneralError::InternalServerError) as Box<dyn ApplicationError>
                }
            })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::service::transaction::TransactionApplicationService;
    use crate::domain::model::order::{
        BaseOrder, Order, OrderId, OrderStatus, OrderTimeInfo, OrderType, PaymentInfo,
    };
    use crate::domain::model::password::{HashedPassword, PasswordSalt};
    use crate::domain::model::personal_info::PersonalInfoId;
    use crate::domain::model::user::{
        IdentityCardId, Phone, RawPassword, RealName, UserInfo, Username,
    };
    use crate::domain::{Repository, RepositoryError};
    use rust_decimal::Decimal;

    use uuid::Uuid;

    // -------- Test helpers --------
    fn make_user(id: u64) -> User {
        let username = Username::try_from("tester".to_string()).unwrap();
        let hp = HashedPassword {
            hashed_password: vec![1, 2, 3],
            salt: PasswordSalt::from(vec![9, 9, 9]),
        };
        let info = UserInfo::new(
            RealName::try_from("Alice".to_string()).unwrap(),
            None,
            None,
            Phone::try_from("13012345678".to_string()).unwrap(),
            None,
            IdentityCardId::try_from("11010519491231002X".to_string()).unwrap(),
        );
        User::new(
            Some(UserId::from(id)),
            username,
            hp,
            None,
            Default::default(),
            info,
        )
    }

    #[derive(Debug, Clone)]
    struct DummyOrder {
        base: BaseOrder,
        ty: OrderType,
    }
    impl Order for DummyOrder {
        fn order_id(&self) -> Option<OrderId> {
            self.base.order_id
        }
        fn uuid(&self) -> Uuid {
            self.base.uuid
        }
        fn already_refund(&self) -> bool {
            self.base.payment_info.refund_transaction_id().is_some()
        }
        fn order_status(&self) -> OrderStatus {
            self.base.order_status
        }
        fn order_type(&self) -> OrderType {
            self.ty
        }
        fn order_time_info(&self) -> OrderTimeInfo {
            self.base.order_time_info
        }
        fn unit_price(&self) -> Decimal {
            self.base.unit_price
        }
        fn amount(&self) -> Decimal {
            self.base.amount
        }
        fn payment_info(&self) -> PaymentInfo {
            self.base.payment_info
        }
        fn payment_info_mut(&mut self) -> &mut PaymentInfo {
            &mut self.base.payment_info
        }
        fn personal_info_id(&self) -> PersonalInfoId {
            self.base.personal_info_id
        }
        fn set_status(&mut self, status: OrderStatus) {
            self.base.order_status = status;
        }
    }
    fn base_order(price_cent: i64, amount: i64, status: OrderStatus) -> BaseOrder {
        let now: sea_orm::prelude::DateTimeWithTimeZone = chrono::Utc::now().into();
        BaseOrder::new(
            None,
            Uuid::new_v4(),
            status,
            OrderTimeInfo::new(now, now, now),
            Decimal::new(price_cent, 2),
            Decimal::new(amount, 0),
            PaymentInfo::new(None, None),
            PersonalInfoId::from(1_u64),
        )
    }

    // -------- Stubs --------
    struct SessOk;
    #[async_trait]
    impl SessionManagerService for SessOk {
        async fn create_session(
            &self,
            _user_id: UserId,
        ) -> Result<crate::domain::model::session::Session, RepositoryError> {
            unimplemented!()
        }
        async fn delete_session(
            &self,
            _session: crate::domain::model::session::Session,
        ) -> Result<(), RepositoryError> {
            unimplemented!()
        }
        async fn get_session(
            &self,
            _session_id: SessionId,
        ) -> Result<Option<crate::domain::model::session::Session>, RepositoryError> {
            Ok(None)
        }
        async fn get_user_id_by_session(
            &self,
            _session_id: SessionId,
        ) -> Result<Option<UserId>, RepositoryError> {
            Ok(Some(UserId::from(1_u64)))
        }
        async fn verify_session_id(&self, _session_id_str: &str) -> Result<bool, RepositoryError> {
            Ok(true)
        }
    }

    struct UserRepoOk;
    #[async_trait]
    impl Repository<User> for UserRepoOk {
        async fn find(&self, id: UserId) -> Result<Option<User>, RepositoryError> {
            Ok(Some(make_user(id.into())))
        }
        async fn remove(&self, _aggregate: User) -> Result<(), RepositoryError> {
            Ok(())
        }
        async fn save(&self, _aggregate: &mut User) -> Result<UserId, RepositoryError> {
            Ok(UserId::from(1_u64))
        }
    }
    #[async_trait]
    impl UserRepository for UserRepoOk {
        async fn find_by_phone(&self, _phone: Phone) -> Result<Option<User>, RepositoryError> {
            Ok(None)
        }
        async fn find_by_identity_card_id(
            &self,
            _identity_card_id: IdentityCardId,
        ) -> Result<Option<User>, RepositoryError> {
            Ok(None)
        }
        async fn remove_by_phone(&self, _phone: Phone) -> Result<(), RepositoryError> {
            Ok(())
        }
    }

    struct UserSvc {
        verify_ok: bool,
        verify_payment_ok: bool,
    }
    #[async_trait]
    impl UserService for UserSvc {
        async fn register(
            &self,
            _username: Username,
            _raw_password: RawPassword,
            _name: RealName,
            _phone: Phone,
            _identity_card_id: IdentityCardId,
        ) -> Result<(), UserServiceError> {
            unimplemented!()
        }
        async fn delete(&self, _phone: Phone) -> Result<(), UserServiceError> {
            unimplemented!()
        }
        async fn verify_password(
            &self,
            _user: &User,
            _raw_password: String,
        ) -> Result<(), UserServiceError> {
            if self.verify_ok {
                Ok(())
            } else {
                Err(UserServiceError::InvalidPassword)
            }
        }
        async fn verify_payment_password(
            &self,
            _user: &User,
            _raw_payment_password: String,
        ) -> Result<(), UserServiceError> {
            if self.verify_payment_ok {
                Ok(())
            } else {
                Err(UserServiceError::InvalidPassword)
            }
        }
        async fn set_password(
            &self,
            _user_id: UserId,
            _raw_password: String,
        ) -> Result<(), UserServiceError> {
            Ok(())
        }
        async fn set_payment_password(
            &self,
            _user_id: UserId,
            _payment_password: Option<PaymentPassword>,
        ) -> Result<(), UserServiceError> {
            Ok(())
        }
        async fn set_wrong_payment_password_tried(
            &self,
            _user_id: UserId,
            _password_attempts: crate::domain::model::user::PasswordAttempts,
        ) -> Result<(), UserServiceError> {
            Ok(())
        }
        async fn clear_wrong_payment_password_tried(
            &self,
            _user_id: UserId,
        ) -> Result<(), UserServiceError> {
            Ok(())
        }
        async fn increment_wrong_payment_password_tried(
            &self,
            _user_id: UserId,
        ) -> Result<(), UserServiceError> {
            Ok(())
        }
        async fn set_user_info(
            &self,
            _user_id: UserId,
            _user_info: UserInfo,
        ) -> Result<(), UserServiceError> {
            Ok(())
        }
    }

    struct TxSvc {
        recharge_err: bool,
        balance: Decimal,
        pay_err: bool,
        refund_err: bool,
        convert_err: bool,
    }
    #[async_trait]
    impl TransactionService for TxSvc {
        async fn recharge(
            &self,
            _user_id: UserId,
            _amount: TransactionAmountAbs,
        ) -> Result<Uuid, TransactionServiceError> {
            if self.recharge_err {
                Err(TransactionServiceError::InsufficientFunds {
                    transaction_id: Uuid::new_v4(),
                    balance: Decimal::ZERO,
                    amount: TransactionAmountAbs::from(Decimal::new(1, 0)),
                })
            } else {
                Ok(Uuid::new_v4())
            }
        }
        async fn get_balance(&self, _user_id: UserId) -> Result<Decimal, TransactionServiceError> {
            Ok(self.balance)
        }
        async fn new_transaction(
            &self,
            _user_id: UserId,
            _orders: Vec<Box<dyn Order>>,
            _atomic: bool,
        ) -> Result<Uuid, TransactionServiceError> {
            unimplemented!()
        }
        async fn pay_transaction(
            &self,
            _transaction_id: Uuid,
        ) -> Result<(), TransactionServiceError> {
            if self.pay_err {
                Err(TransactionServiceError::InvalidTransactionId(Uuid::new_v4()))
            } else {
                Ok(())
            }
        }
        async fn refund_transaction(
            &self,
            _transaction_id: Uuid,
            _to_refund_orders: &[Box<dyn Order>],
        ) -> Result<Uuid, TransactionServiceError> {
            if self.refund_err {
                Err(TransactionServiceError::RefundError(
                    crate::domain::model::transaction::RefundError::NotPaid(Uuid::new_v4()),
                ))
            } else {
                Ok(Uuid::new_v4())
            }
        }
        async fn convert_transaction_to_dto(
            &self,
            transaction: Transaction,
        ) -> Result<TransactionDataDto, TransactionServiceError> {
            if self.convert_err {
                Err(TransactionServiceError::InvalidTransactionId(
                    transaction.uuid(),
                ))
            } else {
                Ok(TransactionDataDto {
                    transaction_id: transaction.uuid().to_string(),
                    status: transaction.status().to_string(),
                    create_time: "t".into(),
                    pay_time: None,
                    orders: vec![],
                    amount: transaction.raw_amount().to_f64().unwrap(),
                })
            }
        }
    }

    struct TxRepo {
        tx_list: Vec<Transaction>,
        find_err: bool,
        save_err: bool,
    }
    #[async_trait]
    impl Repository<Transaction> for TxRepo {
        async fn find(
            &self,
            _id: crate::domain::model::transaction::TransactionId,
        ) -> Result<Option<Transaction>, RepositoryError> {
            Ok(None)
        }
        async fn remove(&self, _aggregate: Transaction) -> Result<(), RepositoryError> {
            Ok(())
        }
        async fn save(
            &self,
            _aggregate: &mut Transaction,
        ) -> Result<crate::domain::model::transaction::TransactionId, RepositoryError> {
            if self.save_err {
                Err(RepositoryError::Db(anyhow::anyhow!("x")))
            } else {
                Ok(crate::domain::model::transaction::TransactionId::from(
                    1_u64,
                ))
            }
        }
    }
    #[async_trait]
    impl TransactionRepository for TxRepo {
        async fn find_by_uuid(&self, _uuid: Uuid) -> Result<Option<Transaction>, RepositoryError> {
            Ok(None)
        }
        async fn find_by_user_id(
            &self,
            _user_id: UserId,
        ) -> Result<Vec<Transaction>, RepositoryError> {
            if self.find_err {
                Err(RepositoryError::Db(anyhow::anyhow!("fail")))
            } else {
                Ok(self.tx_list.clone())
            }
        }
        async fn get_user_balance(
            &self,
            _user_id: UserId,
        ) -> Result<Option<Decimal>, RepositoryError> {
            Ok(None)
        }
    }

    // -------- Tests --------
    type AppType = TransactionApplicationServiceImpl<SessOk, TxSvc, TxRepo, UserSvc, UserRepoOk>;
    fn make_app(
        debug: bool,
        sess: Arc<SessOk>,
        txs: Arc<TxSvc>,
        repo: Arc<TxRepo>,
        us: Arc<UserSvc>,
        ur: Arc<UserRepoOk>,
    ) -> AppType {
        TransactionApplicationServiceImpl::new(debug, sess, txs, repo, us, ur)
    }

    #[tokio::test]
    async fn recharge_invalid_session_format() {
        let app = make_app(
            true,
            Arc::new(SessOk),
            Arc::new(TxSvc {
                recharge_err: false,
                balance: Decimal::ZERO,
                pay_err: false,
                refund_err: false,
                convert_err: false,
            }),
            Arc::new(TxRepo {
                tx_list: vec![],
                find_err: false,
                save_err: false,
            }),
            Arc::new(UserSvc {
                verify_ok: true,
                verify_payment_ok: true,
            }),
            Arc::new(UserRepoOk),
        );
        let err = app
            .recharge(RechargeCommand {
                session_id: "not-a-uuid".into(),
                amount: 10.0,
            })
            .await
            .unwrap_err();
        assert_eq!(err.error_code(), 403);
    }

    #[tokio::test]
    async fn recharge_invalid_amount_nan() {
        let app = make_app(
            true,
            Arc::new(SessOk),
            Arc::new(TxSvc {
                recharge_err: false,
                balance: Decimal::ZERO,
                pay_err: false,
                refund_err: false,
                convert_err: false,
            }),
            Arc::new(TxRepo {
                tx_list: vec![],
                find_err: false,
                save_err: false,
            }),
            Arc::new(UserSvc {
                verify_ok: true,
                verify_payment_ok: true,
            }),
            Arc::new(UserRepoOk),
        );
        let err = app
            .recharge(RechargeCommand {
                session_id: Uuid::new_v4().to_string(),
                amount: f64::NAN,
            })
            .await
            .unwrap_err();
        assert_eq!(err.error_code(), 400);
    }

    #[tokio::test]
    async fn recharge_insufficient_funds_mapping() {
        let app = make_app(
            true,
            Arc::new(SessOk),
            Arc::new(TxSvc {
                recharge_err: true,
                balance: Decimal::ZERO,
                pay_err: false,
                refund_err: false,
                convert_err: false,
            }),
            Arc::new(TxRepo {
                tx_list: vec![],
                find_err: false,
                save_err: false,
            }),
            Arc::new(UserSvc {
                verify_ok: true,
                verify_payment_ok: true,
            }),
            Arc::new(UserRepoOk),
        );
        let err = app
            .recharge(RechargeCommand {
                session_id: Uuid::new_v4().to_string(),
                amount: 1.0,
            })
            .await
            .unwrap_err();
        assert_eq!(err.error_code(), 11004);
    }

    #[tokio::test]
    async fn query_balance_success() {
        let app = make_app(
            true,
            Arc::new(SessOk),
            Arc::new(TxSvc {
                recharge_err: false,
                balance: Decimal::new(12345, 2),
                pay_err: false,
                refund_err: false,
                convert_err: false,
            }),
            Arc::new(TxRepo {
                tx_list: vec![],
                find_err: false,
                save_err: false,
            }),
            Arc::new(UserSvc {
                verify_ok: true,
                verify_payment_ok: true,
            }),
            Arc::new(UserRepoOk),
        );
        let dto = app
            .query_balance(BalanceQuery {
                session_id: Uuid::new_v4().to_string(),
            })
            .await
            .unwrap();
        assert!((dto.balance - 123.45).abs() < 1e-6);
    }

    #[tokio::test]
    async fn query_transactions_repo_error() {
        let app = make_app(
            true,
            Arc::new(SessOk),
            Arc::new(TxSvc {
                recharge_err: false,
                balance: Decimal::ZERO,
                pay_err: false,
                refund_err: false,
                convert_err: false,
            }),
            Arc::new(TxRepo {
                tx_list: vec![],
                find_err: true,
                save_err: false,
            }),
            Arc::new(UserSvc {
                verify_ok: true,
                verify_payment_ok: true,
            }),
            Arc::new(UserRepoOk),
        );
        let err = app
            .query_transactions(TransactionQuery {
                session_id: Uuid::new_v4().to_string(),
            })
            .await
            .unwrap_err();
        assert_eq!(err.error_code(), 500);
    }

    #[tokio::test]
    async fn set_payment_password_invalid_format() {
        let app = make_app(
            true,
            Arc::new(SessOk),
            Arc::new(TxSvc {
                recharge_err: false,
                balance: Decimal::ZERO,
                pay_err: false,
                refund_err: false,
                convert_err: false,
            }),
            Arc::new(TxRepo {
                tx_list: vec![],
                find_err: false,
                save_err: false,
            }),
            Arc::new(UserSvc {
                verify_ok: true,
                verify_payment_ok: true,
            }),
            Arc::new(UserRepoOk),
        );
        let err = app
            .set_payment_password(SetPaymentPasswordCommand {
                session_id: Uuid::new_v4().to_string(),
                user_password: "abc".into(),
                payment_password: "12x456".into(),
            })
            .await
            .unwrap_err();
        assert_eq!(err.error_code(), 11007);
    }

    #[tokio::test]
    async fn set_payment_password_wrong_user_password() {
        let app = make_app(
            true,
            Arc::new(SessOk),
            Arc::new(TxSvc {
                recharge_err: false,
                balance: Decimal::ZERO,
                pay_err: false,
                refund_err: false,
                convert_err: false,
            }),
            Arc::new(TxRepo {
                tx_list: vec![],
                find_err: false,
                save_err: false,
            }),
            Arc::new(UserSvc {
                verify_ok: false,
                verify_payment_ok: true,
            }),
            Arc::new(UserRepoOk),
        );
        let err = app
            .set_payment_password(SetPaymentPasswordCommand {
                session_id: Uuid::new_v4().to_string(),
                user_password: "bad".into(),
                payment_password: "123456".into(),
            })
            .await
            .unwrap_err();
        assert_eq!(err.error_code(), 11002);
    }

    #[tokio::test]
    async fn pay_transaction_missing_passwords() {
        let app = make_app(
            true,
            Arc::new(SessOk),
            Arc::new(TxSvc {
                recharge_err: false,
                balance: Decimal::ZERO,
                pay_err: false,
                refund_err: false,
                convert_err: false,
            }),
            Arc::new(TxRepo {
                tx_list: vec![],
                find_err: false,
                save_err: false,
            }),
            Arc::new(UserSvc {
                verify_ok: true,
                verify_payment_ok: true,
            }),
            Arc::new(UserRepoOk),
        );
        let err = app
            .pay_transaction(PayTransactionCommand {
                session_id: Uuid::new_v4().to_string(),
                transaction_id: Uuid::new_v4(),
                user_password: None,
                payment_password: None,
            })
            .await
            .unwrap_err();
        assert_eq!(err.error_code(), 400);
    }

    #[tokio::test]
    async fn pay_transaction_wrong_payment_password_maps_to_user_wrong() {
        let app = make_app(
            true,
            Arc::new(SessOk),
            Arc::new(TxSvc {
                recharge_err: false,
                balance: Decimal::ZERO,
                pay_err: false,
                refund_err: false,
                convert_err: false,
            }),
            Arc::new(TxRepo {
                tx_list: vec![],
                find_err: false,
                save_err: false,
            }),
            Arc::new(UserSvc {
                verify_ok: true,
                verify_payment_ok: false,
            }),
            Arc::new(UserRepoOk),
        );
        let err = app
            .pay_transaction(PayTransactionCommand {
                session_id: Uuid::new_v4().to_string(),
                transaction_id: Uuid::new_v4(),
                user_password: None,
                payment_password: Some("123456".into()),
            })
            .await
            .unwrap_err();
        assert_eq!(err.error_code(), 11002);
    }

    #[tokio::test]
    async fn generate_debug_transaction_mode_off() {
        let app = make_app(
            false,
            Arc::new(SessOk),
            Arc::new(TxSvc {
                recharge_err: false,
                balance: Decimal::ZERO,
                pay_err: false,
                refund_err: false,
                convert_err: false,
            }),
            Arc::new(TxRepo {
                tx_list: vec![],
                find_err: false,
                save_err: false,
            }),
            Arc::new(UserSvc {
                verify_ok: true,
                verify_payment_ok: true,
            }),
            Arc::new(UserRepoOk),
        );
        let err = app
            .generate_debug_transaction(GenerateDebugTransactionCommand {
                session_id: Uuid::new_v4().to_string(),
                amount: 12.3,
            })
            .await
            .unwrap_err();
        assert_eq!(err.error_code(), 403);
    }

    #[tokio::test]
    async fn generate_debug_transaction_negative_amount() {
        let app = make_app(
            true,
            Arc::new(SessOk),
            Arc::new(TxSvc {
                recharge_err: false,
                balance: Decimal::ZERO,
                pay_err: false,
                refund_err: false,
                convert_err: false,
            }),
            Arc::new(TxRepo {
                tx_list: vec![],
                find_err: false,
                save_err: false,
            }),
            Arc::new(UserSvc {
                verify_ok: true,
                verify_payment_ok: true,
            }),
            Arc::new(UserRepoOk),
        );
        let err = app
            .generate_debug_transaction(GenerateDebugTransactionCommand {
                session_id: Uuid::new_v4().to_string(),
                amount: -1.0,
            })
            .await
            .unwrap_err();
        assert_eq!(err.error_code(), 400);
    }

    #[tokio::test]
    async fn generate_debug_transaction_repo_save_error() {
        let app = make_app(
            true,
            Arc::new(SessOk),
            Arc::new(TxSvc {
                recharge_err: false,
                balance: Decimal::ZERO,
                pay_err: false,
                refund_err: false,
                convert_err: false,
            }),
            Arc::new(TxRepo {
                tx_list: vec![],
                find_err: false,
                save_err: true,
            }),
            Arc::new(UserSvc {
                verify_ok: true,
                verify_payment_ok: true,
            }),
            Arc::new(UserRepoOk),
        );
        let err = app
            .generate_debug_transaction(GenerateDebugTransactionCommand {
                session_id: Uuid::new_v4().to_string(),
                amount: 1.0,
            })
            .await
            .unwrap_err();
        assert_eq!(err.error_code(), 500);
    }

    #[tokio::test]
    async fn generate_debug_transaction_success() {
        let app = make_app(
            true,
            Arc::new(SessOk),
            Arc::new(TxSvc {
                recharge_err: false,
                balance: Decimal::ZERO,
                pay_err: false,
                refund_err: false,
                convert_err: false,
            }),
            Arc::new(TxRepo {
                tx_list: vec![],
                find_err: false,
                save_err: false,
            }),
            Arc::new(UserSvc {
                verify_ok: true,
                verify_payment_ok: true,
            }),
            Arc::new(UserRepoOk),
        );
        let dto = app
            .generate_debug_transaction(GenerateDebugTransactionCommand {
                session_id: Uuid::new_v4().to_string(),
                amount: 12.34,
            })
            .await
            .unwrap();
        assert!((dto.amount - 12.34).abs() < 1e-6);
        assert_eq!(dto.status, "unpaid");
    }

    #[tokio::test]
    async fn query_transaction_details_convert_error() {
        let tx = Transaction::new_debug(
            UserId::from(1_u64),
            TransactionAmountAbs::from(Decimal::new(1000, 2)),
        );
        let app = make_app(
            true,
            Arc::new(SessOk),
            Arc::new(TxSvc {
                recharge_err: false,
                balance: Decimal::ZERO,
                pay_err: false,
                refund_err: false,
                convert_err: true,
            }),
            Arc::new(TxRepo {
                tx_list: vec![tx],
                find_err: false,
                save_err: false,
            }),
            Arc::new(UserSvc {
                verify_ok: true,
                verify_payment_ok: true,
            }),
            Arc::new(UserRepoOk),
        );
        let res = app
            .query_transaction_details(TransactionDetailQuery {
                session_id: Uuid::new_v4().to_string(),
            })
            .await;
        assert!(res.is_err());
        let err = res.err().unwrap();
        assert_eq!(err.error_code(), 500);
    }

    #[tokio::test]
    async fn query_transaction_details_success() {
        let tx = Transaction::new_debug(
            UserId::from(1_u64),
            TransactionAmountAbs::from(Decimal::new(1000, 2)),
        );
        let app = make_app(
            true,
            Arc::new(SessOk),
            Arc::new(TxSvc {
                recharge_err: false,
                balance: Decimal::ZERO,
                pay_err: false,
                refund_err: false,
                convert_err: false,
            }),
            Arc::new(TxRepo {
                tx_list: vec![tx],
                find_err: false,
                save_err: false,
            }),
            Arc::new(UserSvc {
                verify_ok: true,
                verify_payment_ok: true,
            }),
            Arc::new(UserRepoOk),
        );
        let list = app
            .query_transaction_details(TransactionDetailQuery {
                session_id: Uuid::new_v4().to_string(),
            })
            .await
            .unwrap();
        assert_eq!(list.len(), 1);
    }

    #[tokio::test]
    async fn cancel_order_not_found() {
        let tx = Transaction::new_debug(
            UserId::from(1_u64),
            TransactionAmountAbs::from(Decimal::new(1000, 2)),
        );
        let app = make_app(
            true,
            Arc::new(SessOk),
            Arc::new(TxSvc {
                recharge_err: false,
                balance: Decimal::ZERO,
                pay_err: false,
                refund_err: false,
                convert_err: false,
            }),
            Arc::new(TxRepo {
                tx_list: vec![tx],
                find_err: false,
                save_err: false,
            }),
            Arc::new(UserSvc {
                verify_ok: true,
                verify_payment_ok: true,
            }),
            Arc::new(UserRepoOk),
        );
        let err = app
            .cancel_order(CancelOrderCommand {
                session_id: Uuid::new_v4().to_string(),
                order_id: Uuid::new_v4(),
            })
            .await
            .unwrap_err();
        assert_eq!(err.error_code(), 404);
    }

    #[tokio::test]
    async fn cancel_order_refund_error_mapping() {
        // build tx with one order matching target order id
        let order_id = Uuid::new_v4();
        let order: Box<dyn Order> = Box::new(DummyOrder {
            base: BaseOrder {
                uuid: order_id,
                ..base_order(1000, 1, OrderStatus::Paid)
            },
            ty: OrderType::Train,
        });
        let tx = Transaction::new(UserId::from(1_u64), vec![order], false);

        let app = make_app(
            true,
            Arc::new(SessOk),
            Arc::new(TxSvc {
                recharge_err: false,
                balance: Decimal::ZERO,
                pay_err: false,
                refund_err: true,
                convert_err: false,
            }),
            Arc::new(TxRepo {
                tx_list: vec![tx],
                find_err: false,
                save_err: false,
            }),
            Arc::new(UserSvc {
                verify_ok: true,
                verify_payment_ok: true,
            }),
            Arc::new(UserRepoOk),
        );
        let err = app
            .cancel_order(CancelOrderCommand {
                session_id: Uuid::new_v4().to_string(),
                order_id,
            })
            .await
            .unwrap_err();
        assert_eq!(err.error_code(), 11005);
    }

    #[tokio::test]
    async fn cancel_order_success() {
        let order_id = Uuid::new_v4();
        let order: Box<dyn Order> = Box::new(DummyOrder {
            base: BaseOrder {
                uuid: order_id,
                ..base_order(1000, 1, OrderStatus::Paid)
            },
            ty: OrderType::Train,
        });
        let tx = Transaction::new(UserId::from(1_u64), vec![order], false);
        let app = make_app(
            true,
            Arc::new(SessOk),
            Arc::new(TxSvc {
                recharge_err: false,
                balance: Decimal::ZERO,
                pay_err: false,
                refund_err: false,
                convert_err: false,
            }),
            Arc::new(TxRepo {
                tx_list: vec![tx],
                find_err: false,
                save_err: false,
            }),
            Arc::new(UserSvc {
                verify_ok: true,
                verify_payment_ok: true,
            }),
            Arc::new(UserRepoOk),
        );
        assert!(
            app.cancel_order(CancelOrderCommand {
                session_id: Uuid::new_v4().to_string(),
                order_id
            })
            .await
            .is_ok()
        );
    }
}
