//! 列车班次领域模型模块
//!
//! 该模块定义了火车票订购系统中的列车班次相关实体。
//! 班次表示特定日期运行的列车实例，包含动态的座位占用情况。
//!
//! # 核心概念
//! - [`TrainSchedule`][]: 列车班次聚合根，表示特定日期的列车运行实例
//! - [`SeatAvailability`][]: 座位可用性信息，记录特定区间的座位占用情况
//! - [`OccupiedSeat`][]: 被占用的座位记录
//! - [`Seat`][]: 座位实体，包含位置信息和状态
//!
//! # 关键特性
//! - 与静态[`crate::domain::model::train::Train`]模板分离，管理动态运行数据
//! - 支持车站区间(StationRange)的座位占用管理
//! - 提供精细化的座位位置信息(车厢、排数、位置)
use super::{
    personal_info::PersonalInfoId,
    station::StationId,
    train::{SeatType, TrainId},
};
use crate::domain::model::route::RouteId;
use crate::domain::model::train::SeatTypeId;
use crate::domain::{Aggregate, Entity, Identifiable, Identifier};
use crate::{Unverified, Verified};
use chrono::NaiveDate;
use id_macro::define_id_type;
use std::collections::HashMap;
use std::marker::PhantomData;

define_id_type!(TrainSchedule);

/// 车站区间值对象
///
/// 表示从出发站到到达站的区间，使用类型状态模式区分验证状态。
///
/// # 泛型参数
/// - `State`: 验证状态标记(`Verified`或`Unverified`)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StationRange<State = Unverified>(StationId, StationId, PhantomData<State>);

impl StationRange<Verified> {
    /// 创建已验证的车站区间
    ///
    /// # Safety
    /// 调用者需确保车站ID有效且顺序合理
    pub fn from_unchecked(
        from_station: StationId,
        to_station: StationId,
    ) -> StationRange<Verified> {
        StationRange(from_station, to_station, PhantomData)
    }
}

impl<T> StationRange<T> {
    /// 获取出发站ID
    pub fn get_from_station_id(&self) -> StationId {
        self.0
    }

    /// 获取到达站ID
    pub fn get_to_station_id(&self) -> StationId {
        self.1
    }
}

impl From<(StationId, StationId)> for StationRange<Unverified> {
    fn from(value: (StationId, StationId)) -> Self {
        Self(value.0, value.1, PhantomData)
    }
}

/// 座位可用性映射类型
///
/// 键为车站区间，值为该区间各座位类型的可用性信息
pub type SeatAvailabilityMap = HashMap<StationRange<Verified>, HashMap<SeatType, SeatAvailability>>;

/// 列车班次聚合根
///
/// 表示特定日期运行的列车实例，包含：
/// - 关联的静态列车模板
/// - 运行日期
/// - 使用的路线
/// - 各区间座位占用情况
///
/// # 字段说明
/// - `id`: 班次唯一标识
/// - `train_id`: 关联的列车模板ID
/// - `date`: 运行日期
/// - `route_id`: 使用的路线ID
/// - `seat_availability`: 座位可用性信息
///
/// # 不变量
/// - 日期必须有效(不早于当前日期)
/// - 路线必须与列车模板兼容
/// - 座位容量不能超过列车模板定义
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrainSchedule {
    id: Option<TrainScheduleId>,
    train_id: TrainId,
    date: NaiveDate,
    route_id: RouteId,
    seat_availability: SeatAvailabilityMap,
}

impl Identifiable for TrainSchedule {
    type ID = TrainScheduleId;

    fn get_id(&self) -> Option<Self::ID> {
        self.id
    }

    /// 设置班次ID并同步更新所有占用座位记录的train_schedule_id
    fn set_id(&mut self, id: Self::ID) {
        self.id = Some(id);
        self.update_occupied_seat_train_schedule_id();
    }
}

impl Entity for TrainSchedule {}
impl Aggregate for TrainSchedule {}

