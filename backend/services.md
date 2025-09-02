# Services

> We will never forget those who fell in the defense of Malevelon Creek.

下面每小节表示一个拆分后的微服务，每个微服务中包含：

- Application Service：向外界提供 API 服务，与拆分前相同
- Internal Service：这些服务原本属于 Domain Service，在拆分为微服务后，其它微服务需要跨越微服务边界调用其中的内容，故需要在内部提供 API 服务，这些 API 不应暴露给外界
- Domain Service：拆分后仍属于 Domain Service，且不需要为其它微服务提供内部 API
  - 函数名前用`+`标注的，表示该函数需要跨微服务调用，这些函数应当被移动到合适的 Internal Service 中，例如`+clear_wrong_payment_password_tried(user_id: UserId,) -> Result<()>;`
- Repository：微服务中应当包含的仓储
  - 每个仓储下都列出了该仓储需要访问的数据库表，`A -> B`表示表`A`含有参照表`B`的外键
  - 用`++`标注的表，表示拆分后该表将不再属于当前微服务，需要跨微服务访问，例如：`++station`
- Tables：该微服务的数据库应当包含的表
- Depends：该微服务中仓储的运行依赖于其它微服务中的数据库中的表
- Duplicates：为了支持跨微服务的数据库连接操作，使用非规范化设计，冗余存储需要的表，并根据领域事件进行同步。

拆分前的数据库表：

```text
                    List of relations
 Schema |          Name           | Type  |    Owner
--------+-------------------------+-------+--------------
 public | city                    | table | swiftjourney
 public | dish                    | table | swiftjourney
 public | dish_order              | table | swiftjourney
 public | hotel                   | table | swiftjourney
 public | hotel_order             | table | swiftjourney
 public | hotel_rating            | table | swiftjourney
 public | hotel_room_type         | table | swiftjourney
 public | message                 | table | swiftjourney
 public | occupied_room           | table | swiftjourney
 public | occupied_seat           | table | swiftjourney
 public | person_info             | table | swiftjourney
 public | route                   | table | swiftjourney
 public | seaql_migrations        | table | swiftjourney
 public | seat_availability       | table | swiftjourney
 public | seat_type               | table | swiftjourney
 public | seat_type_in_train_type | table | swiftjourney
 public | seat_type_mapping       | table | swiftjourney
 public | station                 | table | swiftjourney
 public | takeaway_dish           | table | swiftjourney
 public | takeaway_order          | table | swiftjourney
 public | takeaway_shop           | table | swiftjourney
 public | train                   | table | swiftjourney
 public | train_order             | table | swiftjourney
 public | train_schedule          | table | swiftjourney
 public | train_type              | table | swiftjourney
 public | transaction             | table | swiftjourney
 public | user                    | table | swiftjourney
(27 rows)
```

Events：

领域事件以及其触发途径，接收到领域事件后，应当同步相关的表。

- `CityUpdatedEvent`
  - `load_city`：`base/src/infrastructure/application/service/train_data.rs:165`
- `StationUpdatedEvent()`
  - `load_station`：`base/src/infrastructure/application/service/train_data.rs:188`
- `UserUpdatedEvent()`
  - `set_profile`：`base/src/infrastructure/application/service/user_profile.rs:162`
  - `register`：`base/src/infrastructure/application/service/user_manager.rs:101`
  - `update_password`：`base/src/infrastructure/application/service/user_manager.rs:215`
  - `set_payment_password`：`base/src/infrastructure/application/service/transaction.rs:211`
  - BUG?：`wrong_payment_password_tried`值似乎并未更新
- `PersonalInfoUpdatedEvent()`
  - `set_personal_info`：`base/src/infrastructure/application/service/personal_info.rs:140`
- `TrainUpdatedEvent()`
  - `save_raw_train_number`：`base/src/infrastructure/repository/train.rs:953`
- `TrainScheduleUpdatedEvent()`
  - `auto_plan_schedule`：`base/src/infrastructure/service/train_schedule.rs:288`
- `RouteUpdatedEvent()`
  - `save_raw_train_number`：`base/src/infrastructure/repository/train.rs:953`
- `seat_type + seat_type_mapping: SeatTypeUpdatedEvent()`
  - `load_train_type`：`base/src/infrastructure/application/service/train_data.rs:218`
