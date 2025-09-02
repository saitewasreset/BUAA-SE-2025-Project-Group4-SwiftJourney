# Split Prompt

> We will never forget those who fell in the defense of Malevelon Creek.

**以下内容仅用作AI提示词，请勿过度解读。**

1. 读取对应微服务Internal Service中的内容，确认要新增的内部接口。

    例如，对于`User`微服务：

    ```markdown
    - Internal Service
    - `+verify_password(user_id: UserId, raw_password: String) -> Result<bool>;`
    - `+verify_payment_password(user_id: UserId, raw_payment_password: String,) -> Result<bool>;`
    - `+set_payment_password(user_id: UserId, payment_password: Option<PaymentPassword>,) -> Result<()>;`
    - `+clear_wrong_payment_password_tried(user_id: UserId,) -> Result<()>;`
    - `+get_session(session_id: SessionId) -> Result<Option<Session>>;` + `+verify_session_id(session_id_str: &str) -> Result<bool>;` + `+get_user_id_by_session(session_id: SessionId) -> Result<Option<UserId>>;`
    - `+get_user_info(user_id: UserId) -> Result<Vec<PersonalInfo> + User + UserInfo>`
    - `+db_get_user_info() -> Result<Vec<models::User>>`
    - `+db_get_personal_info() -> Result<Vec<models::PerosnalInfo>>`
    ```

2. 对于每个有参数的Internal Service，根据CQRS在对应的crate中创建相关结构体。为了在不同微服务间共享相关结构体，请在`shared`crate中创建。

    例如，对于`User`微服务，在`shared/src/internal`中创建`user`模块，其中的`command.rs`包含CQRS结构体定义，内容如下（注意在`shared/src/internal/user/mod.rs`中添加模块`pub mod command`定义）：

    ```rust
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct VerifyPasswordCommand {
        pub user_id: u64,
        pub raw_password: String,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct VerifyPaymentPasswordCommand {
        pub user_id: u64,
        pub raw_payment_password: String,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct SetPaymentPasswordCommand {
        pub user_id: u64,
        pub raw_payment_password: Option<String>,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct ClearWrongPaymentPasswordTriedCommand {
        pub user_id: u64,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct SessionQuery {
        pub session_id: String,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct UserInfoQuery {
        pub user_id: u64,
    }
    ```

3. 创建Internal Service Trait，包含需要的DTO结构体定义、特征定义以及错误定义。若有你无法确定的类型，请询问我，不要擅自决定。
   1. 对于错误定义，对于调用其它Domain Service产生的错误，直接使用`anyhow::Error`包装。
   2. 对于简单类型，如`String`、`Option<bool>`，直接返回，无需定义DTO；对于复杂类型，如`UserInfo`、`PersonalInfo`，请定义DTO。即使DTO与对应的Domain Entity结构相同，也要定义DTO，并实现`From<Domain Entity>`，不要直接给Domain Entity添加`Serialize`特征。
   3. 对于以`db_`开头的方法，例如`db_get_user_info`，定义的DTO应当与`base/src/models`中的数据库Entity**完全相同**，并实现`Serialize`、`Deserialize`特征。DTO命名应当为`Db<数据库表名，首字母大写>DTO`。请为DTO实现`From<Database Entity>`。**虽然这是微服务拆分中的反模式，但严格按此执行。**

    例如，对于`User`微服务，在`shared/src/internal`中的`user`模块（已在上一步创建），其中的`dto.rs`包含CQRS结构体定义，内容如下（注意在`shared/src/internal/user/mod.rs`中添加模块`pub mod dto`定义）：

    ```rust
    use serde::{Deserialize, Serialize};
    use uuid::Uuid;

    #[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct PersonalInfoDTO {
        pub id: Option<u64>,
        pub uuid: Uuid,
        pub name: String,
        pub identity_card_id: String,
        pub preferred_seat_location: Option<char>,
        pub user_id: u64,
        pub is_default: bool,
    }

    #[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct UserInfoDTO {
        /// 用户真实姓名
        pub name: String,
        /// 用户性别(可选)
        pub gender: Option<String>,
        /// 用户年龄(可选)
        pub age: Option<u16>,
        /// 用户手机号码
        pub phone: String,
        /// 用户电子邮箱(可选)
        pub email: Option<String>,
        /// 用户身份证号
        pub identity_card_id: String,
    }

    #[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct UserCombinedInfoDTO {
        pub user_id: u64,
        pub username: String,
        pub user_info: UserInfoDTO,
        pub personal_info_list: Vec<PersonalInfoDTO>,
    }

    #[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct DbUserDTO {
        pub id: i32,
        pub username: String,
        pub hashed_password: Vec<u8>,
        pub hashed_payment_password: Option<Vec<u8>>,
        pub salt: Vec<u8>,
        pub wrong_payment_password_tried: i32,
        pub gender: Option<String>,
        pub age: Option<i32>,
        pub phone: String,
        pub email: Option<String>,
        pub name: String,
        pub identity_card_id: String,
    }

    #[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct DbPersonalInfo {
        pub id: i32,
        pub uuid: Uuid,
        pub name: String,
        pub identity_card: String,
        pub preferred_seat_location: Option<String>,
        pub user_id: i32,
        pub is_default: bool,
    }
    ```

    `From<Domain Entity>`应在相应微服务中实现，例如，对于`User`微服务，在`user/base/src/application/service/internal.rs`中定义`UserInternalService`，并实现`From<Domain Entity>`：

    ```rust
    use crate::domain::model::personal_info::PersonalInfo;
    use crate::domain::model::session::Session;
    use crate::domain::model::user::UserInfo;
    use async_trait::async_trait;
    use shared::domain::Identifiable;
    use shared::internal::user::command::{
        ClearWrongPaymentPasswordTriedCommand, SessionQuery, SetPaymentPasswordCommand, UserInfoQuery,
        VerifyPasswordCommand, VerifyPaymentPasswordCommand,
    };
    use shared::internal::user::dto::{DbPersonalInfo, DbUserDTO, PersonalInfoDTO, UserInfoDTO};
    use thiserror::Error;

    #[derive(Error, Debug)]
    pub enum UserInternalServiceError {
        #[error(transparent)]
        RelatedServiceError(#[from] anyhow::Error),
    }

    impl From<PersonalInfo> for PersonalInfoDTO {
        fn from(value: PersonalInfo) -> Self {
            PersonalInfoDTO {
                id: Some(
                    value
                        .get_id()
                        .expect("saved personal info should have id")
                        .into(),
                ),
                uuid: value.uuid(),
                name: value.name().to_string(),
                identity_card_id: value.identity_card_id().to_string(),
                preferred_seat_location: value.preferred_seat_location().map(|x| x.into()),
                user_id: (*value.user_id()).into(),
                is_default: value.is_default(),
            }
        }
    }

    impl From<UserInfo> for UserInfoDTO {
        fn from(value: UserInfo) -> Self {
            UserInfoDTO {
                name: value.name.to_string(),
                gender: value.gender.map(|x| x.to_string()),
                age: value.age.map(|x| x.into()),
                phone: value.phone.to_string(),
                email: value.email.map(|x| x.to_string()),
                identity_card_id: value.identity_card_id.to_string(),
            }
        }
    }

    impl From<crate::models::user::Model> for DbUserDTO {
        fn from(value: crate::models::user::Model) -> Self {
            DbUserDTO {
                id: value.id,
                username: value.username,
                hashed_password: value.hashed_password,
                hashed_payment_password: value.hashed_payment_password,
                salt: value.salt,
                wrong_payment_password_tried: value.wrong_payment_password_tried,
                gender: value.gender,
                age: value.age,
                phone: value.phone,
                email: value.email,
                name: value.name,
                identity_card_id: value.identity_card_id,
            }
        }
    }

    impl From<crate::models::person_info::Model> for DbPersonalInfo {
        fn from(value: crate::models::person_info::Model) -> Self {
            DbPersonalInfo {
                id: value.id,
                uuid: value.uuid,
                name: value.name,
                identity_card: value.identity_card,
                preferred_seat_location: value.preferred_seat_location,
                user_id: value.user_id,
                is_default: value.is_default,
            }
        }
    }

    #[async_trait]
    pub trait UserInternalService: 'static + Send + Sync {
        async fn verify_password(
            &self,
            command: VerifyPasswordCommand,
        ) -> Result<bool, UserInternalServiceError>;

        async fn verify_payment_password(
            &self,
            command: VerifyPaymentPasswordCommand,
        ) -> Result<bool, UserInternalServiceError>;

        async fn set_payment_password(
            &self,
            command: SetPaymentPasswordCommand,
        ) -> Result<(), UserInternalServiceError>;

        async fn clear_wrong_payment_password_tried(
            &self,
            command: ClearWrongPaymentPasswordTriedCommand,
        ) -> Result<(), UserInternalServiceError>;

        async fn get_session(
            &self,
            query: SessionQuery,
        ) -> Result<Option<Session>, UserInternalServiceError>;

        async fn get_user_info(
            &self,
            query: UserInfoQuery,
        ) -> Result<Option<UserInfo>, UserInternalServiceError>;

        async fn db_get_user_info(&self) -> Result<Vec<DbUserDTO>, UserInternalServiceError>;

        async fn db_get_personal_info(&self) -> Result<Vec<DbPersonalInfo>, UserInternalServiceError>;
    }
    ```

