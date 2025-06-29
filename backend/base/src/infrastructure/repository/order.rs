use crate::domain::model::hotel::HotelId;
use crate::domain::model::order::{
    DishOrder, HotelOrder, Order, OrderId, OrderStatus, TakeawayOrder, TrainOrder,
};
use crate::domain::model::train_schedule::TrainScheduleId;
use crate::domain::model::user::UserId;
use crate::domain::repository::order::{
    DishOrderRelatedData, HotelOrderRelatedData, OrderRepository, RouteInfo,
    TakeawayOrderRelatedData, TrainOrderRelatedData,
};
use crate::domain::{DbId, RepositoryError};
use crate::infrastructure::repository::transaction::{OrderDataConverter, TrainOrderDoPack};
use anyhow::{Context, anyhow};
use async_trait::async_trait;
use chrono::{FixedOffset, NaiveDate, NaiveTime};
use sea_orm::ColumnTrait;
use sea_orm::{
    DatabaseBackend, DatabaseConnection, DbBackend, EntityTrait, FromQueryResult, QueryFilter,
    Statement,
};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use tracing::{error, instrument};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Hash, FromQueryResult)]
struct TrainOrderQueryResult {
    /// 车次号
    pub train_number: String,
    /// 车次离开始发站相对当日00:00:00的秒数
    pub departure_time: i32,
    /// 旅客姓名
    pub name: String,
    /// 起始站
    pub begin_station_id: i32,
    /// 达到站
    pub end_station_id: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, FromQueryResult)]
struct DishOrderQueryResult {
    /// 车次号
    pub train_number: String,
    /// 离开起始站的时间相对当日00:00:00的秒数
    pub departure_time: i32,
    /// 点餐人姓名
    pub name: String,
    /// 餐品名称
    pub dish_name: String,
    /// 用餐时间
    pub dish_time: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, FromQueryResult)]
struct TakeawayOrderQueryResult {
    pub train_number: String,
    pub station_id: i32,
    pub shop_name: String,
    pub name: String,
    pub takeaway_name: String,
}

#[derive(Debug, FromQueryResult)]
struct DepartureTimeQueryResult {
    pub departure_date: NaiveDate,
    pub origin_departure_time: i32,
}

pub struct OrderRepositoryImpl {
    db: DatabaseConnection,
}

impl OrderRepositoryImpl {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}
/*
SELECT route.order, route.arrival_time, route.departure_time, station.id, station.name FROM train_order
    INNER JOIN train_schedule
        ON train_schedule.id = train_order.train_schedule_id
    INNER JOIN route
        ON route.line_id = train_schedule.line_id
    INNER JOIN station
        ON station.id = route.station_id
*/

#[async_trait]
impl OrderRepository for OrderRepositoryImpl {
    async fn find_train_order_by_uuid(
        &self,
        order_uuid: Uuid,
    ) -> Result<Option<TrainOrder>, RepositoryError> {
        let train_order = crate::models::train_order::Entity::find()
            .filter(crate::models::train_order::Column::Uuid.eq(order_uuid))
            .one(&self.db)
            .await
            .context(format!(
                "failed to find train order for order uuid: {}",
                order_uuid
            ))?;

        if let Some(train_order) = train_order {
            let seat_type_models = crate::models::seat_type::Entity::find()
                .all(&self.db)
                .await
                .context("failed to find seat type from db")?;

            let seat_type_mapping_models = crate::models::seat_type_mapping::Entity::find()
                .from_raw_sql(Statement::from_sql_and_values(
                    DbBackend::Postgres,
                    r#"SELECT
    "seat_type_mapping"."train_type_id",
    "seat_type_mapping"."seat_type_id",
    "seat_type_mapping"."seat_id",
    "seat_type_mapping"."carriage",
    "seat_type_mapping"."row",
    "seat_type_mapping"."location"
FROM "train_order"
    INNER JOIN "train_schedule"
        ON "train_order"."train_schedule_id" = "train_schedule"."id"
    INNER JOIN "train"
        ON "train_schedule"."train_id" = "train"."id"
    INNER JOIN "seat_type_mapping"
        ON "seat_type_mapping"."train_type_id" = "train"."type_id"
    WHERE "train_order"."uuid" = $1"#,
                    [order_uuid.into()],
                ))
                .all(&self.db)
                .await
                .context(format!(
                    "failed to find seat type mappings from db for order uuid: {}",
                    order_uuid
                ))?;

            let seat_type_map = seat_type_models
                .into_iter()
                .map(|x| (x.id, x))
                .collect::<HashMap<_, _>>();

            let mut seat_type_mapping_map: HashMap<
                i32,
                HashMap<i64, crate::models::seat_type_mapping::Model>,
            > = HashMap::new();

            for model in seat_type_mapping_models {
                seat_type_mapping_map
                    .entry(model.seat_type_id)
                    .or_default()
                    .insert(model.seat_id, model);
            }

            let train_order_do_pack = TrainOrderDoPack {
                train_order,
                seat_type: seat_type_map,
                seat_type_mapping: seat_type_mapping_map,
            };

            let result = OrderDataConverter::make_from_do_train(train_order_do_pack)
                .map_err(RepositoryError::InconsistentState)?;

            Ok(Some(result))
        } else {
            Ok(None)
        }
    }

    async fn find_hotel_order_by_uuid(
        &self,
        order_uuid: Uuid,
    ) -> Result<Option<HotelOrder>, RepositoryError> {
        let hotel_order_do = crate::models::hotel_order::Entity::find()
            .filter(crate::models::hotel_order::Column::Uuid.eq(order_uuid))
            .one(&self.db)
            .await
            .context(format!(
                "failed to find hotel order for order uuid: {}",
                order_uuid
            ))?;

        if let Some(hotel_order_do) = hotel_order_do {
            let result = OrderDataConverter::make_from_do_hotel(hotel_order_do)
                .map_err(RepositoryError::InconsistentState)?;

            Ok(Some(result))
        } else {
            Ok(None)
        }
    }

    async fn find_hotel_order_by_userid(
        &self,
        user_id: UserId,
        hotel_id: HotelId,
    ) -> Result<Vec<HotelOrder>, RepositoryError> {
        crate::models::hotel_order::Entity::find()
            .from_raw_sql(Statement::from_sql_and_values(
                DatabaseBackend::Postgres,
                r#"SELECT
    "hotel_order"."id",
    "hotel_order"."uuid",
    "hotel_order"."hotel_id",
    "hotel_order"."begin_date",
    "hotel_order"."end_date",
    "hotel_order"."hotel_room_type_id",
    "hotel_order"."person_info_id",
    "hotel_order"."pay_transaction_id",
    "hotel_order"."refund_transaction_id",
    "hotel_order"."price",
    "hotel_order"."amount",
    "hotel_order"."create_time",
    "hotel_order"."active_time",
    "hotel_order"."complete_time",
    "hotel_order"."status"
FROM "hotel_order"
    INNER JOIN "person_info"
        ON "hotel_order"."person_info_id" = "person_info"."id"
    INNER JOIN "user"
        ON "person_info"."user_id" = "user"."id"
WHERE "user"."id" = $1 AND "hotel_order"."hotel_id" = $2"#,
                [user_id.to_db_value().into(), hotel_id.to_db_value().into()],
            ))
            .all(&self.db)
            .await
            .context(format!(
                "failed to find hotel order for user id: {} and hotel id: {}",
                user_id.to_db_value(),
                hotel_id.to_db_value()
            ))?
            .into_iter()
            .map(|x| {
                OrderDataConverter::make_from_do_hotel(x)
                    .map_err(RepositoryError::InconsistentState)
            })
            .collect()
    }

    async fn find_dish_order_by_uuid(
        &self,
        order_uuid: Uuid,
    ) -> Result<Option<DishOrder>, RepositoryError> {
        let dish_order_do = crate::models::dish_order::Entity::find()
            .filter(crate::models::dish_order::Column::Uuid.eq(order_uuid))
            .one(&self.db)
            .await
            .context(format!(
                "failed to find dish order for order uuid: {}",
                order_uuid
            ))?;

        if let Some(dish_order_do) = dish_order_do {
            let result = OrderDataConverter::make_from_do_dish(dish_order_do)
                .map_err(RepositoryError::InconsistentState)?;

            Ok(Some(result))
        } else {
            Ok(None)
        }
    }

    async fn find_takeaway_order_by_uuid(
        &self,
        order_uuid: Uuid,
    ) -> Result<Option<TakeawayOrder>, RepositoryError> {
        let takeaway_order_do = crate::models::takeaway_order::Entity::find()
            .filter(crate::models::takeaway_order::Column::Uuid.eq(order_uuid))
            .one(&self.db)
            .await
            .context(format!(
                "failed to find takeaway order for order uuid: {}",
                order_uuid
            ))?;

        if let Some(takeaway_order_do) = takeaway_order_do {
            let result = OrderDataConverter::make_from_do_takeaway(takeaway_order_do)
                .map_err(RepositoryError::InconsistentState)?;

            Ok(Some(result))
        } else {
            Ok(None)
        }
    }

    #[instrument(skip(self))]
    async fn load_all_active_orders(&self) -> Result<Vec<Box<dyn Order>>, RepositoryError> {
        let train_list = crate::models::train::Entity::find()
            .all(&self.db)
            .await
            .inspect_err(|e| error!("Failed to load trains: {}", e))
            .context("failed to load trains from db")?;

        let train_schedule_list = crate::models::train_schedule::Entity::find()
            .all(&self.db)
            .await
            .inspect_err(|e| error!("Failed to load train schedules: {}", e))
            .context("failed to load train schedules from db")?;

        let train_id_to_train_type_id = train_list
            .into_iter()
            .map(|x| (x.id, x.type_id))
            .collect::<HashMap<_, _>>();

        let train_schedule_id_to_train_id = train_schedule_list
            .into_iter()
            .map(|x| (x.id, x.train_id))
            .collect::<HashMap<_, _>>();

        let mut train_schedule_id_to_train_type_id: HashMap<i32, i32> = HashMap::new();

        for (train_schedule_id, train_id) in train_schedule_id_to_train_id {
            if let Some(train_type_id) = train_id_to_train_type_id.get(&train_id) {
                train_schedule_id_to_train_type_id.insert(train_schedule_id, *train_type_id);
            } else {
                return Err(RepositoryError::InconsistentState(anyhow!(
                    "no train type id for train schedule id: {}",
                    train_schedule_id
                )));
            }
        }

        let train_orders = crate::models::train_order::Entity::find()
            .all(&self.db)
            .await
            .inspect_err(|e| error!("Failed to load train orders: {}", e))
            .context("failed to load train orders".to_string())?;

        let hotel_orders = crate::models::hotel_order::Entity::find()
            .all(&self.db)
            .await
            .inspect_err(|e| error!("Failed to load hotel orders: {}", e))
            .context("failed to load hotel orders".to_string())?;

        let dish_orders = crate::models::dish_order::Entity::find()
            .all(&self.db)
            .await
            .inspect_err(|e| error!("Failed to load dish orders: {}", e))
            .context("failed to load dish orders".to_string())?;

        let takeaway_orders = crate::models::takeaway_order::Entity::find()
            .all(&self.db)
            .await
            .inspect_err(|e| error!("Failed to load takeaway orders: {}", e))
            .context("failed to load takeaway orders".to_string())?;

        let seat_type_models = crate::models::seat_type::Entity::find()
            .all(&self.db)
            .await
            .inspect_err(|e| error!("Failed to load seat type: {}", e))
            .context("failed to find seat type from db")?;

        let seat_type_mapping_models = crate::models::seat_type_mapping::Entity::find()
            .all(&self.db)
            .await
            .inspect_err(|e| error!("Failed to load seat type mappings: {}", e))
            .context("failed to load seat type mapping from db".to_string())?;

        let seat_type_map = seat_type_models
            .into_iter()
            .map(|x| (x.id, x))
            .collect::<HashMap<_, _>>();

        let mut train_type_id_to_seat_type_mapping_map: HashMap<
            i32,
            HashMap<i32, HashMap<i64, crate::models::seat_type_mapping::Model>>,
        > = HashMap::new();

        for model in seat_type_mapping_models {
            train_type_id_to_seat_type_mapping_map
                .entry(model.train_type_id)
                .or_default()
                .entry(model.seat_type_id)
                .or_default()
                .insert(model.seat_id, model);
        }

        let mut result = Vec::with_capacity(
            train_orders.len() + hotel_orders.len() + dish_orders.len() + takeaway_orders.len(),
        );

        for order in train_orders {
            let train_type_id = train_schedule_id_to_train_type_id
                .get(&order.train_schedule_id)
                .copied()
                .ok_or(RepositoryError::InconsistentState(anyhow!(
                    "no train type id for train schedule id: {}",
                    order.train_schedule_id
                )))?;

            let train_order_do_pack = TrainOrderDoPack {
                train_order: order,
                seat_type: seat_type_map.clone(),
                seat_type_mapping: train_type_id_to_seat_type_mapping_map
                    .get(&train_type_id)
                    .cloned()
                    .ok_or(RepositoryError::InconsistentState(anyhow!(
                        "no seat type mapping for train type id: {}",
                        train_type_id
                    )))?,
            };

            let train_order = OrderDataConverter::make_from_do_train(train_order_do_pack)
                .map_err(RepositoryError::InconsistentState)?;

            result.push(Box::new(train_order) as Box<dyn Order>);
        }

        for order in hotel_orders {
            let hotel_order = OrderDataConverter::make_from_do_hotel(order)
                .map_err(RepositoryError::InconsistentState)?;

            result.push(Box::new(hotel_order) as Box<dyn Order>);
        }

        for order in dish_orders {
            let dish_order = OrderDataConverter::make_from_do_dish(order)
                .map_err(RepositoryError::InconsistentState)?;

            result.push(Box::new(dish_order) as Box<dyn Order>);
        }

        for order in takeaway_orders {
            let takeaway_order = OrderDataConverter::make_from_do_takeaway(order)
                .map_err(RepositoryError::InconsistentState)?;

            result.push(Box::new(takeaway_order) as Box<dyn Order>);
        }

        Ok(result
            .into_iter()
            .filter(|order| {
                order.order_status() == OrderStatus::Ongoing
                    || order.order_status() == OrderStatus::Active
            })
            .collect())
    }

