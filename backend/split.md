# Split

## User

### UserManagerService

内部:

Service:

- `UserService`
- `SessionManagerService`

Repository:

- `UserRepository`

### UserProfileService

内部:

Service:

- `SessionManagerService`

Repository:

- `UserRepository`

### PersonalInfoService

内部:

Service:

- `SessionManagerService`

Repository:

- `PersonalInfoRepository`

## Geo

### GeoApplicationService

内部:

Service:

- `GeoService`
- `StationService`

Repository:

- `CityRepository`
- `StationRepository`

## Train

### TrainQueryService

内部：

Service:

- `TrainScheduleService`
- `RouteService`

Repository:

- `TrainRepository`
- `TrainScheduleRepository`
- `RouteRepository`

外部：

Service:

- `StationService`
  - `Vec<Station>`，全部站点列表
- `SessionManagerService`
  - `bool`，检查 session id

Repository:

- `StationRepository`: `Vec<Station>`，全部站点列表 -> `StationService`，已实现

### TrainOrderService

内部：

Service:

- `TrainBookingService`
- `TrainSeatService`
- `TrainTypeConfigurationService`

Repository:

- `SeatAvailabilityRepository`

外部：

Service:

- `TrainScheduleService`
  - `get_station_arrival_time`: `train_schedule_id, departure_station_id` -> `sea_orm::prelude::DateTimeWithTimeZone`
- `TrainTypeConfigurationService`
  - `verify_train_number`: `TrainNumber<Unverified>` -> `TrainNumber<Verified>`
- `TransactionService`
  - `new_transaction`: `user_id, Vec<Box<dyn Order>>, atomic` -> `Uuid`
  - `refund_transaction`: `transaction_id, Vec<Box<dyn Order>>` -> `Uuid`
- `SessionManagerService`
  - `get_user_id_by_session`: `session_id` -> `user_id`

Repository:

- `OrderRepository`
  - `TrainOrder`，按 Order UUID
  - 更新`TrainOrder`状态到数据库
- `StationRepository`
  - `Station`，按 ID
- `PersonalInfoRepository`
  - `Vec<PersonalInfo>`，按用户 ID

### TrainDataService

内部:

Service:

Repository:

- `TrainRepository`
- `RouteRepository`

外部：

Service:

- `ObjectStorageService`
  - `put_object`
  - `get_object`
  - `delete_object`

Repository:

- `CityRepository`
  - `save_raw`
- `StationRepository`
  - `save_raw`
  - indirect:`takeaway_shop_repository.save_raw_takeaway`
- `DishRepository`
  - `save_raw_dish`
- `TakeawayShopRepository`
  - `save_raw_takeaway`

## Hotel

### HotelService

内部:

Service:

- `HotelRatingService`
- `HotelQueryService`
- `HotelBookingService`

Repository:

- `HotelRepository`
- `HotelRatingRepository`
- `OccupiedRoomRepository`

外部:

Service:

- `SessionManagerService`
  - `get_user_id_by_session`: `session_id` -> `UserId`
  - `get_session`: `session_id` -> `Session`

Repository:

- `OrderRepository`
  - `HotelOrder`，按 UUID
  - `Vec<HotelOrder>`，按用户 ID
  - 更新`HotelOrder`到数据库
- `CityRepository`
  - `Vec<City>`，按城市名称
- `StationRepository`
  - `Vec<Station>`，按车站名称
- `UserRepository`
  - `User`，按`UserId`

### HotelOrderService

内部:

Service:

- `HotelBookingService`

Repository:

- `HotelRepository`

外部:

Service:

- `TransactionService`
  - `new_transaction`: `user_id, Vec<Box<dyn Order>>, atomic` -> `Uuid`
  - `refund_transaction`: `transaction_id, Vec<Box<dyn Order>>` -> `Uuid`
- `SessionManagerService`
  - `get_user_id_by_session`: `session_id` -> `UserId`

Repository:

- `OrderRepository`
  - `HotelOrder`，按 UUID
  - 更新`HotelOrder`到数据库
- `PersonalInfoRepository`
  - `Vec<PersonalInfo>`，按 UserId

### HotelDataService

内部:

Service:

Repository:

- `HotelRepository`

外部:

Service:

- `ObjectStorageService`

Repository:

- `CityRepository`
- `StationRepository`

## Dish

### TrainDishApplicationService

内部:

Service:

Repository:

- `DishRepository`
- `TakeawayShopRepository`

外部:

Service:

- `TrainTypeConfigurationService`
  - `verify_train_number`
- `SessionManagerService`
  - `get_user_id_by_session`

Repository:

- `TrainRepository`
  - `find`: train_number -> `Train`
- `TrainScheduleRepository`
  - `find_by_train_id_and_origin_departure_time`: `TrainId, DateTime`
- `PersonalInfoRepository`
  - `find_by_user_id`: `UserId` -> `Vec<PersonalInfo>`
- `StationRepository`
  - `load`: `()` -> `Vec<Station>`，加载所有城市
- `TransactionRepository`
  - `find_by_user_id`： `UserId` -> `Vec<Transaction>`
  - 更新交易状态

### DishQueryService

内部:

Service:

Repository:

- `DishRepository`
- `TakeawayShopRepository`

外部:

Service:

- `SessionManagerService`
  - `get_user_id_by_session`: `session_id` -> `UserId`
- `TrainScheduleService`
  - `get_terminal_arrival_time`: `TrainNumber, DateTime` -> `DateTimeWithTimeZone`
- `TrainTypeConfigurationService`
  - `verify_train_number`
- `StationService`
  - `get_stations`: `Vec<Station>`
- `OrderService`
  - `verify_train_order`: `UserId, TrainNumber, origin_departure_time`

Repository:

- `TrainRepository`
  - `find_by_train_number`

## Order

### Order

### TransactionApplicationService

内部:

Service:

- `TransactionService`
- `OrderService`
- `OrderStatusManagerService`

Repository:

- `OrderStatusManagerService`
- `OrderRepository`

外部:

Service:

- `SessionManagerService`
  - `get_user_id_by_session`
- `UserService`
  - `verify_password`: `User, user_password: String`
  - `verify_payment_password`: `User, payment_password: String`
  - `set_payment_password`: `UserId, Option<PaymentPassword>`
  - `clear_wrong_payment_password_tried`: `UserId`

Repository:

- `UserRepository`
  - `find`: `UserId` -> `User`

## Message

### MessageApplicationService

内部:

Service:

- `MessageService`
- `MessageListenerService`

Repository:

- `NotifyRepository`

外部:

Service:

- `OrderService`
  - `convert_order_to_dto`
- `SessionManagerService`
  - `get_user_id_by_session`

Repository:
