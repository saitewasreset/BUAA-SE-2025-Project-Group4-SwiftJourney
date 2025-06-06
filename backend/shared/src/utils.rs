use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::time::Duration;

pub struct TimeMeter {
    job_name: String,
    last_instant: std::time::Instant,
    duration_map: HashMap<String, (usize, Duration)>,
    order: usize,
}

impl TimeMeter {
    pub fn new(job_name: &str) -> Self {
        TimeMeter {
            job_name: job_name.to_string(),
            last_instant: std::time::Instant::now(),
            duration_map: HashMap::new(),
            order: 0,
        }
    }

    pub fn meter(&mut self, key: &str) {
        let now = std::time::Instant::now();
        let duration = now.duration_since(self.last_instant);
        self.duration_map
            .insert(key.to_string(), (self.order, duration));
        self.last_instant = now;

        self.order += 1;
    }

    pub fn get_duration(&self, key: &str) -> Option<Duration> {
        self.duration_map.get(key).map(|tuple| tuple.1)
    }

    pub fn summarize(&self) -> String {
        let total_duration: Duration = self.duration_map.values().map(|tuple| tuple.1).sum();

        let mut summary = format!("{}: {:?} = ", self.job_name, total_duration);

        let mut tuples = self.duration_map.iter().collect::<Vec<_>>();

        tuples.sort_by(|(_, (order_a, _)), (_, (order_b, _))| order_a.cmp(order_b));

        let sorted_tuples = tuples;

        let parts_str = sorted_tuples
            .iter()
            .map(|(key, (_, duration))| format!("{:?} ({})", duration, key))
            .collect::<Vec<_>>();

        summary.push_str(&parts_str.join(" + "));

        summary
    }
}

impl Display for TimeMeter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.summarize())
    }
}