4. 实现Internal Service，对于一般方法（不以`db_`开头），直接转发到Domain Service中的同名方法（在某些情况下，你可能需要从数据库中加载相应的实体）；对于`db_`开头的方法，转换为对Repository的调用。在本步骤中，你可能需要修改之前的错误类型（例如，`UserInternalServiceError`），添加适用的更多变体。不要轻易使用`unwrap`等会导致`panic`的方法处理错误。你可能需要注意，所有`*Repository`都实现了`Repository`特征，你可能需要手动将DTO中的`u64`转换为对应的`*Id`，请为每个实现的Internal Service方法添加`#[instrument(skip(self))]`。在实现内部，对于除了API使用方式导致的错误（例如，要求年龄大于0，但传入了小于0的数值；要求支付密码为6位数字，但传入了其它内容），其余错误请在返回前使用`inspect_err` + `tracing::error`打印信息（`{:?}`），对于在多个函数中重复使用的代码，请搜集到单独的函数中，不要多次重复：

    ```rust
    use anyhow::Context;
    use async_trait::async_trait;
    use std::any::{Any, TypeId};
    use std::collections::HashMap;
    use std::fmt::Debug;
    use std::hash::Hash;
    use std::sync::{Arc, Mutex};
    use thiserror::Error;
    use tracing::{debug, error, instrument};

    /// 标识符特征，用于表示可作为实体ID的类型
    ///
    /// # 要求
    /// - 必须是`'static`的（但并不意味着需要在整个程序生命周期都存在）
    /// - 可发送到不同线程（`Send`）
    /// - 可复制（`Copy` + `Clone`）
    /// - 可比较相等性（`PartialEq` + `Eq`）
    /// - 可哈希（`Hash`）
    /// - 可调试打印（`Debug`）
    ///
    /// # 关于'static的说明
    /// 这里的`'static`约束表示我们可以持有该标识符任意长的时间，
    /// 而并非要求它必须在整个程序运行期间都存在。
    /// 详见：[Common Rust Lifetime Misconceptions](https://github.com/pretzelhammer/rust-blog/blob/master/posts/common-rust-lifetime-misconceptions.md)
    ///
    /// # Examples
    /// ```
    /// use base::domain::Identifier;
    /// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    /// struct UserId(u64);
    ///
    /// impl Identifier for UserId {}
    /// ```
    pub trait Identifier: Debug + 'static + Send + Copy + Clone + PartialEq + Eq + Hash {}

    /// 可标识特征，表示具有唯一标识符的类型
    ///
    /// # 关联类型
    /// - `ID`: 实现`Identifier`特征的标识符类型
    ///
    /// # 方法
    /// - `get_id`: 获取当前对象的标识符（可能为None）
    ///
    /// # Examples
    /// ```
    /// use base::domain::model::user::UserId;
    /// use base::domain::Identifiable;
    ///
    /// #[derive(Debug, Clone)]
    /// struct User {
    ///     id: UserId,
    ///     name: String,
    /// }
    ///
    /// impl Identifiable for User {
    ///     type ID = UserId;
    ///     fn get_id(&self) -> Option<Self::ID> {
    ///         Some(self.id)
    ///     }
    ///     fn set_id(&mut self, id: Self::ID) {
    ///        self.id = id;
    ///    }
    /// }
    /// ```
    pub trait Identifiable {
        type ID: Identifier;
        fn get_id(&self) -> Option<Self::ID>;

        fn set_id(&mut self, id: Self::ID);
    }

    /// 实体特征，表示领域模型中的基本实体
    ///
    /// # 要求
    /// - 必须是`'static`的（用于支持`Any`特征）
    /// - 可发送到不同线程（`Send`）
    /// - 可克隆（`Clone`）
    /// - 可调试打印（`Debug`）
    /// - 具有唯一标识符（`Identifiable`）
    ///
    /// # 注意
    /// 在Repository实现中，我们通过Snapshot追踪实体状态变更，
    /// 这需要实体实现`Any`特征，而`Any`特征只为`'static`的类型实现。
    ///
    /// # Examples
    /// ```
    /// # use base::domain::model::user::UserId;
    /// # use base::domain::{Identifiable, Entity};
    /// #[derive(Debug, Clone)]
    /// struct User {
    ///     id: UserId,
    ///     name: String,
    /// }
    ///
    /// # impl Identifiable for User {
    /// #    type ID = UserId;
    /// #
    /// #    fn get_id(&self) -> Option<Self::ID> {
    /// #       todo!()
    /// #     }
    /// #     fn set_id(&mut self, id: Self::ID) {
    /// #        todo!()
    /// #    }
    /// # }
    /// impl Entity for User {}
    /// ```
    pub trait Entity: Debug + Identifiable + 'static + Send + Clone {}

    /// 聚合根特征，表示领域模型中的聚合根
    ///
    /// 聚合根是实体的一种特殊形式，作为聚合的入口点，
    /// 负责维护聚合内的不变条件和一致性边界。
    ///
    /// # 要求
    /// 继承所有`Entity`特征的约束
    ///
    /// # Examples
    /// ```
    /// # use base::domain::{Aggregate, Entity, Identifiable, Identifier};
    /// # #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    /// # pub struct OrderId(u64);
    /// # #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    /// # pub struct OrderItemId(u64);
    /// # impl Identifier for OrderId {}
    /// # impl Identifier for OrderItemId {}
    /// # #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    /// # pub struct OrderItem {
    /// #    item_id: OrderItemId,
    /// # }
    /// # impl Identifiable for OrderItem {
    /// #    type ID = OrderItemId;
    /// #
    /// #    fn get_id(&self) -> Option<Self::ID> {
    /// #        todo!()
    /// #   }
    /// #    fn set_id(&mut self, id: Self::ID) {
    /// #        todo!()
    /// #   }
    /// # }
    /// #[derive(Debug, Clone)]
    /// struct Order {
    ///     id: OrderId,
    ///     items: Vec<OrderItem>,
    /// }
    /// # impl Identifiable for Order {
    /// #     type ID = OrderId;
    /// #
    /// #    fn get_id(&self) -> Option<Self::ID> {
    /// #       todo!()
    /// #   }
    /// #     fn set_id(&mut self, id: Self::ID) {
    /// #        todo!()
    /// #    }
    /// # }
    /// # impl Entity for OrderItem {}
    /// # impl Entity for Order {}
    /// impl Aggregate for Order {}
    /// ```
    pub trait Aggregate: Entity {}

    /// 表示变更类型的枚举
    ///
    /// # 变体
    /// - `Added`: 表示实体被添加
    /// - `Removed`: 表示实体被移除
    /// - `Modified`: 表示实体被修改
    /// - `Unchanged`: 表示实体未发生变化
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum DiffType {
        Added,
        Removed,
        Modified,
        Unchanged,
    }

    impl DiffType {
        pub fn from_with_compare_fn<T>(
            old: Option<&T>,
            new: Option<&T>,
            compare_fn: fn(&T, &T) -> bool,
        ) -> Self
        where
            T: Entity + PartialEq + Eq,
        {
            match (old, new) {
                (None, None) => DiffType::Unchanged,
                (None, Some(_)) => DiffType::Added,
                (Some(_), None) => DiffType::Removed,
                (Some(old_value), Some(new_value)) => {
                    if compare_fn(old_value, new_value) {
                        DiffType::Unchanged
                    } else {
                        DiffType::Modified
                    }
                }
            }
        }
    }

    impl<T> From<&DiffInfo<T>> for DiffType
    where
        T: Aggregate + PartialEq + Eq,
    {
        fn from(value: &DiffInfo<T>) -> Self {
            let old = &value.old;
            let new = &value.new;

            match (old, new) {
                (None, None) => DiffType::Unchanged,
                (None, Some(_)) => DiffType::Added,
                (Some(_), None) => DiffType::Removed,
                (Some(old_value), Some(new_value)) => {
                    if old_value == new_value {
                        DiffType::Unchanged
                    } else {
                        DiffType::Modified
                    }
                }
            }
        }
    }

    /// 定义变更检测的基本特征
    ///
    /// # 方法
    /// - `diff_type`: 获取变更类型
    /// - `is_empty`: 检查变更是否为空（即是否为`Unchanged`状态）
    pub trait Diff {
        fn diff_type(&self) -> DiffType;

        fn is_empty(&self) -> bool;
    }

    /// 类型化的实体变更记录
    ///
    /// 用于存储实体变更前后的状态，并记录变更类型
    ///
    /// # 类型参数
    /// - `T`: 实现`Entity`特征的实体类型
    ///
    /// # 字段
    /// - `diff_type`: 变更类型
    /// - `old_value`: 变更前的值（None表示新增）
    /// - `new_value`: 变更后的值（None表示删除）
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct TypedDiff<T> {
        pub diff_type: DiffType,
        pub old_value: Option<T>,
        pub new_value: Option<T>,
    }

    impl<T> TypedDiff<T> {
        /// 创建新的类型化变更记录
        ///
        /// # 参数
        /// - `diff_type`: 变更类型
        /// - `old`: 变更前的实体状态
        /// - `new`: 变更后的实体状态
        pub fn new(diff_type: DiffType, old: Option<T>, new: Option<T>) -> Self {
            TypedDiff {
                diff_type,
                old_value: old,
                new_value: new,
            }
        }
    }

    impl<T> Diff for TypedDiff<T>
    where
        T: 'static + Send + Sync,
    {
        fn diff_type(&self) -> DiffType {
            self.diff_type
        }

        fn is_empty(&self) -> bool {
            self.diff_type == DiffType::Unchanged
        }
    }

    /// 任意类型变更的trait对象特征
    ///
    /// 主要用于实现类型擦除，允许将不同类型变更存储在同一个集合中
    trait AnyDiff: Send {
        /// 将自身转换为`Any` trait对象，以支持向下转型
        fn as_any(&self) -> &dyn Any;
    }

    impl<T> AnyDiff for TypedDiff<T>
    where
        T: 'static + Send + Sync,
    {
        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    /// 多实体变更集合
    ///
    /// 使用`TypeId`作为键存储不同类型的实体变更
    ///
    /// # Examples
    /// ```
    /// # use base::domain::{MultiEntityDiff, TypedDiff, DiffType, Identifier, Identifiable, Entity};
    /// #
    /// # #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    /// # pub struct UserId(u64);
    /// #
    /// # impl Identifier for UserId {}
    /// #
    /// # #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    /// # pub struct User {
    /// #     user_id: UserId
    /// # }
    /// #
    /// # impl Identifiable for User {
    /// #     type ID = UserId;
    /// #
    /// #     fn get_id(&self) -> Option<Self::ID> {
    /// #        todo!()
    /// #    }
    /// #     fn set_id(&mut self, id: Self::ID) {
    /// #       todo!()
    /// #   }
    /// # }
    /// #
    /// # impl User {
    /// #     pub fn new() -> Self {
    /// #         User {
    /// #             user_id: UserId(0),
    /// #         }
    /// #     }
    /// # }
    /// #
    /// # impl Entity for User {}
    /// #
    /// let user = User::new();
    /// let mut multi_diff = MultiEntityDiff::new();
    /// multi_diff.add_change(TypedDiff::new(DiffType::Added, None, Some(user)));
    /// let changes = multi_diff.get_changes::<User>();
    /// ```
    #[derive(Default)]
    pub struct MultiEntityDiff {
        changes: HashMap<TypeId, Vec<Box<dyn AnyDiff>>>,
    }

    impl MultiEntityDiff {
        /// 创建新的多实体变更集合
        pub fn new() -> Self {
            MultiEntityDiff {
                changes: HashMap::new(),
            }
        }

        /// 添加实体变更记录
        ///
        /// # 类型参数
        /// - `T`: 实现`Entity`特征的实体类型
        ///
        /// # 参数
        /// - `diff`: 类型化变更记录
        pub fn add_change<T>(&mut self, diff: TypedDiff<T>)
        where
            T: 'static + Send + Sync,
        {
            self.changes
                .entry(TypeId::of::<TypedDiff<T>>())
                .or_default()
                .push(Box::new(diff))
        }

        // 获取指定类型的实体变更记录
        ///
        /// # 类型参数
        /// - `T`: 实现`Entity`特征的实体类型
        ///
        /// # 返回值
        /// 返回该类型的所有变更记录的`Vec`
        pub fn get_changes<T>(&self) -> Vec<TypedDiff<T>>
        where
            T: Clone + 'static,
        {
            self.changes
                .get(&TypeId::of::<TypedDiff<T>>())
                .map(|v| {
                    v.iter()
                        .filter_map(|d| d.as_any().downcast_ref::<TypedDiff<T>>())
                        .cloned()
                        .collect()
                })
                .unwrap_or_default()
        }

        /// 检查变更集合是否为空
        pub fn is_empty(&self) -> bool {
            self.changes.is_empty()
        }
    }

    /// 聚合根管理特征
    ///
    /// 定义了对聚合根进行变更检测和状态管理的基本操作
    ///
    /// # 类型参数
    /// - `AG`: 实现`Aggregate`特征的聚合根类型
    pub trait AggregateManager<AG>
    where
        AG: Aggregate,
    {
        /// 附加聚合根到管理器
        fn attach(&mut self, aggregate: AG);
        /// 从管理器分离聚合根
        fn detach(&mut self, aggregate: &AG);
        /// 合并聚合根状态
        fn merge(&mut self, aggregate: AG);
        /// 检测聚合根状态变更
        ///
        /// # 参数
        /// - `aggregate`: 要检测的聚合根
        ///
        /// # 返回值
        /// 返回包含所有变更的`MultiEntityDiff`
        ///
        /// # Notes
        /// 该函数不更新聚合根中的快照，需要手动调用merge
        fn detect_changes(&self, aggregate: AG) -> MultiEntityDiff;
    }

    #[derive(Error, Debug)]
    pub enum RepositoryError {
        #[error("database error: {0}")]
        Db(anyhow::Error),

        #[error("invalid data object from db: {0}")]
        ValidationError(#[from] anyhow::Error),

        #[error("inconsistent database state: {0}")]
        InconsistentState(anyhow::Error),
    }

    /// 仓储接口，定义了对聚合根(AG)的持久化操作
    ///
    /// # 泛型参数
    /// - `AG`: 实现`Aggregate`特征的聚合根类型
    ///
    /// # 方法
    /// - `attach`: 将聚合根附加到仓储聚合根管理器中
    /// - `detach`: 从仓储仓储聚合根管理器中分离聚合根
    /// - `find`: 根据ID查找聚合根
    /// - `remove`: 移除指定的聚合根
    /// - `save`: 保存聚合根（根据ID是否存在自动判断插入或更新）
    #[async_trait]
    pub trait Repository<AG>: 'static + Send + Sync
    where
        AG: Aggregate,
    {
        async fn find(&self, id: AG::ID) -> Result<Option<AG>, RepositoryError>;
        async fn remove(&self, aggregate: AG) -> Result<(), RepositoryError>;
        async fn save(&self, aggregate: &mut AG) -> Result<AG::ID, RepositoryError>;
    }

    pub trait DbId {
        type DbType;

        fn to_db_value(&self) -> Self::DbType;
        fn from_db_value(value: Self::DbType) -> Result<Self, anyhow::Error>
        where
            Self: Sized;
    }

    pub trait SnapshottingRepository<AG>: Repository<AG>
    where
        AG: Aggregate,
    {
        fn attach(&self, aggregate: AG);
        fn detach(&self, aggregate: &AG);
    }

    /// 数据库仓储支持特性，提供与数据库交互的底层操作
    ///
    /// # 泛型参数
    /// - `AG`: 实现`Aggregate`特征的聚合根类型
    ///
    /// # 关联类型
    /// - `Manager`: 实现`AggregateManager<AG>`的聚合管理器类型
    ///
    /// # 方法
    /// - `get_aggregate_manager`: 获取聚合管理器
    /// - `on_insert`: 执行插入操作时的回调
    /// - `on_select`: 执行查询操作时的回调
    /// - `on_update`: 执行更新操作时的回调
    /// - `on_delete`: 执行删除操作时的回调
    ///

    #[async_trait]
    pub trait DbRepositorySupport<AG>
    where
        AG: Aggregate,
    {
        type Manager: AggregateManager<AG>;

        fn get_aggregate_manager(&self) -> Arc<Mutex<Self::Manager>>;
        async fn on_insert(&self, aggregate: AG) -> Result<AG::ID, RepositoryError>;
        async fn on_select(&self, id: AG::ID) -> Result<Option<AG>, RepositoryError>;
        async fn on_update(&self, diff: MultiEntityDiff) -> Result<(), RepositoryError>;
        async fn on_delete(&self, aggregate: AG) -> Result<(), RepositoryError>;
    }

    /// 为实现了`DbRepositorySupport`的类型自动提供`Repository`的默认实现
    ///
    /// 这个实现桥接了仓储接口与数据库具体操作，并提供了：
    /// - 变更追踪
    /// - 自动附加/分离聚合根
    /// - 根据状态自动选择插入或更新
    ///
    /// # 泛型参数
    /// - `AG`: 实现`Aggregate`特征的聚合根类型
    /// - `T`: 实现`DbRepositorySupport<AG>`的类型
    ///
    /// # Errors
    /// - 当数据库操作失败时返回`RepositoryError`
    #[async_trait]
    impl<AG, T> Repository<AG> for T
    where
        AG: Aggregate,
        T: DbRepositorySupport<AG> + 'static + Send + Sync,
    {
        #[instrument(skip(self))]
        async fn find(&self, id: AG::ID) -> Result<Option<AG>, RepositoryError> {
            let entity = self.on_select(id).await?;

            if let Some(ref entity) = entity {
                self.get_aggregate_manager()
                    .lock()
                    .unwrap()
                    .attach(entity.clone())
            }

            Ok(entity)
        }

        #[instrument(skip(self))]
        async fn remove(&self, aggregate: AG) -> Result<(), RepositoryError> {
            self.get_aggregate_manager()
                .lock()
                .unwrap()
                .detach(&aggregate);

            self.on_delete(aggregate).await
        }

        #[instrument(skip(self))]
        async fn save(&self, aggregate: &mut AG) -> Result<AG::ID, RepositoryError> {
            debug!("saving aggregate: {:?} with cache", aggregate);
            if let Some(id) = aggregate.get_id() {
                debug!("aggregate has id, checking difference");

                let diff = self
                    .get_aggregate_manager()
                    .lock()
                    .unwrap()
                    .detect_changes(aggregate.clone());
                if !diff.is_empty() {
                    debug!("found update, calling on_update");
                    self.on_update(diff).await?;
                    self.get_aggregate_manager()
                        .lock()
                        .unwrap()
                        .merge(aggregate.clone());
                } else {
                    debug!("no update found");
                }
                Ok(id)
            } else {
                debug!("aggregate doesn't have id, inserting");
                let id = self.on_insert(aggregate.clone()).await?;

                aggregate.set_id(id);

                self.get_aggregate_manager()
                    .lock()
                    .unwrap()
                    .attach(aggregate.clone());

                Ok(id)
            }
        }
    }

    /// 快照仓储接口，定义了对聚合根(AG)的支持快照的持久化操作
    ///
    /// # 泛型参数
    /// - `AG`: 实现`Aggregate`特征的聚合根类型
    ///
    /// # 方法
    /// - `attach`: 将聚合根附加到仓储聚合根管理器中
    /// - `detach`: 从仓储仓储聚合根管理器中分离聚合根
    impl<AG, T> SnapshottingRepository<AG> for T
    where
        T: DbRepositorySupport<AG> + 'static + Send + Sync,
        AG: Aggregate,
    {
        fn attach(&self, aggregate: AG) {
            self.get_aggregate_manager()
                .lock()
                .unwrap()
                .attach(aggregate);
        }

        fn detach(&self, aggregate: &AG) {
            self.get_aggregate_manager()
                .lock()
                .unwrap()
                .detach(aggregate);
        }
    }

    pub struct DiffInfo<AG>
    where
        AG: Aggregate,
    {
        pub old: Option<AG>,
        pub new: Option<AG>,
    }

    impl<AG> DiffInfo<AG>
    where
        AG: Aggregate,
    {
        pub fn new(old: Option<AG>, new: AG) -> Self {
            DiffInfo {
                old,
                new: Some(new),
            }
        }
    }

    /// 聚合根管理器的默认实现
    ///
    /// 通过内部哈希映射维护聚合根状态，允许通过自定义差异检测函数识别变更
    ///
    /// # 类型参数
    /// - `AG`: 实现 [`Aggregate`] trait 的聚合根类型
    ///
    /// # 架构说明
    /// - 使用 `HashMap<AG::ID, AG>` 跟踪聚合根的最新已知状态
    /// - 通过可注入的 `detect_changes_fn` 实现灵活的状态差异检测策略
    pub struct AggregateManagerImpl<AG>
    where
        AG: Aggregate,
    {
        aggregate_map: HashMap<AG::ID, AG>,
        detect_changes_fn: Box<dyn Fn(DiffInfo<AG>) -> MultiEntityDiff + Sync + Send>,
    }

    impl<AG> AggregateManagerImpl<AG>
    where
        AG: Aggregate,
    {
        /// 创建新的聚合根管理器实例
        ///
        /// # 参数
        /// - `detect_changes_fn`: 状态差异检测回调函数，接收新旧状态并返回差异报告
        ///
        /// # Examples
        /// ```rust
        /// # use base::domain::MultiEntityDiff;
        /// # use std::collections::HashMap;
        /// # use base::domain::{Aggregate, Entity, Identifiable, Identifier};
        /// # use base::domain::service::AggregateManagerImpl;
        /// # #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        /// # struct User(UserId);
        /// # #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        /// # struct UserId(i32);
        /// # impl Identifier for UserId {}
        /// # impl Identifiable for User {
        ///     type ID = UserId;
        ///         fn get_id(&self) -> Option<Self::ID> {
        ///         Some(self.0)
        ///     }
        ///         fn set_id(&mut self, id: Self::ID) {
        ///         self.0 = id;
        ///     }
        /// }
        /// # impl Entity for User {}
        /// # impl Aggregate for User {}
        /// # let changes_fn = Box::new(|_| MultiEntityDiff::new());
        /// let manager = AggregateManagerImpl::<User>::new(changes_fn);
        /// ```
        pub fn new(
            detect_changes_fn: Box<dyn Fn(DiffInfo<AG>) -> MultiEntityDiff + Sync + Send>,
        ) -> Self {
            AggregateManagerImpl {
                aggregate_map: HashMap::new(),
                detect_changes_fn,
            }
        }
    }
    impl<AG> AggregateManager<AG> for AggregateManagerImpl<AG>
    where
        AG: Aggregate,
    {
        /// 附加聚合根到管理器
        ///
        /// 若聚合根已存在有效ID，将覆盖现有记录
        ///
        /// # Notes
        /// - 无ID的聚合根将被静默忽略
        fn attach(&mut self, aggregate: AG) {
            if let Some(id) = aggregate.get_id() {
                self.aggregate_map.insert(id, aggregate);
            }
        }

        /// 从管理器分离聚合根
        ///
        /// # Notes
        /// - 根据聚合根当前ID进行删除，删除后ID变化可能导致残留
        fn detach(&mut self, aggregate: &AG) {
            if let Some(id) = aggregate.get_id() {
                self.aggregate_map.remove(&id);
            }
        }

        /// 合并聚合根状态（当前实现为替换策略）
        ///
        /// # 实现细节
        /// 直接调用 [self.attach] 方法，用新实例完全替换旧状态
        fn merge(&mut self, aggregate: AG) {
            self.attach(aggregate);
        }

        /// 检测给定聚合根的状态变更
        ///
        /// # 流程
        /// 1. 根据聚合根ID查找已注册的旧状态
        /// 2. 通过注入的差异检测函数生成变更报告
        /// 3. 不会自动更新内部状态，需手动调用合并/附加操作
        ///
        fn detect_changes(&self, aggregate: AG) -> MultiEntityDiff {
            let old = aggregate
                .get_id()
                .and_then(|id| self.aggregate_map.get(&id).cloned());

            (self.detect_changes_fn)(DiffInfo::new(old, aggregate))
        }
    }

    #[derive(Error, Debug)]
    pub enum ServiceError {
        #[error("repository error: {0}")]
        RepositoryError(RepositoryError),
        #[error("a related service returned an error: {0}")]
        RelatedServiceError(anyhow::Error),
    }

    impl From<RepositoryError> for ServiceError {
        fn from(value: RepositoryError) -> Self {
            ServiceError::RepositoryError(value)
        }
    }

    #[instrument(level = "trace", skip_all)]
    pub fn transform_list<T, U, I>(
        list: Vec<T>,
        converter: impl Fn(T) -> Result<U, anyhow::Error>,
        get_id: impl Fn(&T) -> I,
    ) -> Result<Vec<U>, anyhow::Error>
    where
        I: std::fmt::Display,
    {
        let mut result_list = Vec::with_capacity(list.len());

        for model in list {
            let id = get_id(&model);
            let city = converter(model)
                .context(format!("Failed to validate entity with id: {}", id))
                .map_err(|e| {
                    error!("Failed to validate entity with id: {}. Error: {}", id, e);
                    e
                })?;
            result_list.push(city);
        }

        Ok(result_list)
    }
    ```

   1. 根据给出的微服务拆分计划中的依赖关系，定义实现结构体以及其依赖的泛型参数。例如，对于`User`微服务，在`user/base/src/infrastructure/service/internal.rs`中，编写如下定义，并实现`new`关联方法：

        ```rust
        use crate::domain::repository::personal_info::PersonalInfoRepository;
        use crate::domain::repository::user::UserRepository;
        use crate::domain::service::session::SessionManagerService;
        use crate::domain::service::user::UserService;
        use std::sync::Arc;

        pub struct UserInternalServiceImpl<US, SMS, UR, PIR>
        where
            US: UserService,
            SMS: SessionManagerService,
            UR: UserRepository,
            PIR: PersonalInfoRepository,
        {
            user_service: Arc<US>,
            session_manager_service: Arc<SMS>,
            user_repository: Arc<UR>,
            personal_info_repository: Arc<PIR>,
        }

        impl<US, SMS, UR, PIR> UserInternalServiceImpl<US, SMS, UR, PIR>
        where
            US: UserService,
            SMS: SessionManagerService,
            UR: UserRepository,
            PIR: PersonalInfoRepository,
        {
            pub fn new(
                user_service: Arc<US>,
                session_manager_service: Arc<SMS>,
                user_repository: Arc<UR>,
                personal_info_repository: Arc<PIR>,
            ) -> Self {
                Self {
                    user_service,
                    session_manager_service,
                    user_repository,
                    personal_info_repository,
                }
            }
        }
        ```

   2. 实现特征上定义的方法，在实现过程中，你可能需要对之前定义的`*InteralServiceError`枚举增加错误类型；在实现`db_`开头的方法时，你可能需要修改对应`Repository`的定义，新增`load_all_raw`等加载所有数据的方法，注意，`load_all_raw`应当直接返回Database Model，不要转换为Domain Entity。

        例如，对于`User`微服务，在实现过程中，对于`UserInternalServiceError`进行了修改：

        ```rust
        #[derive(Error, Debug)]
        pub enum UserInternalServiceError {
            #[error("no such user: id = {0}")]
            NoSuchUser(u64),
            #[error("invalid payment password: {0}")]
            InvalidPaymentPassword(String),
            #[error("invalid session ID: {0}")]
            InvalidSessionId(String),
            #[error(transparent)]
            RelatedServiceError(#[from] anyhow::Error),
        }
        ```

        在`user/base/src/domain/repository/user.rs`的`UserRepository`中添加了`async fn load_all_raw(&self) -> Result<Vec<User>, RepositoryError>;`方法
        在`user/base/src/domain/repository/personal_info.rs`的`PersonalInfoRepository`中添加了`async fn load_all_raw(&self) -> Result<Vec<PersonalInfo>, RepositoryError>;`方法

        在`user/base/src/infrastructure/repository/user.rs`、`user/base/src/infrastructure/repository/personal_info.rs`中实现上述方法，注意`#[instrument(skip(self))]`：

        ```rust
        #[instrument(skip(self))]
        async fn load_all_raw(&self) -> Result<Vec<crate::models::user::Model>, RepositoryError> {
            crate::models::user::Entity::find()
                .all(&self.db)
                .await
                .map_err(|e| RepositoryError::Db(e.into()))
        }
        ```

        ```rust
        #[instrument(skip(self))]
        async fn load_all_raw(
            &self,
        ) -> Result<Vec<crate::models::person_info::Model>, RepositoryError> {
            crate::models::person_info::Entity::find()
                .all(&self.db)
                .await
                .map_err(|e| RepositoryError::Db(e.into()))
        }
        ```

        例如，对`UserInternalService`实现如下：

        ```rust
        use crate::application::service::internal::{UserInternalService, UserInternalServiceError};
        use crate::domain::model::session::SessionId;
        use crate::domain::model::user::{PaymentPassword, User, UserId};
        use crate::domain::repository::personal_info::PersonalInfoRepository;
        use crate::domain::repository::user::UserRepository;
        use crate::domain::service::session::SessionManagerService;
        use crate::domain::service::user::UserService;
        use async_trait::async_trait;
        use shared::domain::Identifiable;
        use shared::internal::user::command::{
            ClearWrongPaymentPasswordTriedCommand, SessionQuery, SetPaymentPasswordCommand, UserInfoQuery,
            VerifyPasswordCommand, VerifyPaymentPasswordCommand,
        };
        use shared::internal::user::dto::PersonalInfoDTO;
        use shared::internal::user::dto::{
            DbPersonalInfo, DbUserDTO, SessionDTO, UserCombinedInfoDTO, UserInfoDTO,
        };
        use std::sync::Arc;
        use tracing::{error, instrument};
        use uuid::Uuid;

        pub struct UserInternalServiceImpl<US, SMS, UR, PIR>
        where
            US: UserService,
            SMS: SessionManagerService,
            UR: UserRepository,
            PIR: PersonalInfoRepository,
        {
            user_service: Arc<US>,
            session_manager_service: Arc<SMS>,
            user_repository: Arc<UR>,
            personal_info_repository: Arc<PIR>,
        }

        impl<US, SMS, UR, PIR> UserInternalServiceImpl<US, SMS, UR, PIR>
        where
            US: UserService,
            SMS: SessionManagerService,
            UR: UserRepository,
            PIR: PersonalInfoRepository,
        {
            pub fn new(
                user_service: Arc<US>,
                session_manager_service: Arc<SMS>,
                user_repository: Arc<UR>,
                personal_info_repository: Arc<PIR>,
            ) -> Self {
                Self {
                    user_service,
                    session_manager_service,
                    user_repository,
                    personal_info_repository,
                }
            }

            async fn load_user_from_query(
                &self,
                user_id: u64,
            ) -> Result<Option<User>, UserInternalServiceError> {
                let user_id = UserId::from(user_id);

                let user = self
                    .user_repository
                    .find(user_id)
                    .await
                    .inspect_err(|e| error!("error loading user from db: {:?}", e))
                    .map_err(|e| UserInternalServiceError::RelatedServiceError(e.into()))?;

                Ok(user)
            }
        }

        #[async_trait]
        impl<US, SMS, UR, PIR> UserInternalService for UserInternalServiceImpl<US, SMS, UR, PIR>
        where
            US: UserService,
            SMS: SessionManagerService,
            UR: UserRepository,
            PIR: PersonalInfoRepository,
        {
            #[instrument(skip(self))]
            async fn verify_password(
                &self,
                command: VerifyPasswordCommand,
            ) -> Result<bool, UserInternalServiceError> {
                let user = self.load_user_from_query(command.user_id).await?;

                if let Some(user) = user {
                    Ok(self
                        .user_service
                        .verify_password(&user, command.raw_password)
                        .await
                        .is_ok())
                } else {
                    Err(UserInternalServiceError::NoSuchUser(command.user_id))
                }
            }

            async fn verify_payment_password(
                &self,
                command: VerifyPaymentPasswordCommand,
            ) -> Result<bool, UserInternalServiceError> {
                let user = self
                    .load_user_from_query(command.user_id)
                    .await?
                    .ok_or(UserInternalServiceError::NoSuchUser(command.user_id))?;

                Ok(self
                    .user_service
                    .verify_payment_password(&user, command.raw_payment_password)
                    .await
                    .is_ok())
            }

            async fn set_payment_password(
                &self,
                command: SetPaymentPasswordCommand,
            ) -> Result<(), UserInternalServiceError> {
                let user_id = UserId::from(command.user_id);

                let payment_password_opt = if let Some(raw_payment_password) = command.raw_payment_password
                {
                    Some(
                        PaymentPassword::try_from(raw_payment_password.as_str()).map_err(
                            |_for_super_earth| {
                                UserInternalServiceError::InvalidPaymentPassword(raw_payment_password)
                            },
                        )?,
                    )
                } else {
                    None
                };

                self.user_service
                    .set_payment_password(user_id, payment_password_opt)
                    .await
                    .inspect_err(|e| error!("error set payment password: {:?}", e))
                    .map_err(|e| UserInternalServiceError::RelatedServiceError(e.into()))
            }

            async fn clear_wrong_payment_password_tried(
                &self,
                command: ClearWrongPaymentPasswordTriedCommand,
            ) -> Result<(), UserInternalServiceError> {
                let user_id = UserId::from(command.user_id);

                self.user_service
                    .clear_wrong_payment_password_tried(user_id)
                    .await
                    .inspect_err(|e| error!("error clear wrong payment password: {:?}", e))
                    .map_err(|e| UserInternalServiceError::RelatedServiceError(e.into()))
            }

            async fn get_session(
                &self,
                query: SessionQuery,
            ) -> Result<Option<SessionDTO>, UserInternalServiceError> {
                let session_id = SessionId::from(Uuid::try_from(query.session_id.as_str()).map_err(
                    |_for_super_earth| UserInternalServiceError::InvalidSessionId(query.session_id),
                )?);

                Ok(self
                    .session_manager_service
                    .get_session(session_id)
                    .await
                    .map_err(|e| UserInternalServiceError::RelatedServiceError(e.into()))?
                    .map(|s| s.into()))
            }

            async fn get_user_info(
                &self,
                query: UserInfoQuery,
            ) -> Result<Option<UserCombinedInfoDTO>, UserInternalServiceError> {
                let user = self.load_user_from_query(query.user_id).await?;

                if let Some(user) = user {
                    let user_info_dto: UserInfoDTO = user.user_info().clone().into();

                    let personal_info_list = self
                        .personal_info_repository
                        .find_by_user_id(user.get_id().unwrap())
                        .await
                        .map_err(|e| UserInternalServiceError::RelatedServiceError(e.into()))?;

                    let personal_info_dto_list: Vec<PersonalInfoDTO> = personal_info_list
                        .into_iter()
                        .map(|personal_info| personal_info.into())
                        .collect();

                    Ok(Some(UserCombinedInfoDTO {
                        user_id: user.get_id().unwrap().into(),
                        username: user.username().to_string(),
                        user_info: user_info_dto,
                        personal_info_list: personal_info_dto_list,
                    }))
                } else {
                    Ok(None)
                }
            }

            async fn db_get_user_info(&self) -> Result<Vec<DbUserDTO>, UserInternalServiceError> {
                let user_entity_list = self
                    .user_repository
                    .load_all_raw()
                    .await
                    .map_err(|e| UserInternalServiceError::RelatedServiceError(e.into()))?;

                Ok(user_entity_list.into_iter().map(|u| u.into()).collect())
            }

            async fn db_get_personal_info(&self) -> Result<Vec<DbPersonalInfo>, UserInternalServiceError> {
                let personal_info_entity_list = self
                    .personal_info_repository
                    .load_all_raw()
                    .await
                    .map_err(|e| UserInternalServiceError::RelatedServiceError(e.into()))?;

                Ok(personal_info_entity_list
                    .into_iter()
                    .map(|p| p.into())
                    .collect())
            }
        }
        ```

   3. 实现完成后，请执行`cargo check`、`cargo clippy`，修复其中的警告。禁止通过`#[allow]`消除警告，若你认为警告是误报、不太可能消除，请询问我意见，不要擅自行动。
5. 实现调用Internal Service的API
   1. 为`*InternalServiceError`实现`Application Error`特征，错误代码可从微服务划分中获取，例如，`User(Err: 90XXX)`表示本微服务可用的错误码为`90000 -- 90999`，建议按`*InternalServiceError`枚举中变体出现的顺序，从`XX000`开始编号。
    例如，对于`User`微服务，在`user/base/src/application/service/internal.rs`中添加：

        ```rust
        #[derive(Error, Debug)]
        pub enum UserInternalServiceError {
            #[error("no such user: id = {0}")]
            NoSuchUser(u64),
            #[error("invalid payment password: {0}")]
            InvalidPaymentPassword(String),
            #[error("invalid session ID: {0}")]
            InvalidSessionId(String),
            #[error(transparent)]
            RelatedServiceError(#[from] anyhow::Error),
        }

        impl ApplicationError for UserInternalServiceError {
            fn error_code(&self) -> u32 {
                match self {
                    Self::NoSuchUser(_) => 900000,
                    Self::InvalidPaymentPassword(_) => 90001,
                    Self::InvalidSessionId(_) => 90002,
                    Self::RelatedServiceError(_) => 90003,
                }
            }

            fn error_message(&self) -> String {
                self.to_string()
            }
        }
        ```

   2. 按照微服务划分，编写`*InternalService`的API端点，端点路径、处理函数名与微服务划分中的函数名相同，若有至少一个参数，使用POST请求，否则使用GET请求。例如，对于`User`微服务，在`user/api/src/internal.rs`中新增如下API端点，新增`scoped_config`函数注册API端点：

        ```rust
        use actix_web::web::Bytes;
        use actix_web::{get, post, web};
        use shared::api::{ApiResponse, ApplicationErrorBox, parse_request_body};
        use shared::application_error::ApplicationError;
        use shared::internal::user::command::{
            ClearWrongPaymentPasswordTriedCommand, SessionQuery, SetPaymentPasswordCommand, UserInfoQuery,
            VerifyPasswordCommand, VerifyPaymentPasswordCommand,
        };
        use shared::internal::user::dto::{DbPersonalInfo, DbUserDTO, SessionDTO, UserCombinedInfoDTO};
        use user_base::application::service::internal::UserInternalService;

        #[post("/verify_password")]
        pub async fn verify_password(
            body: Bytes,
            user_internal_service: web::Data<dyn UserInternalService>,
        ) -> Result<ApiResponse<bool>, ApplicationErrorBox> {
            let command: VerifyPasswordCommand = parse_request_body(body)?;

            let result = user_internal_service
                .verify_password(command)
                .await
                .map_err(|e| Box::new(e) as Box<dyn ApplicationError>)?;

            ApiResponse::ok(result)
        }

        #[post("/verify_payment_password")]
        pub async fn verify_payment_password(
            body: Bytes,
            user_internal_service: web::Data<dyn UserInternalService>,
        ) -> Result<ApiResponse<bool>, ApplicationErrorBox> {
            let command: VerifyPaymentPasswordCommand = parse_request_body(body)?;

            let result = user_internal_service
                .verify_payment_password(command)
                .await
                .map_err(|e| Box::new(e) as Box<dyn ApplicationError>)?;

            ApiResponse::ok(result)
        }

        #[post("/set_payment_password")]
        pub async fn set_payment_password(
            body: Bytes,
            user_internal_service: web::Data<dyn UserInternalService>,
        ) -> Result<ApiResponse<()>, ApplicationErrorBox> {
            let command: SetPaymentPasswordCommand = parse_request_body(body)?;

            user_internal_service
                .set_payment_password(command)
                .await
                .map_err(|e| Box::new(e) as Box<dyn ApplicationError>)?;

            ApiResponse::ok(())
        }

        #[post("/clear_wrong_payment_password_tried")]
        pub async fn clear_wrong_payment_password_tried(
            body: Bytes,
            user_internal_service: web::Data<dyn UserInternalService>,
        ) -> Result<ApiResponse<()>, ApplicationErrorBox> {
            let command: ClearWrongPaymentPasswordTriedCommand = parse_request_body(body)?;

            user_internal_service
                .clear_wrong_payment_password_tried(command)
                .await
                .map_err(|e| Box::new(e) as Box<dyn ApplicationError>)?;

            ApiResponse::ok(())
        }

        #[post("/get_session")]
        pub async fn get_session(
            body: Bytes,
            user_internal_service: web::Data<dyn UserInternalService>,
        ) -> Result<ApiResponse<Option<SessionDTO>>, ApplicationErrorBox> {
            let query: SessionQuery = parse_request_body(body)?;

            let result = user_internal_service
                .get_session(query)
                .await
                .map_err(|e| Box::new(e) as Box<dyn ApplicationError>)?;

            ApiResponse::ok(result)
        }

        #[post("/get_user_info")]
        pub async fn get_user_info(
            body: Bytes,
            user_internal_service: web::Data<dyn UserInternalService>,
        ) -> Result<ApiResponse<Option<UserCombinedInfoDTO>>, ApplicationErrorBox> {
            let query: UserInfoQuery = parse_request_body(body)?;

            let result = user_internal_service
                .get_user_info(query)
                .await
                .map_err(|e| Box::new(e) as Box<dyn ApplicationError>)?;

            ApiResponse::ok(result)
        }

        #[get("/db_get_user_info")]
        pub async fn db_get_user_info(
            user_internal_service: web::Data<dyn UserInternalService>,
        ) -> Result<ApiResponse<Vec<DbUserDTO>>, ApplicationErrorBox> {
            let result = user_internal_service
                .db_get_user_info()
                .await
                .map_err(|e| Box::new(e) as Box<dyn ApplicationError>)?;

            ApiResponse::ok(result)
        }

        #[get("/db_get_personal_info")]
        pub async fn db_get_personal_info(
            user_internal_service: web::Data<dyn UserInternalService>,
        ) -> Result<ApiResponse<Vec<DbPersonalInfo>>, ApplicationErrorBox> {
            let result = user_internal_service
                .db_get_personal_info()
                .await
                .map_err(|e| Box::new(e) as Box<dyn ApplicationError>)?;

            ApiResponse::ok(result)
        }

        pub fn scoped_config(cfg: &mut web::ServiceConfig) {
            cfg.service(verify_password)
                .service(verify_payment_password)
                .service(set_payment_password)
                .service(clear_wrong_payment_password_tried)
                .service(get_session)
                .service(get_user_info)
                .service(db_get_user_info)
                .service(db_get_personal_info);
        }
        ```

   3. 在`main.rs`中，例如，对于`User`微服务，`user/api/src/main.rs`中，新增`*InternalService`的构建，并为内部服务创建单独的HTTP服务器，绑定到23333端口：

        ```rust
        let user_internal_service_impl = Arc::new(UserInternalServiceImpl::new(
            Arc::clone(&user_service_impl),
            Arc::clone(&session_manager_service_impl),
            Arc::clone(&user_repository_impl),
            Arc::clone(&personal_info_repository_impl),
        ));

        let user_internal_service: web::Data<dyn UserInternalService> =
        web::Data::from(user_internal_service_impl as Arc<dyn UserInternalService>);

        // 新增
        tokio::task::spawn(async move {
        HttpServer::new(move || {
            App::new()
                .app_data(user_internal_service.clone())
                .app_data(web::PayloadConfig::default().limit(MAX_BODY_LENGTH))
                .wrap(TracingLogger::default())
                .service(web::scope("/internal").configure(user_api::internal::scoped_config))
        })
        .bind(("0.0.0.0", 23333))
        .unwrap()
        .run()
        .await
        .unwrap();
        });

        // 原有对外API服务
        HttpServer::new(move || {
            App::new()
                .app_data(session_manager_service.clone())
                .app_data(user_repository.clone())
                .app_data(user_service.clone())
                .app_data(user_manager_service.clone())
                .app_data(user_profile_service.clone())
                .app_data(personal_info_service.clone())
                .app_data(conn.clone())
                .app_data(web::PayloadConfig::default().limit(MAX_BODY_LENGTH))
                .wrap(TracingLogger::default())
                .service(
                    web::scope("/api")
                        .service(web::scope("/user").configure(user_api::user::scoped_config)),
                )
        })
        .bind(("0.0.0.0", 8080))?
        .run()
        .await?;
        ```

   4. 实现完成后，请执行`cargo check`、`cargo clippy`，修复其中的警告。禁止通过`#[allow]`消除警告，若你认为警告是误报、不太可能消除，请询问我意见，不要擅自行动。
