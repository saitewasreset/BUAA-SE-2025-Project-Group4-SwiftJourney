//! TrainQueryServiceImpl 实现
//!
//! 该文件位于 `backend/base/src/infrastructure/application/service/train_query.rs`，
//! 负责把 HTTP / RPC 层收到的 CQRS 命令简单校验后转交给真正的
//! `TrainQueryService`（领域 / 应用服务）。
//!
//! * **为什么需要 `Arc<T>`?**
//!   应用服务通常是无状态、线程安全的，把它包进 `Arc` 便于在整个线程池里共享，
//!   无需再加 `Mutex`（不会修改内部状态）。
//! * **为什么要 `T: Send + Sync + 'static`?**
//!   Actix‑web、Tokio 等异步运行时会把服务对象跨线程移动，
//!   因此需要 `Send`；并发访问需要 `Sync`；`'static` 避免悬垂引用。

/// 火车查询应用服务实现
///
/// 此服务负责处理火车查询相关的应用逻辑，包括直达车次查询和中转车次查询。
/// 主要职责是验证输入参数，然后将请求委派给实际的领域服务处理。
///
/// # 功能
/// - 直达车次查询：根据出发站、目的站和出发日期查询直达火车
/// - 中转车次查询：根据出发城市、目的城市和出发日期查询需要换乘的火车方案
///
/// # 错误处理
/// - 输入验证：检查车站ID和城市ID是否为空
/// - 委托错误：传递领域服务可能返回的错误
///
/// # 实现细节
/// 该服务是对实际查询逻辑的一个适配层，主要负责：
/// 1. 参数校验 - 确保输入参数符合业务规则
/// 2. 错误转换 - 将领域错误映射为应用错误
/// 3. 转发调用 - 将实际查询工作委托给内部服务
///
/// # 示例
/// ```
/// let train_query_service = TrainQueryServiceImpl::new(Arc::new(real_service));
/// let command = DirectTrainQueryCommand {
///     session_id: "sessionId".to_string(),
///     from_station_id: "A".to_string(),
///     to_station_id: "B".to_string(),
///     departure_time: NaiveDate::from_ymd_opt(2023, 5, 1).unwrap(),
/// };
/// let result = train_query_service.query_direct_trains(command).await;
/// ```
// Step 1: Define `TrainQueryServiceImpl` application service implementation

// Step 2: Choose correct generics parameter according to data you need
// Exercise 1.2.1D - 5: Your code here. (1 / 6)
// HINT: You may refer to `UserManagerServiceImpl` for example
use std::sync::Arc;

use async_trait::async_trait;

use crate::application::commands::train_query::{
    DirectTrainQueryCommand, TransferTrainQueryCommand,
};
use crate::application::service::train_query::{
    DirectTrainQueryDTO, TrainQueryService, TransferTrainQueryDTO,
};
use crate::application::{ApplicationError, GeneralError};

// Thinking 1.2.1D - 4: 为何需要使用`+ 'static + Send + Sync`约束泛型参数？
// Thinking 1.2.1D - 5: 为何需要使用`Arc<T>`存储领域服务？为何无需使用`Arc<Mutex<T>>`？
pub struct TrainQueryServiceImpl<T>
where
    T: TrainQueryService + 'static + Send + Sync,
{
    // Step 3: Store service instance you need using `Arc<T>` and generics parameter
    // HINT: You may refer to `UserManagerServiceImpl` for example
    // Exercise 1.2.1D - 5: Your code here. (2 / 6)
    inner: Arc<T>,
}

// Step 4: Implement `new` associate function for `TrainQueryServiceImpl`
// HINT: You may refer to `UserManagerServiceImpl` for example
// Exercise 1.2.1D - 5: Your code here. (3 / 6)
impl<T> TrainQueryServiceImpl<T>
where
    T: TrainQueryService + 'static + Send + Sync,
{
    #[inline]
    pub fn new(inner: Arc<T>) -> Self {
        Self { inner }
    }
}

