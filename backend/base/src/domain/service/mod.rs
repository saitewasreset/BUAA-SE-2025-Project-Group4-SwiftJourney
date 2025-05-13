//! 领域服务模块 - 实现领域驱动设计中的领域服务层
//!
//! 本模块提供领域服务的基础实现，特别是聚合根状态管理的核心功能。
//! 作为领域模型的操作者和协调者，领域服务封装了不自然属于实体或值对象的领域逻辑。
//!
//! ## 核心组件
//!
//! ### `AggregateManagerImpl`
//! 聚合根管理器的默认实现，提供以下功能：
//! - 聚合根状态的跟踪与维护
//! - 变更检测与差异报告生成
//! - 聚合根生命周期的管理（附加/分离）
//!
//! ## 主要特性
//!
//! ### 状态管理
//! - 使用`HashMap`内部维护聚合根状态
//! - 通过唯一标识符快速查找聚合根
//! - 支持显式的状态合并策略
//!
//! ### 变更检测
//! - 可配置的差异检测策略（通过构造函数注入）
//! - 灵活的状态对比机制
//! - 生成详细的变更报告(`MultiEntityDiff`)
//!
//! ### 线程安全
//! - 所有操作均为线程安全设计
//! - 差异检测函数要求`Sync + Send`约束
//!
//! ## 架构说明
//!
//! 本实现遵循以下设计原则：
//! 1. **单一职责**：专注于聚合根状态管理
//! 2. **开闭原则**：通过函数注入支持扩展
//! 3. **显式状态**：所有变更必须显式提交
//!
//! ## Examples
//! ```
//! use base::domain::{Aggregate, AggregateManager, Entity, Identifiable, Identifier};
//! use base::domain::service::{AggregateManagerImpl, DiffInfo};
//! use base::domain::MultiEntityDiff;
//!
//! #[derive(Debug, Clone, PartialEq)]
//! struct Order {
//!     id: OrderId,
//!     items: Vec<String>,
//! }
//!
//! #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
//! struct OrderId(i32);
//!
//! // 实现必要的特征
//! impl Identifier for OrderId {}
//! impl Identifiable for Order {
//!     type ID = OrderId;
//!     fn get_id(&self) -> Option<Self::ID> { Some(self.id) }
//!     fn set_id(&mut self, id: Self::ID) { self.id = id; }
//! }
//! impl Entity for Order {}
//! impl Aggregate for Order {}
//!
//! // 创建差异检测函数
//! let diff_fn = Box::new(|info: DiffInfo<Order>| {
//!     let mut diff = MultiEntityDiff::new();
//!     // 实现实际的差异检测逻辑...
//!     diff
//! });
//!
//! // 初始化管理器
//! let mut manager = AggregateManagerImpl::new(diff_fn);
//!
//! // 使用示例
//! let order = Order { id: OrderId(1), items: vec!["item1".into()] };
//! manager.attach(order.clone());
//! let changes = manager.detect_changes(order);
//! ```
//!
//! ## 注意事项
//!
//! - 聚合根必须实现`Clone`特征以支持状态快照
//! - 差异检测函数应保持无状态
//! - 管理器不自动持久化变更，需显式调用仓储接口
//!
//! ## 性能考虑
//!
//! - 内部使用`HashMap`保证O(1)查找性能
//! - 差异检测函数应避免复杂计算
//! - 大规模聚合根集合应考虑分片管理

pub mod geo;
pub mod order;
pub mod order_status;
pub mod password;
pub mod route;
pub mod session;
pub mod station;
pub mod train_booking;
pub mod train_schedule;
pub mod train_seat;
pub mod train_type;
pub mod transaction;
pub mod user;

use crate::domain::{Aggregate, AggregateManager, MultiEntityDiff, RepositoryError};
use std::collections::HashMap;
use thiserror::Error;

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
