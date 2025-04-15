use crate::domain::{Aggregate, AggregateManager, MultiEntityDiff};
use std::collections::HashMap;

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
    /// # use std::collections::HashMap;
    /// # use base::domain::service::AggregateManagerImpl;
    /// # type User = String;
    /// # struct MultiEntityDiff;
    /// # let changes_fn = Box::new(|_| MultiEntityDiff);
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
    /// 直接调用 [`attach`] 方法，用新实例完全替换旧状态
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