impl TrainSchedule {
    fn seat_availability(
        &self,
        seat_type: &SeatType,
        station_range: &StationRange<Verified>,
    ) -> &SeatAvailability {
        self.seat_availability
            .get(station_range)
            .expect("seat_availability should contain verified station range")
            .get(seat_type)
            .expect("seat_availability should contain verified seat type")
    }

    fn seat_availability_mut(
        &mut self,
        seat_type: &SeatType,
        station_range: &StationRange<Verified>,
    ) -> &mut SeatAvailability {
        self.seat_availability
            .get_mut(station_range)
            .expect("seat_availability should contain verified station range")
            .get_mut(seat_type)
            .expect("seat_availability should contain verified seat type")
    }

    fn update_occupied_seat_train_schedule_id(&mut self) {
        self.seat_availability
            .values_mut()
            .flat_map(|v| v.values_mut())
            .flat_map(|x| x.occupied_seat.values_mut())
            .for_each(|occupied_seat| {
                occupied_seat.train_schedule_id = self.id;

                if let Some(id) = occupied_seat.train_schedule_id {
                    occupied_seat.id = Some(OccupiedSeatId::new(
                        id,
                        occupied_seat.seat_type_id,
                        occupied_seat.seat.id,
                    ));
                }
            });
    }

    /// 创建新班次实例
    ///
    /// # Arguments
    /// * `id`: 班次ID(新建时可为None)
    /// * `train_id`: 关联的列车模板ID
    /// * `date`: 运行日期
    /// * `route_id`: 使用的路线ID
    /// * `seat_availability`: 初始座位可用性信息
    pub fn new(
        id: Option<TrainScheduleId>,
        train_id: TrainId,
        date: NaiveDate,
        route_id: RouteId,
        seat_availability: SeatAvailabilityMap,
    ) -> Self {
        Self {
            id,
            route_id,
            train_id,
            date,
            seat_availability,
        }
    }

    /// 获取关联的列车模板ID
    pub fn train_id(&self) -> TrainId {
        self.train_id
    }

    /// 获取使用的路线ID
    pub fn route_id(&self) -> RouteId {
        self.route_id
    }

    /// 获取运行日期
    pub fn date(&self) -> NaiveDate {
        self.date
    }

    /// 获取座位占用条目的迭代器
    pub fn occupied_entry_iter(&self) -> impl Iterator<Item = &OccupiedSeat> {
        self.seat_availability
            .values()
            .flat_map(|x| x.values())
            .flat_map(|x| x.occupied_seat.values())
    }

    /// 获取座位占用条目的数量
    pub fn occupied_entry_len(&self) -> usize {
        self.occupied_entry_iter().count()
    }

    /// 获取指定区间和座位类型的可用座位数
    ///
    /// # Note
    /// 此函数只统计精确占用StationRange的座位数，更小和更大范围内占用的都**不会**被统计
    pub fn available_seats_count(
        &self,
        seat_type: &SeatType,
        station_range: &StationRange<Verified>,
    ) -> Option<u32> {
        Some(
            self.seat_availability(seat_type, station_range)
                .available_seats_count(),
        )
    }

    pub fn seat_type(&self) -> Vec<SeatType> {
        self.seat_availability
            .values()
            .next()
            .expect("SeatAvailabilityMap should have at least one element")
            .keys()
            .cloned()
            .collect()
    }

    pub fn get_seat_status_by_id(
        &self,
        station_range: &StationRange<Verified>,
        seat_id: SeatId,
    ) -> SeatStatus {
        match self
            .seat_availability
            .get(station_range)
            .expect("seat_availability should contain verified station range")
            .values()
            .flat_map(|x| x.occupied_seat.keys())
            .any(|e| *e == seat_id)
        {
            true => SeatStatus::Occupied,
            false => SeatStatus::Available,
        }
    }

    /// 添加座位占用记录
    ///
    /// # Note
    /// - 若座位已被占用，将替换原有记录
    /// - 此函数只精确占用StationRange，更小和更大范围都**不会**被自动标记占用
    pub fn add_occupied_seat(
        &mut self,
        station_range: &StationRange<Verified>,
        seat: Seat,
        passenger_id: PersonalInfoId,
    ) {
        let seat_type = &seat.seat_type;

        let train_schedule_id = self.id;

        self.seat_availability_mut(seat_type, station_range)
            .add_occupied_seat(train_schedule_id, seat, passenger_id)
    }

