use async_trait::async_trait;

// Step 2: Define `TrainQueryServiceError` for possible errors
// HINT: You may refer to RFC4 "直达车次查询（US1.2.1）" and "中转车次查询（US3.1.1）" for possible errors
// HINT: You may refer to `UserManagerError` for example
// Exercise 1.2.1D - 4: Your code here. (1 / 6)

// Step 3: Implement `ApplicationError` trait for `TrainQueryServiceError`
// 实现了该特征的错误类型可在Web框架中自动生成JSON响应
// HINT: You may refer to RFC4 "直达车次查询（US1.2.1）" and "中转车次查询（US3.1.1）" for possible
// error code and error message
// HINT: You may refer to `UserManagerError` for example
// Exercise 1.2.1D - 4: Your code here. (2 / 6)

// Thinking 1.2.1D - 2：尝试探究为何`Result<ApiResponse<T>, ApplicationErrorBox>`
// 可以转化为HTTP响应，且响应体为符合条件的JSON？
// HINT: You may refer to https://actix.rs/docs/handlers#response-with-custom-type

// Step 4: Implement From<UserServiceError> for Box<dyn ApplicationError>
// HINT: You may refer to `UserManagerError` for example
// Exercise 1.2.1D - 4: Your code here. (3 / 6)

// Step 5: Choose proper CQRS struct for user request
// HINT: You may refer to https://mp.weixin.qq.com/s/1rdnkROdcNw5ro4ct99SqQ for definition of CQRS
// HINT: You may refer to `UserManagerService` for example
// Exercise 1.2.1D - 4: Your code here. (4 / 6)

// Step 6: Define proper DTO(Data Transfer Object) for request json and response json
// HINT: You may refer to `UserManagerService` for example
// Exercise 1.2.1D - 4: Your code here. (5 / 6)

// Thinking 1.2.1D - 3：DTO和CQRS结构的区别与联系是什么？它们中的数据是经过校验的，还是未经过校验的？

#[async_trait]
// Step 1: Define `TrainQueryService` application service
// Thinking 1.2.1D - 1：你认为`async_trait`宏的作用是什么？为什么需要使用它？
pub trait TrainQueryService {
    // Step 5: Define service using `async fn xxx(&self, command: XXXCommand)
    //     -> Result<DTO, Box<dyn ApplicationError>>;`
    // HINT: You may refer to `UserManagerService` for example
    // HINT: application service function should only receive CQRS parameter and
    // always return `Result<DTO, Box<dyn ApplicationError>>`
    // Exercise 1.2.1D - 4: Your code here. (6 / 6)
    // Good! Next, implement `TrainQueryService` in `base::infrastructure::application::service`
}
