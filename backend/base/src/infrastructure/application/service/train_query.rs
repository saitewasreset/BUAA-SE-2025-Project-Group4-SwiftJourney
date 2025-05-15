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
    TrainQueryService, DirectTrainQueryDTO,
    TransferTrainQueryDTO,
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
// #[cfg(test)]
// mod tests {
//     use super::*;
//     use chrono::NaiveDate;
//     use mockall::{mock, predicate::*};
//     use std::sync::Arc;

//     // —— 为 TrainQueryService 生成 Mock ——
//     mock! {
//         pub Inner {}
//         #[async_trait]
//         impl TrainQueryService for Inner {
//             async fn query_direct_trains(
//                 &self,
//                 cmd: DirectTrainQueryCommand,
//             ) -> Result<DirectTrainQueryDTO, Box<dyn ApplicationError>>;
//             async fn query_transfer_trains(
//                 &self,
//                 cmd: TransferTrainQueryCommand,
//             ) -> Result<TransferTrainQueryDTO, Box<dyn ApplicationError>>;
//         }
//     }

//     #[tokio::test]
//     async fn delegating_direct_trains() {
//         let mut mock = MockInner::new();
//         let cmd = DirectTrainQueryCommand {
//             session_id: Default::default(),
//             from_station_id: "A".into(),
//             to_station_id: "B".into(),
//             departure_time: NaiveDate::from_ymd_opt(2025, 5, 1).unwrap(),
//         };
//         let expect_dto = DirectTrainQueryDTO { solutions: vec![] };
//         mock.expect_query_direct_trains()
//             .with(eq(cmd.clone()))
//             .times(1)
//             .returning(move |_| Ok(expect_dto.clone()));

//         let svc = TrainQueryServiceImpl::new(Arc::new(mock));
//         let res = svc.query_direct_trains(cmd).await.unwrap();
//         assert!(res.solutions.is_empty());
//     }

//     #[tokio::test]
//     async fn delegating_transfer_trains() {
//         let mut mock = MockInner::new();
//         let cmd = TransferTrainQueryCommand {
//             session_id: Default::default(),
//             from_city_id: "C1".into(),
//             to_city_id: "C2".into(),
//             departure_time: NaiveDate::from_ymd_opt(2025, 5, 1).unwrap(),
//         };
//         let expect_dto = TransferTrainQueryDTO { solutions: vec![] };
//         mock.expect_query_transfer_trains()
//             .with(eq(cmd.clone()))
//             .times(1)
//             .returning(move |_| Ok(expect_dto.clone()));

//         let svc = TrainQueryServiceImpl::new(Arc::new(mock));
//         let res = svc.query_transfer_trains(cmd).await.unwrap();
//         assert!(res.solutions.is_empty());
//     }
// }

// Step 7: Write document comment and mod comment for your implementation
// HINT: You may use AI tools to generate comment
// HINT: You may refer to `UserManagerServiceImpl` for example
// Exercise 1.2.1D - 5: Your code here. (6 / 6)

// Good! Next, register your application service in `api::main`