    /// 移除座位占用记录
    ///
    /// # Note
    /// 若对应ID的座位未被占有，则不执行任何操作
    pub fn remove_occupied_seat(&mut self, station_range: &StationRange<Verified>, seat: Seat) {
        let seat_type = &seat.seat_type;

        self.seat_availability_mut(seat_type, station_range)
            .remove_occupied_seat(seat);
    }

    pub fn into_seat_availability(self) -> Vec<SeatAvailability> {
        self.seat_availability
            .into_values()
            .flat_map(|x| x.into_values())
            .collect()
    }
}

/// 座位可用性信息
///
/// 记录特定区间内某类座位的占用情况
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeatAvailability {
    seat_type: SeatType,
    from_station: StationId,
    to_station: StationId,
    occupied_seat: HashMap<SeatId, OccupiedSeat>,
}

impl SeatAvailability {
    /// 创建新的座位可用性记录
    pub fn new(seat_type: SeatType, station_range: StationRange<Verified>) -> Self {
        Self {
            seat_type,
            from_station: station_range.get_from_station_id(),
            to_station: station_range.get_to_station_id(),
            occupied_seat: HashMap::new(),
        }
    }

    /// 获取可用座位数
    pub fn available_seats_count(&self) -> u32 {
        self.seat_type.capacity() - self.occupied_seat.len() as u32
    }

    /// 添加座位占用记录
    ///
    /// # Note
    /// - 若座位已被占用，将替换原有记录
    pub fn add_occupied_seat(
        &mut self,
        train_schedule_id: Option<TrainScheduleId>,
        seat: Seat,
        passenger_id: PersonalInfoId,
    ) {
        self.occupied_seat.insert(
            seat.id,
            OccupiedSeat::new(
                train_schedule_id,
                self.seat_type.get_id().expect("seat_type_id should be set"),
                StationRange::from_unchecked(self.from_station, self.to_station),
                seat,
                passenger_id,
            ),
        );
    }

    /// 移除座位占用记录
    ///
    /// # Note
    /// 若对应ID的座位未被占有，则不执行任何操作
    pub fn remove_occupied_seat(&mut self, seat: Seat) {
        self.occupied_seat.remove(&seat.id);
    }

    /// 获取该信息对应的座位类型
    pub fn seat_type(&self) -> &SeatType {
        &self.seat_type
    }

    /// 获取该信息对应的车站区间
    pub fn station_range(&self) -> StationRange<Verified> {
        StationRange::from_unchecked(self.from_station, self.to_station)
    }

    pub fn into_occupied_seat(self) -> HashMap<SeatId, OccupiedSeat> {
        self.occupied_seat
    }
}

/// 已占用座位ID复合标识符
///
/// 由三个部分组成：
/// - 班次ID：标识所属的列车班次
/// - 座位类型ID：标识座位类型
/// - 座位ID：标识具体座位
///
/// # 复合键设计
/// 使用复合ID确保全局唯一性，同时支持快速查询：
/// - 按班次查询所有占用座位
/// - 按座位类型统计占用情况
/// - 精确查找特定座位状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OccupiedSeatId {
    train_schedule_id: TrainScheduleId,
    seat_type_id: SeatTypeId,
    seat_id: SeatId,
}

impl OccupiedSeatId {
    /// 创建新的占用座位ID
    ///
    /// # Arguments
    /// * `train_schedule_id`: 关联的班次ID
    /// * `seat_type_id`: 座位类型ID
    /// * `seat_id`: 具体座位ID
    pub fn new(
        train_schedule_id: TrainScheduleId,
        seat_type_id: SeatTypeId,
        seat_id: SeatId,
    ) -> Self {
        Self {
            train_schedule_id,
            seat_type_id,
            seat_id,
        }
    }

    /// 获取关联班次ID
    pub fn train_schedule_id(&self) -> TrainScheduleId {
        self.train_schedule_id
    }

