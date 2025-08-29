use serde_json;
use shared::data::{
    DishInfo, HotelComment, HotelInfo, HotelRoomType, RouteStationInfo, StationDataItem,
    TakeawayDishInfo, TrainNumberInfoItem, TrainTypeInfoItem,
};
use std::collections::HashMap;

#[test]
fn train_number_snapshot() {
    let item = TrainNumberInfoItem {
        train_number: "G1234".to_string(),
        train_type: "G".to_string(),
        origin_departure_time: 8 * 60 + 30, // 08:30
        route: vec![
            RouteStationInfo {
                order: 1,
                station: "北京南".to_string(),
                arrival_time: 0,
                departure_time: 8 * 60 + 30,
            },
            RouteStationInfo {
                order: 2,
                station: "天津".to_string(),
                arrival_time: 9 * 60 + 5,
                departure_time: 9 * 60 + 8,
            },
        ],
    };

    insta::with_settings!({sort_maps => true}, {
        insta::assert_json_snapshot!(item, {});
    });
}

#[test]
fn station_schema_snapshot() {
    let list = vec![
        StationDataItem {
            name: "北京南".to_string(),
            city: "北京".to_string(),
        },
        StationDataItem {
            name: "天津".to_string(),
            city: "天津".to_string(),
        },
    ];

    insta::with_settings!({sort_maps => true}, {
        insta::assert_json_snapshot!(list, {});
    });
}

#[test]
fn train_type_schema_snapshot() {
    // seat: 等级 -> 位置 -> 座位信息数组
    let mut seat: HashMap<String, HashMap<char, Vec<shared::data::SeatInfo>>> = HashMap::new();

    let mut loc_map: HashMap<char, Vec<shared::data::SeatInfo>> = HashMap::new();
    loc_map.insert(
        'A',
        vec![shared::data::SeatInfo {
            description: shared::data::SeatLocationInfo {
                carriage: 3,
                row: 12,
                location: 'A',
                type_name: "二等座".to_string(),
            },
            price: 350,
        }],
    );
    loc_map.insert(
        'B',
        vec![shared::data::SeatInfo {
            description: shared::data::SeatLocationInfo {
                carriage: 3,
                row: 12,
                location: 'B',
                type_name: "二等座".to_string(),
            },
            price: 350,
        }],
    );
    seat.insert("二等座".to_string(), loc_map);

    let item = TrainTypeInfoItem {
        id: "G".to_string(),
        name: "高铁".to_string(),
        seat,
    };

    // 将 seat 中的 char 键转换为字符串键，并进行排序，避免 JSON map 键类型限制
    use std::collections::BTreeMap;
    let mut seat_sorted: BTreeMap<String, BTreeMap<String, Vec<shared::data::SeatInfo>>> =
        BTreeMap::new();
    for (seat_type, loc_map) in &item.seat {
        let mut loc_sorted: BTreeMap<String, Vec<shared::data::SeatInfo>> = BTreeMap::new();
        for (loc, seats) in loc_map {
            loc_sorted.insert(loc.to_string(), seats.clone());
        }
        seat_sorted.insert(seat_type.clone(), loc_sorted);
    }

    let snap = serde_json::json!({
        "id": item.id,
        "name": item.name,
        "seat": seat_sorted,
    });

    insta::with_settings!({sort_maps => true}, {
        insta::assert_json_snapshot!(snap, {});
    });
}

#[test]
fn hotel_schema_snapshot() {
    let mut room_info: HashMap<String, HotelRoomType> = HashMap::new();
    room_info.insert(
        "标准间".to_string(),
        HotelRoomType {
            capacity: 2,
            price: 388.0,
        },
    );

    let item = HotelInfo {
        name: "示例酒店".to_string(),
        address: "示例路 123 号".to_string(),
        city: "北京".to_string(),
        station: Some("北京南".to_string()),
        images: vec!["./images/example_hotel_1.jpg".to_string()],
        phone: vec!["010-12345678".to_string()],
        info: "这是一家示例酒店".to_string(),
        room_info,
        comments: vec![HotelComment {
            time: "2025-01-02 12:34:56".to_string(),
            rating: 4.5,
            text: "房间干净整洁".to_string(),
        }],
    };

    insta::with_settings!({sort_maps => true}, {
        insta::assert_json_snapshot!(item, {});
    });
}

#[test]
fn dish_takeaway_schema_snapshot() {
    let dish_info = vec![DishInfo {
        available_time: vec!["lunch".to_string(), "dinner".to_string()],
        name: "牛肉面".to_string(),
        dish_type: "主食".to_string(),
        picture: "./images/example_dish_1.jpg".to_string(),
        price: 25.0,
    }];

    let mut shop_map: HashMap<String, Vec<TakeawayDishInfo>> = HashMap::new();
    shop_map.insert(
        "老张拉面".to_string(),
        vec![TakeawayDishInfo {
            name: "番茄鸡蛋盖饭".to_string(),
            picture: "./images/example_takeaway_1.jpg".to_string(),
            price: 22.0,
        }],
    );

    let mut takeaway_info: HashMap<String, HashMap<String, Vec<TakeawayDishInfo>>> = HashMap::new();
    takeaway_info.insert("北京南".to_string(), shop_map);

    let pack = shared::data::RawDishTakeawayInfo {
        train_number: "G1234".to_string(),
        dish_info,
        takeaway_info,
    };

    insta::with_settings!({sort_maps => true}, {
        insta::assert_json_snapshot!(pack, {});
    });
}