// Step 5: Implement `TrainQueryService` trait for `TrainQueryServiceImpl`
// HINT: You need to use `async_trait` macro
// HINT: You should check user input in application service function,
// return error in validate failed
// HINT: You SHOULD NOT perform business logic in application service
// just delegate business logic to other service
// HINT: You may refer to `UserManagerServiceImpl` for example
// Exercise 1.2.1D - 5: Your code here. (4 / 6)
#[async_trait]
impl<T> TrainQueryService for TrainQueryServiceImpl<T>
where
    T: TrainQueryService + 'static + Send + Sync,
{
    async fn query_direct_trains(
        &self,
        command: DirectTrainQueryCommand,
    ) -> Result<DirectTrainQueryDTO, Box<dyn ApplicationError>> {
        if command.from_station_id.trim().is_empty() || command.to_station_id.trim().is_empty() {
            return Err(GeneralError::BadRequest("station id 不能为空".into()).into());
        }
        self.inner.query_direct_trains(command).await
    }

    async fn query_transfer_trains(
        &self,
        command: TransferTrainQueryCommand,
    ) -> Result<TransferTrainQueryDTO, Box<dyn ApplicationError>> {
        if command.from_city_id.trim().is_empty() || command.to_city_id.trim().is_empty() {
            return Err(GeneralError::BadRequest("city id 不能为空".into()).into());
        }
        self.inner.query_transfer_trains(command).await
    }
}