- `HotelUpdatedEvent()`
  - `load_hotel`：`base/src/infrastructure/application/service/hotel_data.rs:62`
  - `new_comment`：`base/src/infrastructure/application/service/hotel.rs:126`
  - `process_hotel_orders`：`base/src/infrastructure/application/service/hotel_order.rs:336`
- `HotelRoomTypeUpdatedEvent()`
  - `load_hotel`：`base/src/infrastructure/application/service/hotel_data.rs:62`
- `DishUpdatedEvent()`
  - `load_dish_takeaway`：`base/src/infrastructure/application/service/train_data.rs:263`
- `TakeawayDishUpdatedEvent()`
  - `load_dish_takeaway`：`base/src/infrastructure/application/service/train_data.rs:263`

## User(Err: 90XXX)

Service:

- Application Service

  - `UserProfileService`
    - `get_profile(query: UserProfileQuery) -> Result<UserProfileDTO>;`
    - `set_profile(command: SetUserProfileCommand) -> Result<()>;`
  - `PersonalInfoService`
    - `get_personal_info(query: PersonalInfoQuery) -> Result<Vec<PersonalInfoDTO>>;`
    - `set_personal_info(command: SetPersonalInfoCommand) -> Result<()>;`
  - `UserManagerService`
    - `register(command: UserRegisterCommand) -> Result<()>;`
    - `login(command: UserLoginCommand) -> Result<SessionId>;`
    - `logout(command: UserLogoutCommand) -> Result<()>;`
    - `update_password(command: UserUpdatePasswordCommand) -> Result<()>;`

- Internal Service
  - `+verify_password(user_id: UserId, raw_password: String) -> Result<bool>;`
  - `+verify_payment_password(user_id: UserId, raw_payment_password: String,) -> Result<bool>;`
  - `+set_payment_password(user_id: UserId, payment_password: Option<PaymentPassword>,) -> Result<()>;`
  - `+clear_wrong_payment_password_tried(user_id: UserId,) -> Result<()>;`
  - `+get_session(session_id: SessionId) -> Result<Option<Session>>;` + `+verify_session_id(session_id_str: &str) -> Result<bool>;` + `+get_user_id_by_session(session_id: SessionId) -> Result<Option<UserId>>;`
  - `+get_user_info(user_id: UserId) -> Result<Vec<PersonalInfo> + User + UserInfo>`
  - `+db_get_user_info() -> Result<Vec<models::User>>`
  - `+db_get_personal_info() -> Result<Vec<models::PersonalInfo>>`
- Domain Service
  - `UserService`
    - `register(username: Username, raw_password: RawPassword, name: RealName, phone: Phone, identity_card_id: IdentityCardId,) -> Result<()>;`
    - `delete(phone: Phone) -> Result<()>;`
    - `+verify_password(user: &User, raw_password: String,) -> Result<()>;`
    - `+verify_payment_password(user: &User, raw_payment_password: String,) -> Result<()>;`
    - `set_password(user_id: UserId, raw_password: String,) -> Result<()>;`
    - `+set_payment_password(user_id: UserId, payment_password: Option<PaymentPassword>,) -> Result<()>;`
    - `set_wrong_payment_password_tried(user_id: UserId, password_attempts: PasswordAttempts,) -> Result<()>;`
    - `+clear_wrong_payment_password_tried(user_id: UserId,) -> Result<()>;`
    - `increment_wrong_payment_password_tried(user_id: UserId,) -> Result<()>;`
    - `set_user_info(user_id: UserId, user_info: UserInfo,) -> Result<()>;`
  - `SessionManagerService`
    - `create_session(user_id: UserId) -> Result<Session>;`
    - `delete_session(session: Session) -> Result<()>;`
    - `+get_session(session_id: SessionId) -> Result<Option<Session>>;`
    - `+get_user_id_by_session(session_id: SessionId) -> Result<Option<UserId>>;`
    - `+verify_session_id(session_id_str: &str) -> Result<bool>;`

Repository:

- `UserRepository`
  - `user`
- `PersonalInfoRepository`
  - `personal_info` -> `user`

Tables:

- `user`
- `personal_info`

## Geo(Err: 91XXX)

Service:

- Application Service
  - `GeoApplicationService`
    - `get_city_info() -> Result<CityInfoDTO>;`
    - `get_city_station_info() -> Result<CityStationInfoDTO>;`

