use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::hash::Hash;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};
use thiserror::Error;

pub mod model;
pub mod repository;
pub mod service;

// Identifier必须是'static的
// 注意，这并不是表述Identifier需要从程序开始执行到结束一直存在
// 而表示我们可以持有一个Identifier任意长的时间
// 详见：https://github.com/pretzelhammer/rust-blog/blob/master/posts/common-rust-lifetime-misconceptions.md
// 在Repository中，我们通过Snapshot来追踪实体状态变更，这需要实体实现Any特征
// 而Any特征只为'static的类型实现
pub trait Identifier: 'static + Send + Clone + PartialEq + Eq + Hash {}

pub trait Identifiable<ID>
where
    ID: Identifier,
{
    fn get_id(&self) -> Option<ID>;
}

// 在Repository中，我们通过Snapshot来追踪实体状态变更，这需要实体实现Any特征
// 而Any特征只为'static的类型实现
pub trait Entity<ID>: Identifiable<ID> + 'static + Send + Clone
where
    ID: Identifier,
{
}

pub trait Aggregate<ID>: Entity<ID>
where
    ID: Identifier,
{
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiffType {
    Added,
    Removed,
    Modified,
    Unchanged,
}

pub trait Diff {
    fn diff_type(&self) -> DiffType;

    fn is_empty(&self) -> bool;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedDiff<T, ID>
where
    T: Entity<ID>,
    ID: Identifier,
{
    pub diff_type: DiffType,
    pub old_value: Option<T>,
    pub new_value: Option<T>,

    _for_super_earth: PhantomData<ID>,
}

impl<T, ID> Diff for TypedDiff<T, ID>
where
    T: Entity<ID>,
    ID: Identifier,
{
    fn diff_type(&self) -> DiffType {
        self.diff_type
    }

    fn is_empty(&self) -> bool {
        self.diff_type == DiffType::Unchanged
    }
}

trait AnyDiff: Send {
    fn as_any(&self) -> &dyn Any;
}

impl<T, ID> AnyDiff for TypedDiff<T, ID>
where
    T: Entity<ID>,
    ID: Identifier,
{
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Default)]
pub struct MultiEntityDiff {
    changes: HashMap<TypeId, Vec<Box<dyn AnyDiff>>>,
}

impl MultiEntityDiff {
    pub fn new() -> Self {
        MultiEntityDiff {
            changes: HashMap::new(),
        }
    }

    pub fn add_change<T, ID>(&mut self, diff: TypedDiff<T, ID>)
    where
        T: Entity<ID>,
        ID: Identifier,
    {
        self.changes
            .entry(TypeId::of::<TypedDiff<T, ID>>())
            .or_default()
            .push(Box::new(diff))
    }

    pub fn get_changes<T, ID>(&self) -> Vec<TypedDiff<T, ID>>
    where
        T: Entity<ID>,
        ID: Identifier,
    {
        self.changes
            .get(&TypeId::of::<TypedDiff<T, ID>>())
            .map(|v| {
                v.iter()
                    .filter_map(|d| d.as_any().downcast_ref::<TypedDiff<T, ID>>())
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }
}

pub trait AggregateManager<AG, ID>
where
    AG: Aggregate<ID>,
    ID: Identifier,
{
    fn attach(&mut self, aggregate: AG);
    fn detach(&mut self, aggregate: &AG);
    fn merge(&mut self, aggregate: AG);
    fn detect_changes(&self, aggregate: AG) -> MultiEntityDiff;
}

#[derive(Error, Debug)]
pub enum RepositoryError {
    #[error("database error: {0}")]
    Db(anyhow::Error),

    #[error("invalid data object from db")]
    ValidationError(#[from] anyhow::Error),
}

pub trait Repository<AG, ID>
where
    AG: Aggregate<ID>,
    ID: Identifier,
{
    fn attach(&self, aggregate: AG);
    fn detach(&self, aggregate: &AG);

    fn find(&self, id: ID) -> impl Future<Output = Result<Option<AG>, RepositoryError>> + Send;
    fn remove(&self, aggregate: AG) -> impl Future<Output = Result<(), RepositoryError>> + Send;
    fn save(&self, aggregate: AG) -> impl Future<Output = Result<(), RepositoryError>> + Send;
}

pub trait DbRepositorySupport<AG, ID>
where
    AG: Aggregate<ID>,
    ID: Identifier,
{
    type Manager: AggregateManager<AG, ID>;

    fn get_aggregate_manager(&self) -> Arc<Mutex<Self::Manager>>;
    fn on_insert(&self, aggregate: AG) -> impl Future<Output = Result<(), RepositoryError>> + Send;
    fn on_select(&self, id: ID)
    -> impl Future<Output = Result<Option<AG>, RepositoryError>> + Send;
    fn on_update(
        &self,
        diff: MultiEntityDiff,
    ) -> impl Future<Output = Result<(), RepositoryError>> + Send;
    fn on_delete(&self, aggregate: AG) -> impl Future<Output = Result<(), RepositoryError>> + Send;
}

impl<AG, ID, T> Repository<AG, ID> for T
where
    AG: Aggregate<ID>,
    ID: Identifier,
    T: DbRepositorySupport<AG, ID> + Send + Sync,
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

    async fn find(&self, id: ID) -> Result<Option<AG>, RepositoryError> {
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

    async fn save(&self, aggregate: AG) -> Result<(), RepositoryError> {
        if aggregate.get_id().is_none() {
            self.on_insert(aggregate.clone()).await
        } else {
            let diff = self
                .get_aggregate_manager()
                .lock()
                .unwrap()
                .detect_changes(aggregate.clone());
            if !diff.is_empty() {
                self.on_update(diff).await
            } else {
                Ok(())
            }
        }
    }
}