    async fn update(&self, order: Box<dyn Order>) -> Result<(), RepositoryError> {
        match order.as_ref().type_id() {
            id if id == TypeId::of::<TrainOrder>() => {
                let train_order = (order as Box<dyn Any>).downcast::<TrainOrder>().unwrap();

                let order_uuid = train_order.uuid();

                let train_order_do = OrderDataConverter::transform_to_do_train(*train_order);

                crate::models::train_order::Entity::update(train_order_do)
                    .exec(&self.db)
                    .await
                    .context(format!("failed to update train order uuid: {}", order_uuid))?;
            }
            id if id == TypeId::of::<HotelOrder>() => {
                let hotel_order = (order as Box<dyn Any>).downcast::<HotelOrder>().unwrap();

                let order_uuid = hotel_order.uuid();

                let hotel_order_do = OrderDataConverter::transform_to_do_hotel(*hotel_order);

                crate::models::hotel_order::Entity::update(hotel_order_do)
                    .exec(&self.db)
                    .await
                    .context(format!("failed to update hotel order uuid: {}", order_uuid))?;
            }
            id if id == TypeId::of::<DishOrder>() => {
                let dish_order = (order as Box<dyn Any>).downcast::<DishOrder>().unwrap();

                let order_uuid = dish_order.uuid();

                let dish_order_do = OrderDataConverter::transform_to_do_dish(*dish_order);

                crate::models::dish_order::Entity::update(dish_order_do)
                    .exec(&self.db)
                    .await
                    .context(format!("failed to update dish order uuid: {}", order_uuid))?;
            }
            id if id == TypeId::of::<TakeawayOrder>() => {
                let takeaway_order = (order as Box<dyn Any>).downcast::<TakeawayOrder>().unwrap();

                let order_uuid = takeaway_order.uuid();

                let takeaway_order_do =
                    OrderDataConverter::transform_to_do_takeaway(*takeaway_order);

                crate::models::takeaway_order::Entity::update(takeaway_order_do)
                    .exec(&self.db)
                    .await
                    .context(format!(
                        "failed to update takeaway order uuid: {}",
                        order_uuid
                    ))?;
            }
            _ => panic!("Unknown order type"),
        }

        Ok(())
    }

    /// 保证按order升序排列
    async fn get_route_info_train_order(
        &self,
        order_id: OrderId,
        train_schedule_id: TrainScheduleId,
    ) -> Result<(NaiveDate, Vec<RouteInfo>), RepositoryError> {
        // 此时是相对发车时间
        let mut result = RouteInfo::find_by_statement(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            r#"SELECT
"route"."order" AS "order",
"route"."arrival_time" AS "arrival_time",
"route"."departure_time" AS "departure_time",
"station"."id" AS "station_id",
"station"."name" AS "station_name"
FROM "train_order"
INNER JOIN "train_schedule"
        ON "train_schedule"."id" = "train_order"."train_schedule_id"
    INNER JOIN "route"
        ON "route"."line_id" = "train_schedule"."line_id"
    INNER JOIN "station"
        ON "station"."id" = "route"."station_id"
    WHERE "train_order"."id" = $1
    ORDER BY "route"."order""#,
            [order_id.to_db_value().into()],
        ))
        .all(&self.db)
        .await
        .context("Failed to select route info")?;