Internal Service

- `+get_cities() -> Result<Vec<City>>;`
- `+get_stations() -> Result<Vec<Station>>;`
- `+db_get_cities() -> Result<Vec<models::City>>;`
- `+db_get_stations() -> Result<Vec<models::Station>>;`
- `+save_city_province_map(city_province_map: HashMap<String, String>) -> Result<()>`
- `+save_station_city_map(station_city_map: HashMap<String, String>) -> Result<()>`

- Domain Service
  - `GeoService`
    - `get_city_map() -> Result<HashMap<ProvinceName, Vec<City>>>;`
    - `get_city_by_name(name: &str) -> Result<Option<City>>;`
    - `add_city(city: City) -> Result<CityId>;`
    - `remove_city(city: City) -> Result<()>;`
    - `modify_city(city_id: CityId, city_name: CityName, province: ProvinceName) -> Result<()>;`
  - `StationService`
    - `+get_stations() -> Result<Vec<Station>>;`
    - `get_station_by_city(city_id: CityId) -> Result<Vec<Station>>;`
    - `get_station_by_name(station_name: String) -> Result<Option<Station>>;`
    - `add_station(station_name: String, city_name: String) -> Result<StationId>;`
    - `modify_station(station_id: StationId, station_name: String, city_name: String) -> Result<()>;`
    - `delete_station(station: Station) -> Result<()>;`
    - `get_station_by_city_name(city_name: &str) -> Result<Vec<Station>>;`
    - `station_pairs_by_city(from_city: &str, to_city: &str) -> Result<Vec<(StationId, StationId)>>;`

Repository:

- `CityRepository`
  - `city`
- `StationRepository`
  - `city`
  - `station` -> `city`

Tables:

- `city`
- `station`

## Train(Err: 92XXX)

Service:

- Application Service
  - `TrainQueryService`
    - `query_train(cmd: TrainScheduleQueryCommand) -> Result<TrainQueryResponseDTO>;`
    - `query_direct_trains(cmd: DirectTrainQueryCommand) -> Result<DirectTrainQueryDTO>;`
    - `query_transfer_trains(cmd: TransferTrainQueryCommand) -> Result<TransferTrainQueryDTO>;`
  - `TrainOrderService`
    - `process_train_order_packs(session_id: String, order_packs: Vec<OrderPackDTO>) -> Result<TransactionInfoDTO>;`
- Internal Service
  - `+get_train_by_number(train_number: String) -> Result<Train>`
  - `+get_train_schedule_by_train_id_and_origin_departure_time(train_id: TrainId, origin_departure_time: String) -> Result<Option<TrainSchedule>>`