    /// 获取座位类型ID
    pub fn seat_type_id(&self) -> SeatTypeId {
        self.seat_type_id
    }

    /// 获取具体座位ID
    pub fn seat_id(&self) -> SeatId {
        self.seat_id
    }
}

impl Identifier for OccupiedSeatId {}

/// 已占用座位实体
///
/// 记录乘客对特定座位的占用情况，包含：
/// - 占用时间范围（车站区间）
/// - 乘客信息
/// - 座位详细信息
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OccupiedSeat {
    id: Option<OccupiedSeatId>,
    train_schedule_id: Option<TrainScheduleId>,
    seat_type_id: SeatTypeId,
    station_range: StationRange<Verified>,
    seat: Seat,
    passenger_id: PersonalInfoId,
}

impl Identifiable for OccupiedSeat {
    type ID = OccupiedSeatId;

    fn get_id(&self) -> Option<Self::ID> {
        self.id
    }

    fn set_id(&mut self, id: Self::ID) {
        self.id = Some(id)
    }
}

impl Entity for OccupiedSeat {}

impl OccupiedSeat {
    /// 创建新的占用记录
    ///
    /// # Arguments
    /// * `train_schedule_id`: 关联班次ID（新建时可能为None）
    /// * `seat_type_id`: 座位类型ID
    /// * `station_range`: 有效的车站区间
    /// * `seat`: 被占用的座位实体
    /// * `passenger_id`: 乘客信息ID
    ///
    /// # 注意
    /// 当`train_schedule_id`为None时，ID将无法生成
    /// 在班次插入数据库时，其ID生成，并自动更新关联占用记录的ID
    pub fn new(
        train_schedule_id: Option<TrainScheduleId>,
        seat_type_id: SeatTypeId,
        station_range: StationRange<Verified>,
        seat: Seat,
        passenger_id: PersonalInfoId,
    ) -> Self {
        let id = train_schedule_id
            .map(|train_schedule_id| OccupiedSeatId::new(train_schedule_id, seat_type_id, seat.id));

        Self {
            id,
            train_schedule_id,
            seat_type_id,
            station_range,
            seat,
            passenger_id,
        }
    }

    /// 获取被占用的座位引用
    pub fn seat(&self) -> &Seat {
        &self.seat
    }

    /// 获取乘客信息ID
    pub fn passenger_id(&self) -> PersonalInfoId {
        self.passenger_id
    }

    /// 获取占用区间
    ///
    /// # Returns
    /// 返回已验证的车站区间（出发站→到达站）
    pub fn station_range(&self) -> StationRange<Verified> {
        self.station_range
    }
}

/// 座位状态枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SeatStatus {
    Available,
    Occupied,
}

define_id_type!(Seat);
/// 座位实体
///
/// 表示列车上的具体座位，包含位置信息和当前状态
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Seat {
    id: SeatId,
    seat_type: SeatType,
    info: SeatLocationInfo,
    status: SeatStatus,
}

impl Identifiable for Seat {
    type ID = SeatId;

    fn get_id(&self) -> Option<Self::ID> {
        Some(self.id)
    }

    fn set_id(&mut self, id: Self::ID) {
        self.id = id;
    }
}

impl Seat {
    /// 创建新座位实例
    pub fn new(id: SeatId, seat_type: SeatType, info: SeatLocationInfo) -> Self {
        Self {
            id,
            seat_type,
            info,
            status: SeatStatus::Available,
        }
    }

    /// 获取座位类型
    pub fn seat_type(&self) -> &SeatType {
        &self.seat_type
    }

    /// 获取座位位置信息
    pub fn location_info(&self) -> SeatLocationInfo {
        self.info
    }

    /// 获取座位状态
    pub fn status(&self) -> SeatStatus {
        self.status
    }
}

/// 座位位置信息值对象
///
/// 表示座位的具体位置，如"03车11A"
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SeatLocationInfo {
    pub carriage: i32,  // 车厢号(如3)
    pub row: i32,       // 排数(如11)
    pub location: char, // 位置标记(如'A')
}
