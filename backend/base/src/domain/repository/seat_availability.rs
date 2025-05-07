use crate::domain::Repository;
use crate::domain::model::train_schedule::SeatAvailability;

pub trait SeatAvailabilityRepository: Repository<SeatAvailability> {}