- Domain Service
  - `TrainScheduleService`
    - `add_schedule(train_id: TrainId, date: NaiveDate) -> Result<()>;`
    - `get_schedules(date: NaiveDate) -> Result<Vec<TrainSchedule>>;`
    - `get_schedule_by_train_number_and_date(train_number: String, departure_date: NaiveDate) -> Result<Option<TrainSchedule>>;`
    - `auto_plan_schedule(begin_date: NaiveDate, days: i32) -> Result<()>;`
    - `auto_plan_schedule_daemon(days: i32);`
    - `direct_schedules(date: chrono::NaiveDate, pairs: &[(StationId, StationId)]) -> Result<Vec<(TrainSchedule, StationId, StationId)>>;`
    - `transfer_schedules(date: chrono::NaiveDate, pairs: &[(StationId, StationId)]) -> Result<Vec<(Vec<TrainScheduleId>, StationId, StationId, Option<StationId>)>>;`
    - `+get_station_arrival_time(train_schedule_id: TrainScheduleId, station_id: StationId) -> Result<sea_orm::prelude::DateTimeWithTimeZone>;`
    - `+get_terminal_arrival_time(train_number: TrainNumber<Verified>, origin_departure_time: DateTimeWithTimeZone) -> Result<DateTimeWithTimeZone>;`
  - `TrainBookingService`
    - `booking_ticket(order_uuid: Uuid) -> Result<()>;`
    - `cancel_ticket(order_uuid: Uuid) -> Result<()>;`
    - `booking_group(order_uuid_list: Vec<Uuid>, atomic: bool) -> Result<Vec<TrainOrder>>;`
  - `TrainSeatService`
    - `available_seats_count(seat_availability_id: SeatAvailabilityId) -> Result<u32>;`
    - `reserve_seat(train_schedule: &mut TrainSchedule, station_range: StationRange<Verified>, seat_type: SeatType, seat_location_info: SeatLocationInfo, personal_info_id: PersonalInfoId) -> Result<Seat>;`
    - `free_seat(seat_availability_id: SeatAvailabilityId, seat: Seat) -> Result<()>;`
  - `TrainTypeConfigurationService`
    - `verify_seat_type_name(train_id: TrainId, seat_type_name: SeatTypeName<Unverified>) -> Result<SeatTypeName<Verified>>;`
    - `+verify_train_number(train_number: TrainNumber<Unverified>) -> Result<TrainNumber<Verified>>;`
    - `verify_train_type(train_type: TrainType<Unverified>) -> Result<TrainType<Verified>>;`
    - `get_seat_id_map(train_id: TrainId) -> Result<HashMap<SeatTypeName<Verified>, Vec<(SeatId, SeatLocationInfo)>>>;`
    - `get_trains() -> Result<Vec<Train>>;`
    - `get_train_by_number(train_number: TrainNumber<Verified>) -> Result<Train>;`
    - `add_train_type(train_number: TrainNumber<Verified>, train_type: TrainType<Verified>, seat_configuration: Vec<SeatType>, default_route_id: RouteId, default_origin_departure_time: i32) -> Result<TrainId>;`
    - `modify_train_type(train_id: TrainId, train_number: TrainNumber<Verified>, train_type: TrainType<Verified>, seat_configuration: Vec<SeatType>, default_route_id: RouteId, default_origin_departure_time: i32) -> Result<()>;`
    - `remove_train_type(train: Train) -> Result<()>;`
  - `TrainDataService`
    - `is_debug_mode() -> bool;`
    - `load_city(command: LoadCityCommand) -> Result<()>;`
    - `load_station(command: LoadStationCommand) -> Result<()>;`
    - `load_train_type(command: LoadTrainTypeCommand, db: &DatabaseConnection) -> Result<()>;`
    - `load_train_number(command: LoadTrainNumberCommand, db: &DatabaseConnection) -> Result<()>;`
    - `load_dish_takeaway(command: LoadDishTakeawayCommand, db: &DatabaseConnection) -> Result<()>;`
  - `RouteService`
    - `get_route_map() -> Result<RouteGraph>;`
    - `add_route(stops: Vec<Stop>) -> Result<RouteId>;`
    - `get_routes() -> Result<Vec<Route>>;`

Repository:

- `TrainRepository`
  - `train` -> `train_type`
  - `train_type`
  - `seat_type`
  - `seat_type_in_train_type` -> `train_type` + `seat_type`
  - `seat_type_mapping` -> `train_type` + `seat_type`
  - `route` -> `++station`
- `TrainScheduleRepository`
  - `train_schedule` -> `train`
  - `seat_availability` -> `train_schedule` + `seat_type` + `++station`
- `RouteRepository`
  - `route` -> `++station`
  - `++station`
- `SeatAvailabilityRepository`
  - `occupied_seat` -> `++person_info`

Tables:

- `train`
- `train_schedule`
- `train_type`
- `seat_type`
- `seat_type_in_train_type`
- `seat_type_mapping`
- `seat_availability`
- `occupied_seat`
- `route`
- `+station`
- `+person_info`

Depends:

- `station`
  - `TrainRepository`
    - `route`：外键
  - `TrainScheduleRepository`
    - `seat_availability`：外键
  - `RouteRepository`
    - `route`：外键
    - `save_raw`：获取`station_name` -> `station_id`的映射，用途：存储时将站点名称转化为 ID
- `person_info`
  - `SeatAvailabilityRepository`
    - `occupied_seat`：外键

Duplicates:

- `station`
  - `StationUpdatedEvent()`
    - `load_station`：`base/src/infrastructure/application/service/train_data.rs:188`
- `person_info`
  - `PersonalInfoUpdatedEvent(user_id)`
    - `set_personal_info`：`base/src/infrastructure/application/service/personal_info.rs:140`

## Hotel(Err: 93XXX)

Service:

- Application Service
  - `HotelService`
    - `get_quota(query: QuotaQuery) -> Result<HotelCommentQuotaDTO>;`
    - `new_comment(command: NewCommentCommand) -> Result<()>;`
    - `query_hotels(query: HotelQuery) -> Result<Vec<HotelGeneralInfoDTO>>;`
    - `query_hotel_info(query: HotelInfoQuery) -> Result<HotelDetailInfoDTO>;`
    - `query_hotel_order_info(query: HotelOrderInfoQuery) -> Result<HashMap<String, HotelRoomDetailInfoDTO>>;`
  - `HotelOrderService`
    - `process_hotel_orders(session_id: String, hotel_orders: HotelOrderRequestsDTO) -> Result<TransactionInfoDTO>;`
  - `HotelDataService`
    - `is_debug_mode() -> bool;`
    - `load_hotel(command: LoadHotelCommand, db: &DatabaseConnection) -> Result<()>;`
- Domain Service
  - `HotelRatingService`
    - `get_hotel_rating(hotel_uuid: Uuid) -> Result<Rating>;`
    - `get_hotel_comment_quota(hotel_uuid: Uuid, user_id: UserId) -> Result<i32>;`
    - `get_current_comment_count(hotel_uuid: Uuid, user_id: UserId) -> Result<i32>;`
    - `get_comments(hotel_uuid: Uuid) -> Result<Vec<HotelRating>>;`
    - `add_comment(hotel_uuid: Uuid, user_id: UserId, rating: Rating, text: String) -> Result<()>;`
  - `HotelQueryService`
    - `find_hotels_by_target(target: &str, target_type: &TargetType, search_term: Option<&str>) -> Result<Vec<Hotel>>;`
    - `calculate_minimum_prices(hotels: &[Hotel], date_range: Option<&HotelDateRange>) -> Result<HashMap<HotelId, Decimal>>;`
    - `query_hotels(target: &str, target_type: &TargetType, search_term: Option<&str>, date_range: Option<&HotelDateRange>) -> Result<Vec<HotelGeneralInfoDTO>>;`
  - `HotelBookingService`
    - `get_available_room(hotel_id: HotelId, booking_date_range: HotelDateRange) -> Result<HashMap<HotelRoomTypeId, HotelRoomStatus>>;`
    - `booking_hotel(order_uuid: Uuid) -> Result<()>;`
    - `cancel_hotel(order_uuid: Uuid) -> Result<()>;`
    - `booking_group(order_uuid_list: Vec<Uuid>, atomic: bool) -> Result<Vec<HotelOrder>>;`

Repository:

- `HotelRepository`
  - `hotel` -> `++city` + `++station`
  - `hotel_room_type` -> `hotel`
  - `++city`
  - `++station`
- `HotelRatingRepository`
  - `hotel_rating` -> `++user` + `hotel`
- `OccupiedRoomRepository`
  - `occupied_room` -> `hotel` + `hotel_room_type` + `++person_info`

Tables:

- `hotel`
- `hotel_room_type`
- `hotel_rating`
- `occupied_room`
- `+city`
- `+station`
- `+user`
- `+person_info`

Depends:

- `city`
  - `HotelRepository`
    - `hotel`：外键
    - 连接，用途：构建`Hotel`实体
    - `save_raw_hotel`
      - +加载所有城市，用途：构建`Hotel`实体 -> `+get_cities`
- `station`
  - `HotelRepository`
    - `hotel`：外键
    - 连接，用途：构建`Hotel`实体
    - `save_raw_hotel`
      - 加载所有站点，用途：构建`Hotel`实体
- `user`
  - `HotelRatingRepository`
    - `hotel_rating`：外键
- `person_info`
  - `OccupiedRoomRepository`
    - `occupied_room`：外键

Duplicates:

- `city`
  - `CityUpdatedEvent`
    - `load_city`：`base/src/infrastructure/application/service/train_data.rs:165`
- `station`
  - `StationUpdatedEvent()`
    - `load_station`：`base/src/infrastructure/application/service/train_data.rs:188`
- `user`
  - `UserUpdatedEvent()`
    - `set_profile`：`base/src/infrastructure/application/service/user_profile.rs:162`
    - `register`：`base/src/infrastructure/application/service/user_manager.rs:101`
    - `update_password`：`base/src/infrastructure/application/service/user_manager.rs:215`
    - `set_payment_password`：`base/src/infrastructure/application/service/transaction.rs:211`
    - BUG?：`wrong_payment_password_tried`值似乎并未更新
