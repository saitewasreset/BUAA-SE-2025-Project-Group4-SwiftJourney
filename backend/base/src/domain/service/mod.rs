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