        let train_schedule =
            crate::models::train_schedule::Entity::find_by_id(train_schedule_id.to_db_value())
                .one(&self.db)
                .await
                .context("failed to query train schedule")?
                .ok_or(RepositoryError::InconsistentState(anyhow!(
                    "no train schedule for id: {}",
                    train_schedule_id.to_db_value()
                )))?;

        for data in &mut result {
            data.departure_time += train_schedule.origin_departure_time;
            data.arrival_time += train_schedule.origin_departure_time;
        }

        Ok((train_schedule.departure_date, result))
    }

    async fn get_route_info_takeaway_order(
        &self,
        order_id: OrderId,
        train_order_id: OrderId,
    ) -> Result<(NaiveDate, Vec<RouteInfo>), RepositoryError> {
        // 此时是相对发车时间
        let mut result = RouteInfo::find_by_statement(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            r#"SELECT
"route"."order" AS "order",
"route"."arrival_time" AS "arrival_time",
"route"."departure_time" AS "departure_time",
"station"."id" AS "station_id",
"station"."name" AS "station_name"
FROM "takeaway_order"
    INNER JOIN "train_order"
        ON "takeaway_order"."train_order_id" = "train_order"."id"
    INNER JOIN "train_schedule"
        ON "train_schedule"."id" = "train_order"."train_schedule_id"
    INNER JOIN "route"
        ON "route"."line_id" = "train_schedule"."line_id"
    INNER JOIN "station"
        ON "station"."id" = "route"."station_id"
    WHERE "takeaway_order"."id" = $1 ORDER BY "route"."order""#,
            [order_id.to_db_value().into()],
        ))
        .all(&self.db)
        .await
        .context("Failed to select route info")?;

        let train_order =
            crate::models::train_order::Entity::find_by_id(train_order_id.to_db_value())
                .one(&self.db)
                .await
                .context("failed to query train order")?
                .ok_or(RepositoryError::InconsistentState(anyhow!(
                    "no train order for id: {}",
                    train_order_id.to_db_value()
                )))?;

        let train_schedule =
            crate::models::train_schedule::Entity::find_by_id(train_order.train_schedule_id)
                .one(&self.db)
                .await
                .context("failed to query train schedule")?
                .ok_or(RepositoryError::InconsistentState(anyhow!(
                    "no train schedule for id: {}",
                    train_order.train_schedule_id
                )))?;

        for data in &mut result {
            data.departure_time += train_schedule.origin_departure_time;
            data.arrival_time += train_schedule.origin_departure_time;
        }

        Ok((train_schedule.departure_date, result))
    }

    #[instrument(skip(self))]
    async fn get_train_order_related_data(
        &self,
        order_id: OrderId,
        train_schedule_id: TrainScheduleId,
        tz_offset_hour: i32,
    ) -> Result<TrainOrderRelatedData, RepositoryError> {
        let query_result =
            TrainOrderQueryResult::find_by_statement(Statement::from_sql_and_values(
                DatabaseBackend::Postgres,
                r#"SELECT "train"."number" AS "train_number",
"route"."departure_time" AS "departure_time",
"person_info"."name" AS "name",
"train_order"."begin_station_id" AS "begin_station_id",
"train_order"."end_station_id" AS "end_station_id"
FROM "train_order"
    INNER JOIN "train_schedule"
        ON "train_schedule"."id" =  "train_order"."train_schedule_id"
    INNER JOIN "train"
        ON "train"."id" = "train_schedule"."train_id"
    INNER JOIN "route"
        ON "route"."line_id" = "train_schedule"."line_id"
    INNER JOIN "person_info"
        ON "person_info"."id" = "train_order"."person_info_id"
    WHERE "train_order"."id" = $1 AND "route"."order" = 0;"#,
                [order_id.to_db_value().into()],
            ))
            .one(&self.db)
            .await
            .inspect_err(|e| error!("Failed to query related train order data: {}", e))
            .context("failed to query related train order data")?
            .ok_or(RepositoryError::InconsistentState(anyhow!(
                "no related data for train order id: {}",
                order_id.to_db_value()
            )))?;

        let (departure_date, route_info_list) = self
            .get_route_info_train_order(order_id, train_schedule_id)
            .await?;

        if route_info_list.len() < 2 {
            panic!(
                "Invalid line for train schedule id: {}, less than 2 stations",
                train_schedule_id.to_db_value()
            );
        }

        let station_id_to_route_info = route_info_list
            .iter()
            .map(|item| (item.station_id, item.clone()))
            .collect::<HashMap<_, _>>();

        let departure_station_info = station_id_to_route_info.get(&query_result.begin_station_id).ok_or_else(|| {
            error!("Inconsistent: begin station id {} not found in route info list for order id: {}", query_result.begin_station_id, order_id.to_db_value());

            RepositoryError::InconsistentState(anyhow!("Inconsistent: begin station id {} not found in route info list for order id: {}", query_result.begin_station_id, order_id.to_db_value()))
        })?;

        let arrival_station_info = station_id_to_route_info
            .get(&query_result.end_station_id)
            .ok_or_else(|| {
                error!(
                    "Inconsistent: end station id {} not found in route info list for order id: {}",
                    query_result.end_station_id,
                    order_id.to_db_value()
                );

                RepositoryError::InconsistentState(anyhow!(
                    "Inconsistent: end station id {} not found in route info list for order id: {}",
                    query_result.end_station_id,
                    order_id.to_db_value()
                ))
            })?;

        let origin_station_info = &route_info_list[0];
        let terminal_station_info = &route_info_list[route_info_list.len() - 1];

        let departure_time_sec = departure_station_info.departure_time;
        let terminal_time_sec = arrival_station_info.arrival_time;

        let origin_departure_time_sec = origin_station_info.departure_time;
        let terminal_arrival_time_sec = terminal_station_info.arrival_time;

        let tz = FixedOffset::east_opt(tz_offset_hour * 3600).unwrap();

        let departure_time = departure_date
            .and_time(
                NaiveTime::from_num_seconds_from_midnight_opt(departure_time_sec as u32, 0)
                    .unwrap(),
            )
            .and_local_timezone(tz)
            .unwrap();

        let terminal_time = departure_date
            .and_time(
                NaiveTime::from_num_seconds_from_midnight_opt(terminal_time_sec as u32, 0).unwrap(),
            )
            .and_local_timezone(tz)
            .unwrap();

        let origin_departure_time = departure_date
            .and_time(
                NaiveTime::from_num_seconds_from_midnight_opt(origin_departure_time_sec as u32, 0)
                    .unwrap(),
            )
            .and_local_timezone(tz)
            .unwrap();

        let terminal_arrival_time = departure_date
            .and_time(
                NaiveTime::from_num_seconds_from_midnight_opt(terminal_arrival_time_sec as u32, 0)
                    .unwrap(),
            )
            .and_local_timezone(tz)
            .unwrap();

        let result = TrainOrderRelatedData {
            train_number: query_result.train_number,
            departure_station: departure_station_info.station_name.clone(),
            arrival_station: arrival_station_info.station_name.clone(),
            departure_time: departure_time.to_rfc3339(),
            arrival_time: terminal_time.to_rfc3339(),

            origin_station: origin_station_info.station_name.clone(),
            terminal_station: terminal_station_info.station_name.clone(),
            origin_departure_time: origin_departure_time.to_rfc3339(),
            terminal_arrival_time: terminal_arrival_time.to_rfc3339(),

            name: query_result.name,
        };

        Ok(result)
    }

    async fn get_hotel_order_related_data(
        &self,
        order_id: OrderId,
    ) -> Result<HotelOrderRelatedData, RepositoryError> {
        let result = HotelOrderRelatedData::find_by_statement(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            r#"SELECT 
"hotel"."name" AS "hotel_name", 
"hotel"."uuid" AS "hotel_id", 
"person_info"."name" AS "name", 
"hotel_room_type"."type_name" AS "room_type" 
FROM "hotel_order"
    INNER JOIN "hotel"
        ON "hotel"."id" = "hotel_order"."hotel_id"
    INNER JOIN "person_info"
        ON "person_info"."id" = "hotel_order"."person_info_id"
    INNER JOIN "hotel_room_type"
                ON "hotel_room_type"."id" = "hotel_order"."hotel_room_type_id"
    WHERE "hotel_order"."id" = $1;"#,
            [order_id.to_db_value().into()],
        ))
        .one(&self.db)
        .await
        .context("failed to query related hotel order data")?
        .ok_or(RepositoryError::InconsistentState(anyhow!(
            "no related data for hotel order id: {}",
            order_id.to_db_value()
        )))?;

        Ok(result)
    }

    async fn get_dish_order_related_data(
        &self,
        order_id: OrderId,
        tz_offset_hour: i32,
    ) -> Result<DishOrderRelatedData, RepositoryError> {
        let query_result = DishOrderQueryResult::find_by_statement(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            r#"SELECT
"train"."number" AS "train_number",
"route"."departure_time" AS "departure_time",
"person_info"."name" AS "name",
"dish"."name" AS "dish_name",
"dish"."time" AS "dish_time"
FROM "dish_order"
    INNER JOIN "train_order"
        ON "train_order"."id" = "dish_order"."train_order_id"
    INNER JOIN "train_schedule"
        ON "train_schedule"."id" = "train_order"."train_schedule_id"
    INNER JOIN "train"
        ON "train"."id" = "train_schedule"."train_id"
    INNER JOIN "route"
        ON "route"."line_id" = "train_schedule"."line_id"
    INNER JOIN "person_info"
        ON "person_info"."id" = "dish_order"."person_info_id"
    INNER JOIN "dish"
        ON "dish"."id" = "dish_order"."dish_id"
    WHERE "dish_order"."id" = $1 AND "route"."order" = 0;"#,
            [order_id.to_db_value().into()],
        ))
        .one(&self.db)
        .await
        .context("failed to query related dish order data")?
        .ok_or(RepositoryError::InconsistentState(anyhow!(
            "no related data for dish order id: {}",
            order_id.to_db_value()
        )))?;

        let departure_time_info =
            DepartureTimeQueryResult::find_by_statement(Statement::from_sql_and_values(
                DatabaseBackend::Postgres,
                r#"SELECT
     "train_schedule"."departure_date" AS "departure_date",
    "train_schedule"."origin_departure_time" AS "origin_departure_time"
FROM "dish_order"
    INNER JOIN "train_order"
        ON "dish_order"."train_order_id" = "train_order"."id"
    INNER JOIN "train_schedule"
        ON "train_order"."train_schedule_id" = "train_schedule"."id"
WHERE "dish_order"."id" = $1"#,
                [order_id.to_db_value().into()],
            ))
            .one(&self.db)
            .await
            .context("failed to query related train schedule data")?
            .ok_or(RepositoryError::InconsistentState(anyhow!(
                "no related train schedule for dish order id: {}",
                order_id.to_db_value()
            )))?;

        let departure_time = departure_time_info
            .departure_date
            .and_time(
                NaiveTime::from_num_seconds_from_midnight_opt(
                    departure_time_info.origin_departure_time as u32,
                    0,
                )
                .unwrap(),
            )
            .and_local_timezone(FixedOffset::east_opt(tz_offset_hour * 3600).unwrap())
            .unwrap();

        let result = DishOrderRelatedData {
            train_number: query_result.train_number,
            departure_time: departure_time.to_rfc3339(),
            name: query_result.name,
            dish_name: query_result.dish_name,
            dish_time: query_result.dish_time,
        };

        Ok(result)
    }

    async fn get_takeaway_order_related_data(
        &self,
        order_id: OrderId,
        train_order_id: OrderId,
        tz_offset_hour: i32,
    ) -> Result<TakeawayOrderRelatedData, RepositoryError> {
        let query_result =
            TakeawayOrderQueryResult::find_by_statement(Statement::from_sql_and_values(
                DatabaseBackend::Postgres,
                r#"SELECT 
"train"."number" AS "train_number", 
"station"."id" AS "station_id", 
"takeaway_shop"."name" AS "shop_name", 
"person_info"."name" AS "name", 
"takeaway_dish"."name" AS "takeaway_name" 
FROM "takeaway_order"
    INNER JOIN "train_order"
        ON "train_order"."id" = "takeaway_order"."train_order_id"
    INNER JOIN "train_schedule"
        ON "train_schedule"."id" = "train_order"."train_schedule_id"
    INNER JOIN "train"
        ON "train"."id" = "train_schedule"."train_id"
    INNER JOIN "route"
        ON "route"."line_id" = "train_schedule"."line_id"
    INNER JOIN "person_info"
        ON "person_info"."id" = "takeaway_order"."person_info_id"
    INNER JOIN "takeaway_dish"
        ON "takeaway_dish"."id" = "takeaway_order"."takeaway_dish_id"
    INNER JOIN "takeaway_shop"
        ON "takeaway_dish"."takeaway_shop_id" = "takeaway_shop"."id"
    INNER JOIN "station"
        ON "takeaway_shop"."station_id" = "station"."id"
    WHERE "takeaway_order"."id" = $1 AND "route"."order" = 0;"#,
                [order_id.to_db_value().into()],
            ))
            .one(&self.db)
            .await
            .context("failed to query related takeaway order data")?
            .ok_or(RepositoryError::InconsistentState(anyhow!(
                "no related data for takeaway order id: {}",
                order_id.to_db_value()
            )))?;

        let (departure_date, route_info_list) = self
            .get_route_info_takeaway_order(order_id, train_order_id)
            .await?;

        if route_info_list.len() < 2 {
            panic!(
                "Invalid line for train order id: {}, less than 2 stations",
                train_order_id.to_db_value()
            );
        }

        let departure_time_info =
            DepartureTimeQueryResult::find_by_statement(Statement::from_sql_and_values(
                DatabaseBackend::Postgres,
                r#"SELECT
     "train_schedule"."departure_date" AS "departure_date",
    "train_schedule"."origin_departure_time" AS "origin_departure_time"
FROM "takeaway_order"
    INNER JOIN "train_order"
        ON "takeaway_order"."train_order_id" = "train_order"."id"
    INNER JOIN "train_schedule"
        ON "train_order"."train_schedule_id" = "train_schedule"."id"
WHERE "takeaway_order"."id" = $1"#,
                [order_id.to_db_value().into()],
            ))
            .one(&self.db)
            .await
            .context("failed to query related train schedule data")?
            .ok_or(RepositoryError::InconsistentState(anyhow!(
                "no related train schedule for takeaway order id: {}",
                order_id.to_db_value()
            )))?;

        let departure_time = departure_time_info
            .departure_date
            .and_time(
                NaiveTime::from_num_seconds_from_midnight_opt(
                    departure_time_info.origin_departure_time as u32,
                    0,
                )
                .unwrap(),
            )
            .and_local_timezone(FixedOffset::east_opt(tz_offset_hour * 3600).unwrap())
            .unwrap();

        let dish_route_info = route_info_list
            .iter()
            .find(|x| x.station_id == query_result.station_id)
            .ok_or(RepositoryError::InconsistentState(anyhow!(
                "takeaway station id: {} not found in route info, train order id: {}",
                query_result.station_id,
                train_order_id.to_db_value()
            )))?;

        let result = TakeawayOrderRelatedData {
            train_number: query_result.train_number,
            departure_time: departure_time.to_rfc3339(),
            station: dish_route_info.station_name.clone(),
            shop_name: query_result.shop_name,
            takeaway_name: query_result.takeaway_name,
            name: query_result.name,
            dish_time: departure_date
                .and_time(
                    NaiveTime::from_num_seconds_from_midnight_opt(
                        dish_route_info.arrival_time as u32,
                        0,
                    )
                    .unwrap(),
                )
                .and_local_timezone(FixedOffset::east_opt(tz_offset_hour * 3600).unwrap())
                .unwrap()
                .to_rfc3339(),
        };

        Ok(result)
    }

    async fn verify_train_order(
        &self,
        user_id: UserId,
        train_number: String,
        origin_departure_date: NaiveDate,
        origin_departure_time_second: i32,
    ) -> Result<bool, RepositoryError> {
        let order = crate::models::train_order::Entity::find()
            .from_raw_sql(Statement::from_sql_and_values(
                DatabaseBackend::Postgres,
                r#"SELECT * FROM "train_order"
    INNER JOIN "train_schedule"
        ON "train_order"."train_schedule_id" = "train_schedule"."id"
    INNER JOIN "train"
        ON "train_schedule"."train_id" = "train"."id"
    INNER JOIN "transaction"
        ON "train_order"."pay_transaction_id" = "transaction"."id"
    INNER JOIN "user"
        ON "transaction"."user_id" = "user"."id"
         WHERE "transaction"."user_id" = $1
           AND "train"."number" = $2
           AND "train_schedule"."departure_date" = $3
           AND "train_schedule"."origin_departure_time" = $4
           AND "train_order"."status" IN ('paid', 'ongoing', 'active')"#,
                [
                    user_id.to_db_value().into(),
                    train_number.into(),
                    origin_departure_date.into(),
                    origin_departure_time_second.into(),
                ],
            ))
            .one(&self.db)
            .await
            .inspect_err(|e| error!("failed to verify train order: {}", e))
            .map_err(|e| RepositoryError::Db(e.into()))?;

        Ok(order.is_some())
    }
}