- `person_info`
  - `PersonalInfoUpdatedEvent(user_id)`
    - `set_personal_info`：`base/src/infrastructure/application/service/personal_info.rs:140`

## Dish(Err: 94XXX)

Service:

- Application Service
  - `TrainDishApplicationService`
    - `order_dish(command: OrderTrainDishCommand) -> Result<TransactionInfoDTO>;`
  - `DishQueryService`
    - `query_dish(query: DishQueryDTO, session_id: String) -> Result<TrainDishInfoDTO>;`
- Internal Service
  - `save_raw_dish`
  - `save_raw_takeaway`

Repository:

- `DishRepository`
  - `dish` -> `++train`
  - `++train`
- `TakeawayShopRepository`
  - `takeaway_dish` -> `takeaway_shop`
  - `takeaway_shop` -> `++station`
  - `++route`

Tables:

- `dish`
- `takeaway_dish`
- `takeaway_shop`
- `+train`
- `+station`
- `+route`

Depends:

- `train`
  - `DishRepository`
    - `dish`：外键
    - `find_by_train_number`：连接，用途：按 train_number 筛选
    - `save_raw_dish`：加载所有车次，用途：将车次号转换为车次 ID
- `station`
  - `TakeawayShopRepository`
    - `takeaway_shop`：外键
    - `save_raw_takeaway`：加载所有车站，用途：将车站名转化为车站 ID
- `route`
  - `TakeawayShopRepository`
    - `find_by_train_route`：连接，按`line_id`筛选

Duplicates:

- `train`
  - `TrainUpdatedEvent()`
    - `save_raw_train_number`：`base/src/infrastructure/repository/train.rs:953`
- `station`
  - `StationUpdatedEvent()`
    - `load_station`：`base/src/infrastructure/application/service/train_data.rs:188`
- `route`
  - `RouteUpdatedEvent()`
    - `save_raw_train_number`：`base/src/infrastructure/repository/train.rs:953`

## Order(Err: 95XXX)

Service:

- Application Service
  - `TransactionApplicationService`
    - `recharge(command: RechargeCommand) -> Result<()>;`
    - `query_balance(query: BalanceQuery) -> Result<BalanceInfoDTO>;`
    - `query_transactions(query: TransactionQuery) -> Result<Vec<TransactionInfoDTO>>;`
    - `set_payment_password(command: SetPaymentPasswordCommand) -> Result<()>;`
    - `pay_transaction(command: PayTransactionCommand) -> Result<()>;`
    - `generate_debug_transaction(command: GenerateDebugTransactionCommand) -> Result<TransactionInfoDTO>;`
    - `query_transaction_details(query: TransactionDetailQuery) -> Result<Vec<TransactionDataDto>>;`
    - `cancel_order(command: CancelOrderCommand) -> Result<()>;`
- Internal Service
  - `+new_transaction(user_id: UserId, orders: Vec<Box<dyn Order>>, atomic: bool) -> Result<Uuid>;`
  - `+refund_transaction(transaction_id: Uuid, to_refund_orders: &[Box<dyn Order>]) -> Result<Uuid>;`
  - `+convert_order_to_dto(order: Box<dyn Order>) -> Result<OrderInfoDto>;`
  - `+verify_train_order(user_id: UserId, train_number: String, origin_departure_time: DateTimeWithTimeZone) -> Result<bool>;`
  - `update_order(order: Vec<Box<dyn Order>>)`
  - `get_order_list_by_user_id(user_id: UserId) -> Vec<Box<dyn Order>>`
- Domain Service
  - `TransactionService`
    - `recharge(user_id: UserId, amount: TransactionAmountAbs) -> Result<Uuid>;`
    - `get_balance(user_id: UserId) -> Result<Decimal>;`
    - `+new_transaction(user_id: UserId, orders: Vec<Box<dyn Order>>, atomic: bool) -> Result<Uuid>;`
    - `pay_transaction(transaction_id: Uuid) -> Result<()>;`
    - `+refund_transaction(transaction_id: Uuid, to_refund_orders: &[Box<dyn Order>]) -> Result<Uuid>;`
    - `convert_transaction_to_dto(transaction: Transaction) -> Result<TransactionDataDto>;`
  - `OrderService`
    - `+convert_order_to_dto(order: Box<dyn Order>) -> Result<OrderInfoDto>;`
    - `+verify_train_order(user_id: UserId, train_number: String, origin_departure_time: DateTimeWithTimeZone) -> Result<bool>;`
  - `OrderStatusManagerService`
    - `notify_status_change(transaction_uuid: Uuid, atomic: bool, orders: &[&dyn Order], new_status: OrderStatus);`
    - `order_status_daemon();`

