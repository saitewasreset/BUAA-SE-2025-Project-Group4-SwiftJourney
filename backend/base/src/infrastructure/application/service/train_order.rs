use crate::application::commands::train_order::CreateTrainOrderCommand;
use crate::application::service::train_order::CreateTrainOrderDTO;
use crate::application::service::train_order::{TrainOrderService, TrainOrderServiceError};
use crate::domain::Identifiable;
use crate::domain::model::order::Order;
use crate::domain::model::train::TrainNumber;
use crate::domain::repository::order::OrderRepository;
use crate::domain::repository::route::RouteRepository;
use crate::domain::repository::station::StationRepository;
use crate::domain::repository::train::TrainRepository;
use crate::domain::repository::train_schedule::TrainScheduleRepository;
use crate::domain::service::train_booking::TrainBookingService;
use crate::domain::service::transaction::TransactionService;
use async_trait::async_trait;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tracing::{error, info};
use uuid::Uuid;

// RabbitMQ 消息队列实现
mod rabbitmq {
    use lapin::{
        BasicProperties, Connection, ConnectionProperties, Result, options::*,
        publisher_confirm::Confirmation, types::FieldTable,
    };
    use serde::{Deserialize, Serialize};
    use std::future::Future;
    use std::pin::Pin;
    use std::sync::Arc;
    use tokio_stream::StreamExt;
    use tracing::{error, info};
    use uuid::Uuid;

    // 定义消息结构
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct OrderMessage {
        pub transaction_id: Uuid,
        pub order_uuids: Vec<Uuid>,
        pub atomic: bool,
    }

    #[derive(Clone)]
    pub struct RabbitMQClient {
        connection: Arc<Connection>,
    }

    impl RabbitMQClient {
        pub async fn new(url: &str) -> Result<Self> {
            let connection = Connection::connect(url, ConnectionProperties::default()).await?;

            info!("Connected to RabbitMQ");

            Ok(Self {
                connection: Arc::new(connection),
            })
        }

        // 发送订单消息到队列
        pub async fn send_order_message(&self, message: &OrderMessage) -> Result<Confirmation> {
            let channel = self.connection.create_channel().await?;

            // 声明队列
            let queue_name = "order_processing";
            let _ = channel
                .queue_declare(
                    queue_name,
                    QueueDeclareOptions::default(),
                    FieldTable::default(),
                )
                .await?;

            // 序列化消息
            let payload = serde_json::to_string(message).map_err(|e| {
                error!("Failed to serialize message: {}", e);
                lapin::Error::IOError(
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Failed to serialize message",
                    )
                    .into(),
                )
            })?;

            // 发布消息
            channel
                .basic_publish(
                    "",
                    queue_name,
                    BasicPublishOptions::default(),
                    payload.as_bytes(),
                    BasicProperties::default(),
                )
                .await?
                .await
        }

        // 接收并处理来自队列的订单消息
        pub async fn consume_order_messages<F>(&self, handler: F) -> Result<()>
        where
            F: Fn(OrderMessage) -> Pin<Box<dyn Future<Output = ()> + Send + 'static>>
                + Send
                + Sync
                + Clone // 添加 Clone 约束
                + 'static,
        {
            let channel = self.connection.create_channel().await?;

            // 声明队列
            let queue_name = "order_processing";
            let _ = channel
                .queue_declare(
                    queue_name,
                    QueueDeclareOptions::default(),
                    FieldTable::default(),
                )
                .await?;

            // 消费消息
            let mut consumer = channel
                .basic_consume(
                    queue_name,
                    "order_consumer",
                    BasicConsumeOptions::default(),
                    FieldTable::default(),
                )
                .await?;

            info!("Starting to consume messages from queue: {}", queue_name);

            // 处理消息
            while let Some(delivery) = consumer.next().await {
                if let Ok(delivery) = delivery {
                    match serde_json::from_slice::<OrderMessage>(&delivery.data) {
                        Ok(message) => {
                            info!("Received order message: {:?}", message);

                            // 调用处理函数，为每次迭代克隆handler
                            let message_clone = message.clone();
                            let handler_clone = handler.clone();
                            tokio::spawn(async move {
                                handler_clone(message_clone).await;
                            });

                            // 确认消息已处理
                            let _ = delivery.ack(BasicAckOptions::default()).await;
                        }
                        Err(e) => {
                            error!("Failed to deserialize message: {}", e);
                            let _ = delivery.reject(BasicRejectOptions::default()).await;
                        }
                    }
                }
            }

            Ok(())
        }
    }
}

