use crate::domain::{Aggregate, AggregateManager, Identifier, MultiEntityDiff};
use std::collections::HashMap;
use std::marker::PhantomData;

pub struct DiffInfo<AG, ID>
where
    AG: Aggregate<ID>,
    ID: Identifier,
{
    pub old: Option<AG>,
    pub new: Option<AG>,

    _for_super_earth: PhantomData<ID>,
}

impl<AG, ID> DiffInfo<AG, ID>
where
    AG: Aggregate<ID>,
    ID: Identifier,
{
    pub fn new(old: Option<AG>, new: AG) -> Self {
        DiffInfo {
            old,
            new: Some(new),
            _for_super_earth: PhantomData,
        }
    }
}

pub struct AggregateManagerImpl<AG, ID>
where
    AG: Aggregate<ID>,
    ID: Identifier,
{
    aggregate_map: HashMap<ID, AG>,
    detect_changes_fn: Box<dyn Fn(DiffInfo<AG, ID>) -> MultiEntityDiff + Sync + Send>,
}

impl<AG, ID> AggregateManagerImpl<AG, ID>
where
    AG: Aggregate<ID>,
    ID: Identifier,
{
    pub fn new(
        detect_changes_fn: Box<dyn Fn(DiffInfo<AG, ID>) -> MultiEntityDiff + Sync + Send>,
    ) -> Self {
        AggregateManagerImpl {
            aggregate_map: HashMap::new(),
            detect_changes_fn,
        }
    }
}
impl<AG, ID> AggregateManager<AG, ID> for AggregateManagerImpl<AG, ID>
where
    AG: Aggregate<ID>,
    ID: Identifier,
{
    fn attach(&mut self, aggregate: AG) {
        if let Some(id) = aggregate.get_id() {
            self.aggregate_map.insert(id, aggregate);
        }
    }

    fn detach(&mut self, aggregate: &AG) {
        if let Some(id) = aggregate.get_id() {
            self.aggregate_map.remove(&id);
        }
    }

    fn merge(&mut self, aggregate: AG) {
        self.attach(aggregate);
    }

    fn detect_changes(&self, aggregate: AG) -> MultiEntityDiff {
        let old = aggregate
            .get_id()
            .and_then(|id| self.aggregate_map.get(&id).cloned());

        (self.detect_changes_fn)(DiffInfo::new(old, aggregate))
    }
}
