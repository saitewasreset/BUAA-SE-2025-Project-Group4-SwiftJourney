use shared::data::{CityData, RawDishTakeawayData, StationData, TrainNumberData, TrainTypeData};

pub type LoadCityCommand = CityData;
pub type LoadStationCommand = StationData;
pub type LoadTrainTypeCommand = TrainTypeData;
pub type LoadTrainNumberCommand = TrainNumberData;

pub type LoadDishTakeawayCommand = RawDishTakeawayData;
