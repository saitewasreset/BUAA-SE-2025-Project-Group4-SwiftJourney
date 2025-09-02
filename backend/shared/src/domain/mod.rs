pub mod model;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{DiffType, Entity, Identifiable, Identifier, TypedDiff};

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct User {
        id: Option<u32>,
        name: String,
    }

    impl Identifier for u32 {}

    impl Identifiable for User {
        type ID = u32;

        fn get_id(&self) -> Option<Self::ID> {
            self.id
        }

        fn set_id(&mut self, id: Self::ID) {
            self.id = Some(id);
        }
    }

    impl Entity for User {}
    impl Aggregate for User {}

    impl Identifier for i32 {}

    // 模拟差异检测函数
    fn mock_diff(diff: DiffInfo<User>) -> MultiEntityDiff {
        let mut result = MultiEntityDiff::new();

        let old = diff.old;
        let new = diff.new;

        match (old.clone(), new.clone()) {
            // 新增操作
            (None, Some(new)) => {
                result.add_change(TypedDiff::new(DiffType::Added, None, Some(new)));
            }
            // 更新操作
            (Some(old), Some(new)) if old != new => {
                result.add_change(TypedDiff::new(DiffType::Modified, Some(old), Some(new)));
            }
            // 删除操作
            (Some(old), None) => {
                result.add_change(TypedDiff::new(DiffType::Removed, Some(old), None));
            }
            // 未变化的情况
            _ => {
                result.add_change(TypedDiff::new(DiffType::Unchanged, old, new));
            }
        }

        result
    }

    #[test]
    fn test_basic_lifecycle() {
        let mut manager = AggregateManagerImpl::new(Box::new(mock_diff));

        // 测试新增
        let user = User {
            id: Some(1),
            name: "Alice".into(),
        };
        let diff = manager.detect_changes(user.clone());
        let changes = diff.get_changes::<User>();
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].diff_type, DiffType::Added);
        assert!(changes[0].old_value.is_none());
        assert_eq!(changes[0].new_value, Some(user.clone()));

        // 测试更新
        manager.merge(user.clone());
        let updated_user = User {
            id: Some(1),
            name: "Bob".into(),
        };
        let diff = manager.detect_changes(updated_user.clone());
        let changes = diff.get_changes::<User>();
        assert_eq!(changes[0].diff_type, DiffType::Modified);
        assert_eq!(changes[0].old_value, Some(user));
        assert_eq!(changes[0].new_value, Some(updated_user.clone()));

        // 测试删除
        let diff = (manager.detect_changes_fn)(DiffInfo {
            old: Some(updated_user.clone()),
            new: None,
        });
        let changes = diff.get_changes::<User>();
        assert_eq!(changes[0].diff_type, DiffType::Removed);
        assert_eq!(changes[0].old_value, Some(updated_user));
        assert!(changes[0].new_value.is_none());
    }

    #[test]
    fn test_unchanged_detection() {
        let mut manager = AggregateManagerImpl::new(Box::new(mock_diff));

        let user = User {
            id: Some(1),
            name: "Alice".into(),
        };
        manager.attach(user.clone());

        // 检测未变化的实体
        let diff = manager.detect_changes(user.clone());
        let changes = diff.get_changes::<User>();
        assert_eq!(changes[0].diff_type, DiffType::Unchanged);
        assert_eq!(changes[0].old_value, Some(user.clone()));
        assert_eq!(changes[0].new_value, Some(user));
    }

    #[test]
    fn test_multi_entity_support() {
        #[derive(Debug, Clone, PartialEq, Eq)]
        struct Order {
            id: Option<i32>,
            order_type: i32,
            product: Vec<Product>,
        }

        #[derive(Debug, Clone, PartialEq, Eq)]
        struct Product {
            id: Option<i32>,
            sku: String,
        }

        impl Identifiable for Product {
            type ID = i32;

            fn get_id(&self) -> Option<Self::ID> {
                self.id
            }

            fn set_id(&mut self, id: Self::ID) {
                self.id = Some(id);
            }
        }

        impl Identifiable for Order {
            type ID = i32;

            fn get_id(&self) -> Option<Self::ID> {
                self.id
            }

            fn set_id(&mut self, id: Self::ID) {
                self.id = Some(id);
            }
        }

        impl Entity for Product {}
        impl Aggregate for Product {}

        impl Entity for Order {}
        impl Aggregate for Order {}

        // 创建支持多实体的检测函数
        let multi_diff = Box::new(|diff: DiffInfo<Order>| {
            let mut result = MultiEntityDiff::new();

            let old = diff.old;
            let new = diff.new;

            match (old.clone(), new.clone()) {
                // 新增操作
                (None, Some(new)) => {
                    result.add_change(TypedDiff::new(DiffType::Added, None, Some(new)));
                }
                // 更新操作
                (Some(old), Some(new)) if old != new => {
                    for product in &new.product {
                        if let Some(old_product) = old.product.iter().find(|p| p.id == product.id) {
                            if old_product != product {
                                result.add_change(TypedDiff::new(
                                    DiffType::Modified,
                                    Some(old_product.clone()),
                                    Some(product.clone()),
                                ));
                            }
                        } else {
                            result.add_change(TypedDiff::new(
                                DiffType::Added,
                                None,
                                Some(product.clone()),
                            ));
                        }
                    }

                    result.add_change(TypedDiff::new(DiffType::Modified, Some(old), Some(new)));
                }
                // 删除操作
                (Some(old), None) => {
                    result.add_change(TypedDiff::new(DiffType::Removed, Some(old), None));
                }
                // 未变化的情况
                _ => {
                    result.add_change(TypedDiff::new(DiffType::Unchanged, old, new));
                }
            }

            result
        });

        let mut manager = AggregateManagerImpl::new(multi_diff);

        let product1 = Product {
            id: Some(1),
            sku: "SKU1".into(),
        };

        let product2 = Product {
            id: Some(2),
            sku: "SKU2".into(),
        };

        let mut order = Order {
            id: Some(1),
            order_type: 1,
            product: vec![product1, product2],
        };

        manager.attach(order.clone());

        order.order_type = 2;
        order.product.first_mut().unwrap().sku = "SKU3".into();

        let diff = manager.detect_changes(order);
        assert_eq!(diff.get_changes::<Order>().len(), 1);
        assert_eq!(diff.get_changes::<Product>().len(), 1);
    }

    #[test]
    fn test_edge_cases() {
        let mut manager = AggregateManagerImpl::new(Box::new(mock_diff));

        // 测试无ID实体
        let ghost_user = User {
            id: None,
            name: "Democracy Has Landed!".into(),
        };
        manager.attach(ghost_user.clone());
        assert!(manager.aggregate_map.is_empty());

        // 测试重复附加
        let user = User {
            id: Some(1),
            name: "For Super Earth!".into(),
        };
        manager.attach(user.clone());
        manager.attach(user.clone());
        assert_eq!(manager.aggregate_map.len(), 1);

        // 测试分离不存在的实体
        manager.detach(&User {
            id: Some(999),
            name: "Not Today!".into(),
        });
        assert_eq!(manager.aggregate_map.len(), 1);
    }
}
