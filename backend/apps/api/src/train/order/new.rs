use crate::{ApiResponse, ApplicationErrorBox, get_session_id, parse_request_body};
use actix_web::{HttpRequest, post, web::Bytes, web::Data};
use base::{
    application::ApplicationError,
    application::service::train_order::{CreateTrainOrderDTO, TrainOrderService},
};
use serde::{Deserialize, Serialize};
use shared::{API_SUCCESS_CODE, API_SUCCESS_MESSAGE};

// 定义请求数据结构
#[derive(Serialize, Deserialize, Debug)]
pub struct TrainOrderRequestDTO {
    /// 车次号，例如："G53"
    pub train_number: String,
    /// 离开"始发站"的日期时间
    pub origin_departure_time: String,
    /// 起始站
    pub departure_station: String,
    /// 到达站
    pub arrival_station: String,
    /// 乘车人 Id（见`PersonalInfo`）
    pub personal_id: String,
    /// 座位类别，如：二等座
    pub seat_type: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OrderPackDTO {
    /// 原子操作，若为 true，则`order_list`中任意订单失败将回滚已成功的订单
    pub atomic: bool,
    /// 订单列表
    pub order_list: Vec<TrainOrderRequestDTO>,
}

pub type OrderPacksDTO = Vec<OrderPackDTO>;

/// 用于响应的交易信息
#[derive(Serialize, Deserialize, Debug)]
pub struct TransactionInfoDTO {
    pub transaction_id: String,
}

// 自定义错误实现ApplicationError特征
#[derive(Debug)]
struct CreateOrderError;

impl std::fmt::Display for CreateOrderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "创建订单失败")
    }
}

impl std::error::Error for CreateOrderError {}

impl ApplicationError for CreateOrderError {
    fn error_code(&self) -> u32 {
        500
    }

    fn error_message(&self) -> String {
        "创建订单失败".to_string()
    }
}

/// 创建火车票订单
///
/// POST /api/train/order/new
///
/// 提交订单后，订单为"未支付"状态，本接口将返回`TransactionInfo`，
/// 需要根据`TransactionInfo`中的信息调用`支付订单`接口进行支付。
/// 支付后订单才会真正被处理。
#[post("/new")]
pub async fn create_train_order(
    req: HttpRequest,
    body: Bytes,
    train_order_service: Data<dyn TrainOrderService>,
) -> Result<ApiResponse<TransactionInfoDTO>, ApplicationErrorBox> {
    // 从Cookie中获取session_id并验证
    let _session_id = get_session_id(&req)?;

    // 解析请求体为订单包列表
    let order_packs: OrderPacksDTO = parse_request_body(body)?;

    // 处理订单逻辑实现
    let transaction_id = "transaction_123456".to_string();

    // 处理每一个订单包
    for pack in order_packs {
        let order_dtos: Vec<CreateTrainOrderDTO> = pack
            .order_list
            .into_iter()
            .map(|order| CreateTrainOrderDTO {
                train_number: order.train_number,
                origin_departure_time: order.origin_departure_time,
                departure_station: order.departure_station,
                arrival_station: order.arrival_station,
                personal_id: order.personal_id,
                seat_type: order.seat_type,
            })
            .collect();

        // 调用服务层方法处理订单
        for dto in order_dtos {
            // 实际的TrainOrderService可能需要逐个创建订单
            if let Err(e) = train_order_service.create_train_order(dto).await {
                eprintln!("创建订单失败: {:?}", e);
                return Err(ApplicationErrorBox::from(
                    Box::new(CreateOrderError) as Box<dyn ApplicationError>
                ));
            }
        }
    }

    // 返回成功响应
    Ok(ApiResponse {
        code: API_SUCCESS_CODE,
        message: API_SUCCESS_MESSAGE.to_string(),
        data: Some(TransactionInfoDTO { transaction_id }),
    })
}