// Step 6: Add unit test for your implementation
// HINT: You may use `mockall` crate to "mock" other service you depend on
// HINT: You may use AI tools to generate unit test
// HINT: You may refer to `UserManagerServiceImpl` for example
// Exercise 1.2.1D - 5: Your code here. (5 / 6)
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use mockall::{mock, predicate::*};
    use std::sync::Arc;

    mock! {
        pub Inner {}
        #[async_trait]
        impl TrainQueryService for Inner {
            async fn query_direct_trains(
                &self,
                cmd: DirectTrainQueryCommand,
            ) -> Result<DirectTrainQueryDTO, Box<dyn ApplicationError>>;
            async fn query_transfer_trains(
                &self,
                cmd: TransferTrainQueryCommand,
            ) -> Result<TransferTrainQueryDTO, Box<dyn ApplicationError>>;
        }
    }

    #[tokio::test]
    async fn delegating_direct_trains() {
        let mut mock = MockInner::new();
        let cmd = DirectTrainQueryCommand {
            session_id: Default::default(),
            from_station_id: "A".into(),
            to_station_id: "B".into(),
            departure_time: NaiveDate::from_ymd_opt(2025, 5, 1).unwrap(),
        };
        let expect_dto = DirectTrainQueryDTO { solutions: vec![] };
        mock.expect_query_direct_trains()
            .withf(move |c| {
                c.from_station_id == "A"
                    && c.to_station_id == "B"
                    && c.departure_time == NaiveDate::from_ymd_opt(2025, 5, 1).unwrap()
            })
            .times(1)
            .returning(move |_| Ok(expect_dto.clone()));

        let svc = TrainQueryServiceImpl::new(Arc::new(mock));
        let res = svc.query_direct_trains(cmd).await.unwrap();
        assert!(res.solutions.is_empty());
    }

    #[tokio::test]
    async fn delegating_transfer_trains() {
        let mut mock = MockInner::new();
        let cmd = TransferTrainQueryCommand {
            session_id: Default::default(),
            from_city_id: "C1".into(),
            to_city_id: "C2".into(),
            departure_time: NaiveDate::from_ymd_opt(2025, 5, 1).unwrap(),
        };
        let expect_dto = TransferTrainQueryDTO { solutions: vec![] };
        mock.expect_query_transfer_trains()
            .withf(move |c| {
                c.from_city_id == "C1"
                    && c.to_city_id == "C2"
                    && c.departure_time == NaiveDate::from_ymd_opt(2025, 5, 1).unwrap()
            })
            .times(1)
            .returning(move |_| Ok(expect_dto.clone()));

        let svc = TrainQueryServiceImpl::new(Arc::new(mock));
        let res = svc.query_transfer_trains(cmd).await.unwrap();
        assert!(res.solutions.is_empty());
    }

    #[tokio::test]
    async fn test_query_direct_with_empty_from_station() {
        let mut mock = MockInner::new();
        let cmd = DirectTrainQueryCommand {
            session_id: Default::default(),
            from_station_id: "".into(), // 空出发站
            to_station_id: "B".into(),
            departure_time: NaiveDate::from_ymd_opt(2025, 5, 1).unwrap(),
        };

        // 我们期望服务本身拒绝这个请求，不会将其传递到内部服务
        let svc = TrainQueryServiceImpl::new(Arc::new(mock));
        let result = svc.query_direct_trains(cmd).await;

        // 应该返回错误，表示站点ID不能为空
        assert!(result.is_err());
        if let Err(e) = result {
            // 检查错误类型是否正确
            let err_str = format!("{:?}", e);
            assert!(err_str.contains("station id 不能为空"));
        }
    }

    #[tokio::test]
    async fn test_query_direct_with_empty_to_station() {
        let mut mock = MockInner::new();
        let cmd = DirectTrainQueryCommand {
            session_id: Default::default(),
            from_station_id: "A".into(),
            to_station_id: "".into(), // 空目的站
            departure_time: NaiveDate::from_ymd_opt(2025, 5, 1).unwrap(),
        };

        // 我们期望服务本身拒绝这个请求，不会将其传递到内部服务
        let svc = TrainQueryServiceImpl::new(Arc::new(mock));
        let result = svc.query_direct_trains(cmd).await;

        // 应该返回错误，表示站点ID不能为空
        assert!(result.is_err());
        if let Err(e) = result {
            // 检查错误类型是否正确
            let err_str = format!("{:?}", e);
            assert!(err_str.contains("station id 不能为空"));
        }
    }

    #[tokio::test]
    async fn test_query_transfer_with_empty_from_city() {
        let mut mock = MockInner::new();
        let cmd = TransferTrainQueryCommand {
            session_id: Default::default(),
            from_city_id: "".into(), // 空出发城市
            to_city_id: "C2".into(),
            departure_time: NaiveDate::from_ymd_opt(2025, 5, 1).unwrap(),
        };

        // 我们期望服务本身拒绝这个请求，不会将其传递到内部服务
        let svc = TrainQueryServiceImpl::new(Arc::new(mock));
        let result = svc.query_transfer_trains(cmd).await;

        // 应该返回错误，表示城市ID不能为空
        assert!(result.is_err());
        if let Err(e) = result {
            // 检查错误类型是否正确
            let err_str = format!("{:?}", e);
            assert!(err_str.contains("city id 不能为空"));
        }
    }

    #[tokio::test]
    async fn test_query_transfer_with_empty_to_city() {
        let mut mock = MockInner::new();
        let cmd = TransferTrainQueryCommand {
            session_id: Default::default(),
            from_city_id: "C1".into(),
            to_city_id: "".into(), // 空目的城市
            departure_time: NaiveDate::from_ymd_opt(2025, 5, 1).unwrap(),
        };

        // 我们期望服务本身拒绝这个请求，不会将其传递到内部服务
        let svc = TrainQueryServiceImpl::new(Arc::new(mock));
        let result = svc.query_transfer_trains(cmd).await;

        // 应该返回错误，表示城市ID不能为空
        assert!(result.is_err());
        if let Err(e) = result {
            // 检查错误类型是否正确
            let err_str = format!("{:?}", e);
            assert!(err_str.contains("city id 不能为空"));
        }
    }

    #[tokio::test]
    async fn test_query_transfer_success() {
        let mut mock = MockInner::new();
        let cmd = TransferTrainQueryCommand {
            session_id: Default::default(),
            from_city_id: "C1".into(),
            to_city_id: "C2".into(),
            departure_time: NaiveDate::from_ymd_opt(2025, 5, 1).unwrap(),
        };
        let expect_dto = TransferTrainQueryDTO { solutions: vec![] };
        mock.expect_query_transfer_trains()
            .withf(move |c| {
                c.from_city_id == "C1"
                    && c.to_city_id == "C2"
                    && c.departure_time == NaiveDate::from_ymd_opt(2025, 5, 1).unwrap()
            })
            .times(1)
            .returning(move |_| Ok(expect_dto.clone()));

        let svc = TrainQueryServiceImpl::new(Arc::new(mock));
        let res = svc.query_transfer_trains(cmd).await.unwrap();
        assert!(res.solutions.is_empty());
    }
}

// Step 7: Write document comment and mod comment for your implementation
// HINT: You may use AI tools to generate comment
// HINT: You may refer to `UserManagerServiceImpl` for example
// Exercise 1.2.1D - 5: Your code here. (6 / 6)

// Good! Next, register your application service in `api::main`
