//! 领域驱动设计(DDD)的核心领域层实现
//!
//! 本模块提供了领域驱动设计中领域层的基础结构和特征定义，包括：
//! - 实体(Entity)和聚合根(Aggregate)的核心特征
//! - 变更追踪和状态管理
//! - 仓储(Repository)抽象接口
//! - 领域服务的基础设施
//!
//! ## 模块结构
//! - `model`: 包含领域模型定义
//! - `repository`: 定义仓储接口和实现
//! - `service`: 领域服务相关实现
//!
//! ## 核心概念
//!
//! ### 实体(Entity)
//! - 具有唯一标识的对象
//! - 通过`Entity`和`Identifiable`特征定义
//! - 生命周期内可追踪状态变化
//!
//! ### 聚合根(Aggregate)
//! - 作为聚合的入口点和一致性边界
//! - 通过`Aggregate`特征定义
//! - 提供变更检测和状态管理功能
//!
//! ### 仓储(Repository)
//! - 提供聚合根的持久化抽象
//! - 支持基本的CRUD操作
//! - 提供自动变更追踪功能
//!
//! ## 主要特征
//! - `Identifier`: 标识符类型的特征约束
//! - `Identifiable`: 可标识对象的特征
//! - `Entity`: 实体基础特征
//! - `Aggregate`: 聚合根特征
//! - `Repository`: 仓储接口
//! - `SnapshottingRepository`： 支持快照的仓储接口
//! - `DbRepositorySupport`: 数据库仓储支持特性
//!
//! ## Examples
//! ```
//! use base::domain::{Aggregate, Entity, Identifiable, Identifier};
//!
//! // 定义标识符类型
//! #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
//! struct UserId(u64);
//! impl Identifier for UserId {}
//!
//! // 定义领域实体
//! #[derive(Debug, Clone)]
//! struct User {
//!     id: UserId,
//!     name: String,
//! }
//!
//! impl Identifiable for User {
//!     type ID = UserId;
//!     fn get_id(&self) -> Option<Self::ID> {
//!         Some(self.id)
//!     }
//!     fn set_id(&mut self, id: Self::ID) {
//!        self.id = id;
//!    }
//! }
//!
//! impl Entity for User {}
//! impl Aggregate for User {}
//! ```
//!
//! ## 设计原则
//! 1. 明确的领域边界
//! 2. 丰富的领域模型
//! 3. 与技术实现解耦
//! 4. 通过特征明确约束
//!
//! ## 注意事项
//! - 实体和聚合根需要实现`'static`生命周期约束
//! - 变更检测依赖于`TypeId`机制
//! - 线程安全通过`Send`和`Sync`特征保证
//!
//! ## 相关模块
//! - `application`: 应用层服务
//! - `infrastructure`: 基础设施实现
//! - `models`: 数据库Data Object定义

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::{Arc, Mutex};
use thiserror::Error;

pub mod model;
pub mod repository;
pub mod service;

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
pub struct TypedDiff<T>
where
    T: Entity,
{
    pub diff_type: DiffType,
    pub old_value: Option<T>,
    pub new_value: Option<T>,
}

impl<T> TypedDiff<T>
where
    T: Entity,
{
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
    T: Entity,
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
    T: Entity,
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
        T: Entity,
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
        T: Entity,
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

    #[error("invalid data object from db")]
    ValidationError(#[from] anyhow::Error),
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
pub trait Repository<AG>
where
    AG: Aggregate,
{
    fn find(&self, id: AG::ID) -> impl Future<Output = Result<Option<AG>, RepositoryError>> + Send;
    fn remove(&self, aggregate: AG) -> impl Future<Output = Result<(), RepositoryError>> + Send;
    fn save(&self, aggregate: AG) -> impl Future<Output = Result<AG::ID, RepositoryError>> + Send;
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
pub trait DbRepositorySupport<AG>
where
    AG: Aggregate,
{
    type Manager: AggregateManager<AG>;

    fn get_aggregate_manager(&self) -> Arc<Mutex<Self::Manager>>;
    fn on_insert(
        &self,
        aggregate: AG,
    ) -> impl Future<Output = Result<AG::ID, RepositoryError>> + Send;
    fn on_select(
        &self,
        id: AG::ID,
    ) -> impl Future<Output = Result<Option<AG>, RepositoryError>> + Send;
    fn on_update(
        &self,
        diff: MultiEntityDiff,
    ) -> impl Future<Output = Result<(), RepositoryError>> + Send;
    fn on_delete(&self, aggregate: AG) -> impl Future<Output = Result<(), RepositoryError>> + Send;
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
impl<AG, T> Repository<AG> for T
where
    AG: Aggregate,
    T: DbRepositorySupport<AG> + Send + Sync,
{
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

    async fn remove(&self, aggregate: AG) -> Result<(), RepositoryError> {
        self.get_aggregate_manager()
            .lock()
            .unwrap()
            .detach(&aggregate);

        self.on_delete(aggregate).await
    }

    async fn save(&self, aggregate: AG) -> Result<AG::ID, RepositoryError> {
        if let Some(id) = aggregate.get_id() {
            let diff = self
                .get_aggregate_manager()
                .lock()
                .unwrap()
                .detect_changes(aggregate.clone());
            if !diff.is_empty() {
                self.on_update(diff).await?;
                self.get_aggregate_manager()
                    .lock()
                    .unwrap()
                    .merge(aggregate);
            }
            Ok(id)
        } else {
            let id = self.on_insert(aggregate.clone()).await?;

            self.get_aggregate_manager()
                .lock()
                .unwrap()
                .attach(aggregate);

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
    T: DbRepositorySupport<AG> + Send + Sync,
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
