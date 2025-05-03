use anyhow::Context;
use tracing::{error, instrument};

pub mod session;
pub mod user;

pub mod city;
pub mod mock;
pub mod personal_info;
pub mod route;
pub mod station;
pub mod train;
pub mod train_schedule;

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