// 订单批次结果
#[derive(Debug)]
pub struct CreateBatchOrderResult {
    pub transaction_id: Uuid,
    pub order_commands: Vec<CreateTrainOrderCommand>,
}

#[derive(Clone)]
pub struct TrainOrderServiceImpl<TSR, TBS, TR, RR, SR, OR, TS>
where
    TSR: TrainScheduleRepository,
    TBS: TrainBookingService,
    TR: TrainRepository,
    RR: RouteRepository,
    SR: StationRepository,
    OR: OrderRepository,
    TS: TransactionService,
{
    train_schedule_repository: Arc<TSR>,
    train_booking_service: Arc<TBS>,
    rabbitmq_client: Option<Arc<rabbitmq::RabbitMQClient>>,
    train_repository: Arc<TR>,
    route_repository: Arc<RR>,
    station_repository: Arc<SR>,
    order_repository: Arc<OR>,
    transaction_service: Arc<TS>,
}

impl<TSR, TBS, TR, RR, SR, OR, TS> TrainOrderServiceImpl<TSR, TBS, TR, RR, SR, OR, TS>
where
    TSR: TrainScheduleRepository,
    TBS: TrainBookingService,
    TR: TrainRepository,
    RR: RouteRepository,
    SR: StationRepository,
    OR: OrderRepository,
    TS: TransactionService,
{
    pub fn new(
        train_schedule_repository: Arc<TSR>,
        train_booking_service: Arc<TBS>,
        train_repository: Arc<TR>,
        route_repository: Arc<RR>,
        station_repository: Arc<SR>,
        order_repository: Arc<OR>,
        transaction_service: Arc<TS>,
    ) -> Self {
        Self {
            train_schedule_repository,
            train_booking_service,
            rabbitmq_client: None,
            train_repository,
            route_repository,
            station_repository,
            order_repository,
            transaction_service,
        }
    }

    // 添加RabbitMQ客户端
    pub fn with_rabbitmq(mut self, rabbitmq_client: Arc<rabbitmq::RabbitMQClient>) -> Self {
        self.rabbitmq_client = Some(rabbitmq_client);
        self
    }

    // 工厂方法，创建带有RabbitMQ的实例
    pub async fn with_rabbitmq_url(
        self,
        rabbitmq_url: &str,
    ) -> Result<Self, TrainOrderServiceError> {
        match rabbitmq::RabbitMQClient::new(rabbitmq_url).await {
            Ok(client) => Ok(self.with_rabbitmq(Arc::new(client))),
            Err(e) => {
                error!("Failed to connect to RabbitMQ: {:?}", e);
                Err(TrainOrderServiceError::InvalidSessionId)
            }
        }
    }

    // 启动消息队列消费者
    pub async fn start_message_consumer(&self) -> Result<(), TrainOrderServiceError> {
        if let Some(client) = &self.rabbitmq_client {
            let train_booking_service = self.train_booking_service.clone();

            let handler = move |message: rabbitmq::OrderMessage| {
                let booking_service = train_booking_service.clone();
                Box::pin(async move {
                    info!(
                        "Processing order message from queue: transaction_id={}, orders={:?}, atomic={}",
                        message.transaction_id, message.order_uuids, message.atomic
                    );

                    if let Err(e) = booking_service
                        .booking_group(message.order_uuids, message.atomic)
                        .await
                    {
                        error!("Failed to process orders from queue: {:?}", e);
                    }
                }) as Pin<Box<dyn Future<Output = ()> + Send + 'static>>
            };

            // 创建任务消费队列消息
            let client_clone = client.clone();
            tokio::spawn(async move {
                if let Err(e) = client_clone.consume_order_messages(handler).await {
                    error!("Message consumer failed: {:?}", e);
                }
            });

            Ok(())
        } else {
            error!("Cannot start message consumer: RabbitMQ client not configured");
            Err(TrainOrderServiceError::InvalidSessionId)
        }
    }

    // 验证火车订单数据
    async fn validate_train_order(
        &self,
        dto: &CreateTrainOrderDTO,
    ) -> Result<(), TrainOrderServiceError> {
        let train_number = TrainNumber::from_unchecked(dto.train_number.clone());

        let train = self
            .train_repository
            .find_by_train_number(train_number)
            .await
            .map_err(|_| TrainOrderServiceError::InvalidTrainNumber)?;

        let train_id = train
            .get_id()
            .ok_or(TrainOrderServiceError::InvalidTrainNumber)?;

        let schedules_result = self
            .train_schedule_repository
            .find_by_train_id(train_id)
            .await
            .map_err(|_| TrainOrderServiceError::InvalidTrainNumber)?;

        let train_schedule = schedules_result
            .iter()
            .find(|schedule| {
                schedule.origin_departure_time().to_string() == dto.origin_departure_time
            })
            .cloned()
            .ok_or(TrainOrderServiceError::InvalidTrainNumber)?;

        let route_id = train_schedule.route_id();

        let route = self
            .route_repository
            .find(route_id)
            .await
            .map_err(|_| TrainOrderServiceError::InvalidStationId)?
            .ok_or(TrainOrderServiceError::InvalidStationId)?;

        let stations = route.stops();

        let mut departure_exists = false;
        let mut arrival_exists = false;

        for stop in stations {
            if let Ok(Some(station)) = self.station_repository.find(stop.station_id()).await {
                let station_name = station.name().to_string();

                if station_name == dto.departure_station {
                    departure_exists = true;
                }
                if station_name == dto.arrival_station {
                    arrival_exists = true;
                }

                if departure_exists && arrival_exists {
                    break;
                }
            }
        }

        if !departure_exists || !arrival_exists {
            return Err(TrainOrderServiceError::InvalidStationId);
        }

        let train_details = self
            .train_repository
            .find(train_id)
            .await
            .map_err(|_| TrainOrderServiceError::InvalidTrainNumber)?
            .ok_or(TrainOrderServiceError::InvalidTrainNumber)?;

        let seat_type_exists = train_details
            .seats()
            .iter()
            .any(|(key, _)| key == &dto.seat_type);

        if !seat_type_exists {
            return Err(TrainOrderServiceError::InvalidTrainNumber);
        }

        Ok(())
    }

    // 处理订单消息（模拟消息队列消费者处理）
    pub async fn process_order_message(
        &self,
        transaction_id: Uuid,
        order_uuids: Vec<Uuid>,
        atomic: bool,
    ) -> Result<(), TrainOrderServiceError> {
        info!("Processing orders for transaction: {}", transaction_id);

        // 调用booking_group处理订单
        let result = self
            .train_booking_service
            .booking_group(order_uuids.clone(), atomic)
            .await;

        match result {
            Ok(_) => {
                info!(
                    "Successfully processed orders for transaction: {}",
                    transaction_id
                );
                Ok(())
            }
            Err(err) => {
                error!(
                    "Failed to process orders for transaction {}: {:?}",
                    transaction_id, err
                );

                // 自动触发退款流程
                info!(
                    "Initiating automatic refund for failed transaction: {}",
                    transaction_id
                );
                if let Err(refund_err) = self
                    .refund_order_transaction(transaction_id, order_uuids)
                    .await
                {
                    error!(
                        "Failed to process automatic refund for transaction {}: {:?}",
                        transaction_id, refund_err
                    );
                } else {
                    info!(
                        "Automatic refund successfully initiated for transaction: {}",
                        transaction_id
                    );
                }

                Err(TrainOrderServiceError::InvalidTrainNumber)
            }
        }
    }

    // 创建订单批次，可以通过消息队列异步处理
    pub async fn create_order_batch(
        &self,
        _user_id: i32,
        order_dtos: Vec<CreateTrainOrderDTO>,
        atomic: bool,
    ) -> Result<CreateBatchOrderResult, TrainOrderServiceError> {
        let mut orders = Vec::new();
        let transaction_id = Uuid::new_v4(); // 为整个批次生成一个事务ID

        for dto in order_dtos {
            // 验证并创建单个订单
            let order_command = self.create_train_order_internal(&dto).await?;
            orders.push(order_command);
        }

        // 为每个订单生成UUID
        let order_uuids = (0..orders.len())
            .map(|_| Uuid::new_v4())
            .collect::<Vec<_>>();

        // 如果配置了RabbitMQ，则通过消息队列处理
        if let Some(client) = &self.rabbitmq_client {
            let message = rabbitmq::OrderMessage {
                transaction_id,
                order_uuids: order_uuids.clone(),
                atomic,
            };

            match client.send_order_message(&message).await {
                Ok(_) => {
                    info!(
                        "Sent order batch to queue for processing: transaction_id={}",
                        transaction_id
                    );
                }
                Err(e) => {
                    error!("Failed to send order batch to queue: {:?}", e);
                    // 如果消息队列失败，直接处理订单
                    let _ = self
                        .process_order_message(transaction_id, order_uuids, atomic)
                        .await;
                }
            }
        } else {
            // 如果没有配置消息队列，则直接处理
            let _ = self
                .process_order_message(transaction_id, order_uuids, atomic)
                .await;
        }

        Ok(CreateBatchOrderResult {
            transaction_id,
            order_commands: orders,
        })
    }

    // 取消订单
    pub async fn cancel_order(&self, order_uuid: Uuid) -> Result<(), TrainOrderServiceError> {
        // 调用领域服务取消订单
        match self.train_booking_service.cancel_ticket(order_uuid).await {
            Ok(_) => {
                info!("Order {} cancelled successfully", order_uuid);
                Ok(())
            }
            Err(_) => {
                error!("Failed to cancel order {}", order_uuid);
                Err(TrainOrderServiceError::InvalidTrainNumber)
            }
        }
    }

    // 退款方法 - 根据交易ID和订单UUIDs进行退款
    pub async fn refund_order_transaction(
        &self,
        transaction_id: Uuid,
        order_uuids: Vec<Uuid>,
    ) -> Result<Uuid, TrainOrderServiceError> {
        info!("Initiating refund for transaction: {}", transaction_id);

        // 存储要退款的订单
        let mut to_refund_orders: Vec<Box<dyn Order>> = Vec::new();

        // 获取每个要退款的订单
        for order_uuid in order_uuids {
            match self
                .order_repository
                .find_train_order_by_uuid(order_uuid)
                .await
            {
                Ok(Some(order)) => {
                    info!("Found order {} for refund", order_uuid);
                    to_refund_orders.push(Box::new(order));
                }
                Ok(None) => {
                    error!("Order {} not found for refund", order_uuid);
                    return Err(TrainOrderServiceError::InvalidTrainNumber);
                }
                Err(err) => {
                    error!("Error finding order {} for refund: {:?}", order_uuid, err);
                    return Err(TrainOrderServiceError::InvalidTrainNumber);
                }
            }
        }

        if to_refund_orders.is_empty() {
            error!(
                "No valid orders found for refund in transaction {}",
                transaction_id
            );
            return Err(TrainOrderServiceError::InvalidTrainNumber);
        }

        // 调用TransactionService进行退款
        match self
            .transaction_service
            .refund_transaction(transaction_id, &to_refund_orders)
            .await
        {
            Ok(refund_tx_id) => {
                info!(
                    "Refund successful for transaction {}, generated refund transaction: {}",
                    transaction_id, refund_tx_id
                );

                // 执行订单取消操作
                for order in &to_refund_orders {
                    if let Err(e) = self.train_booking_service.cancel_ticket(order.uuid()).await {
                        // 记录错误，但不影响已经创建的退款交易
                        error!(
                            "Failed to cancel order {} after refund: {:?}",
                            order.uuid(),
                            e
                        );
                    }
                }

                Ok(refund_tx_id)
            }
            Err(err) => {
                error!("Failed to refund transaction {}: {:?}", transaction_id, err);
                Err(TrainOrderServiceError::InvalidTrainNumber)
            }
        }
    }

    // 创建单个火车订单的内部实现
    async fn create_train_order_internal(
        &self,
        dto: &CreateTrainOrderDTO,
    ) -> Result<CreateTrainOrderCommand, TrainOrderServiceError> {
        // 验证订单数据
        self.validate_train_order(dto).await?;

        let command = CreateTrainOrderCommand {
            train_number: dto.train_number.clone(),
            origin_departure_time: dto.origin_departure_time.clone(),
            departure_station: dto.departure_station.clone(),
            arrival_station: dto.arrival_station.clone(),
            personal_id: dto.personal_id.clone(),
            seat_type: dto.seat_type.clone(),
        };

        info!("Train order created successfully");
        Ok(command)
    }
}

#[async_trait]
impl<TSR, TBS, TR, RR, SR, OR, TS> TrainOrderService
    for TrainOrderServiceImpl<TSR, TBS, TR, RR, SR, OR, TS>
where
    TSR: TrainScheduleRepository + Send + Sync + 'static,
    TBS: TrainBookingService + Send + Sync + 'static,
    TR: TrainRepository + Send + Sync + 'static,
    RR: RouteRepository + Send + Sync + 'static,
    SR: StationRepository + Send + Sync + 'static,
    OR: OrderRepository + Send + Sync + 'static,
    TS: TransactionService + Send + Sync + 'static,
{
    async fn create_train_order(
        &self,
        dto: CreateTrainOrderDTO,
    ) -> Result<CreateTrainOrderCommand, TrainOrderServiceError> {
        self.create_train_order_internal(&dto).await
    }

    async fn refund_order_transaction(
        &self,
        transaction_id: Uuid,
        order_uuids: Vec<Uuid>,
    ) -> Result<Uuid, TrainOrderServiceError> {
        self.refund_order_transaction(transaction_id, order_uuids)
            .await
    }
}