Repository:

- `OrderRepository`
  - `transaction` -> `++user`
  - `train_order` -> `++train_schedule` + `++seat_type` + `++station` + `++person_info` + `transaction`
  - `hotel_order` -> `++hotel` + `++hotel_room_type` + `++person_info` + `transaction`
  - `dish_order` -> `train_order` + `++dish` + `++person_info` + `transaction`
  - `takeaway_order` -> `train_order` + `++takeaway_dish` + `++person_info` + `transaction`
  - `++seat_type`
  - `++seat_type_mapping`
  - `++train_schedule`
  - `++train`
  - `++user`

Tables:

- `transaction`
- `train_order`
- `hotel_order`
- `dish_order`
- `takeaway_order`
- `+user`
- `+train`
- `+train_schedule`
- `+seat_type`
- `+seat_type_mapping`
- `+station`
- `+person_info`
- `+hotel`
- `+hotel_room_type`
- `+dish`
- `+takeaway_dish`

Depends:

- `user`
  - `transaction`：外键
  - `verify_*_order`：，连接，用途：`user_id -> person_info -> *_order`
- `train`
  - `find_train_order_by_uuid`：连接，用途：根据`train_order`中的`train_schedule_id`获取对应的`seat_type_mapping`
  - `load_all_active_orders`：加载所有车次，用途：获取`train_id`对应的`train_type_id`
  - `get_train_order_related_data`：连接，用途：根据`train_order_id`获取车次号
  - `get_dish_order_related_data`：连接，用途：根据`dish_order`获取车次号
  - `get_takeaway_order_related_data`：连接，用途：根据`takeaway_order`获取车次号
  - `verify_train_order`：检查用户是否有对应车次的订单
- `train_schedule`
  - `find_train_order_by_uuid`：连接，用途：`train_order` -> `train_schedule_id` -> `train_id`
  - `load_all_active_orders`：加载所有车次计划，用途：`train_schedule_id` -> `train_id` + `train_type_id`
  - `get_route_info_train_order`：连接，用途：`train_order` -> `route`，获取`origin_departure_time`，用于从相对出发时间计算实际出发时间
  - `get_route_info_takeaway_order`：
    - 连接，用途：`takeaway_order_id` -> `route` -> `station`
    - 按 ID 加载，用于从相对出发时间计算实际出发时间
  - `get_train_order_related_data`：连接，用途：`train_order_id` -> `train_schedule` -> `route`
  - `get_dish_order_related_data`：
    - 连接，用途：`dish_order_id` -> `train_order` -> `train_schedule` -> `train` -> `route`
    - 连接，用途：`dish_order_id` -> `train_order` -> `train_schedule`获取出发日期、时间
  - `get_takeaway_order_related_data`
    - 连接，用途：`takeaway_order_id` -> `train_schedule` -> `train` -> `route`获取`TrainNumber`、`station`、`takeaway_shop`信息
    - 连接，用途：`takeaway_order_id` -> `train_schedule`获取`departure_date`、`origin_departure_time`
  - `verify_train_order`：连接，用途：检查对应车次计划的订单是否存在
- `seat_type`
  - `find_train_order_by_uuid`：加载所有座位类型，用途：`seat_type_id -> seat_type`
  - `load_all_active_orders`：加载所有座位类型，用途：`seat_type_id -> seat_type`
- `seat_type_mapping`
  - `find_train_order_by_uuid`：连接，用途：`train_order_uuid -> seat_type_mapping`
  - `load_all_active_orders`：加载所有座位类型映射，用途：`train_type_id -> seat_type_id -> seat_id -> seat_type_mapping`
- `station`
  - `get_route_info_train_order`：连接，用途：`train_order_id -> train_schedule -> route -> station`
  - `get_route_info_takeaway_order`：连接，用途：`takeaway_order_id -> train_schedule -> route -> station`
  - `get_takeaway_order_related_data`：连接，用途：`takeaway_order_id -> train_order -> train_schedule -> train -> route -> person_info -> takeaway_dish -> takeaway_shop -> station`
- `person_info`
  - `find_*_order_by_userid`：连接，用途：根据用户 ID 筛选其绑定的所有身份的订单
  - `get_*_order_related_data`：连接，用途：`*_order_id -> person_info`
- `hotel`
  - `get_hotel_order_related_data`：连接，用途：`hotel_order_id -> hotel`
- `hotel_room_type`
  - `get_hotel_order_related_data`：连接，用途：`hotel_order_id -> hotel_room_type`
- `dish`
  - `get_dish_order_related_data`：连接，用途：`dish_order_id -> dish`
- `takeaway_dish`
  - `get_takeaway_order_related_data`：连接，用途：`takeaway_order_id -> takeaway_dish`

Duplicates:

- `user`
  - `UserUpdatedEvent()`
    - `set_profile`：`base/src/infrastructure/application/service/user_profile.rs:162`
    - `register`：`base/src/infrastructure/application/service/user_manager.rs:101`
    - `update_password`：`base/src/infrastructure/application/service/user_manager.rs:215`
    - `set_payment_password`：`base/src/infrastructure/application/service/transaction.rs:211`
    - BUG?：`wrong_payment_password_tried`值似乎并未更新
- `train`
  - `TrainUpdatedEvent()`
    - `save_raw_train_number`：`base/src/infrastructure/repository/train.rs:953`
- `train_schedule`
  - `TrainScheduleUpdatedEvent()`
    - `auto_plan_schedule`：`base/src/infrastructure/service/train_schedule.rs:288`
- `seat_type`
  - `SeatTypeUpdatedEvent()`
    - `load_train_type`：`base/src/infrastructure/application/service/train_data.rs:218`
- `seat_type_mapping`
  - `SeatTypeUpdatedEvent()`
    - `load_train_type`：`base/src/infrastructure/application/service/train_data.rs:218`
- `station`
  - `StationUpdatedEvent()`
    - `load_station`：`base/src/infrastructure/application/service/train_data.rs:188`
- `person_info`
  - `PersonalInfoUpdatedEvent(user_id)`
    - `set_personal_info`：`base/src/infrastructure/application/service/personal_info.rs:140`
- `hotel`
  - `HotelUpdatedEvent()`
    - `load_hotel`：`base/src/infrastructure/application/service/hotel_data.rs:62`
    - `new_comment`：`base/src/infrastructure/application/service/hotel.rs:126`
    - `process_hotel_orders`：`base/src/infrastructure/application/service/hotel_order.rs:336`
- `hotel_room_type`
  - `HotelRoomTypeUpdatedEvent()`
    - `load_hotel`：`base/src/infrastructure/application/service/hotel_data.rs:62`
- `dish`
  - `DishUpdatedEvent()`
    - `load_dish_takeaway`：`base/src/infrastructure/application/service/train_data.rs:263`
- `takeaway_dish`
  - `TakeawayDishUpdatedEvent()`
    - `load_dish_takeaway`：`base/src/infrastructure/application/service/train_data.rs:263`

## ObjectStorageService(Err: 96XXX)

- Internal Service
  - `+put_object(object_category: ObjectCategory, content_type: &str, object: Vec<u8>) -> Result<Uuid>;`
  - `+get_object(object_category: ObjectCategory, object_id: Uuid) -> Result<ObjectInfo>;`
  - `+delete_object(object_category: ObjectCategory, object_id: Uuid) -> Result<()>;`

## Message(Unused)(Err: 97XXX)

如下服务似乎未在其它代码中使用，故可能不需要拆分。

Service:

- `MessageService`
  - `convert_notify_to_dto(notify: Box<dyn Notify>) -> Result<NotifyDTO>;`
  - `send_to_user(user_id: UserId, notify: Box<dyn Notify>) -> Result<()>;`
  - `get_history(user_id: UserId) -> Result<Vec<Box<dyn Notify>>>;`
- `MessageListenerService`
  - `add_listener(user_id: UserId, listener: Box<dyn MessageListener>);`
  - `find_listener_by_user_id(user_id: UserId) -> Vec<Box<dyn MessageListener>>;`
  - `check_session();`

Repository:

- `NotifyRepository`
