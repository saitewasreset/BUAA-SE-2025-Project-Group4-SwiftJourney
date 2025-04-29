// Step 1: Define `TrainQueryServiceImpl` application service implementation

// Step 2: Choose correct generics parameter according to data you need
// Exercise 1.2.1D - 5: Your code here. (1 / 6)
// HINT: You may refer to `UserManagerServiceImpl` for example

// Thinking 1.2.1D - 4: 为何需要使用`+ 'static + Send + Sync`约束泛型参数？
// Thinking 1.2.1D - 5: 为何需要使用`Arc<T>`存储领域服务？为何无需使用`Arc<Mutex<T>>`？
pub struct TrainQueryServiceImpl {
    // Step 3: Store service instance you need using `Arc<T>` and generics parameter
    // HINT: You may refer to `UserManagerServiceImpl` for example
    // Exercise 1.2.1D - 5: Your code here. (2 / 6)
}

// Step 4: Implement `new` associate function for `TrainQueryServiceImpl`
// HINT: You may refer to `UserManagerServiceImpl` for example
// Exercise 1.2.1D - 5: Your code here. (3 / 6)

// Step 5: Implement `TrainQueryService` trait for `TrainQueryServiceImpl`
// HINT: You need to use `async_trait` macro
// HINT: You should check user input in application service function,
// return error in validate failed
// HINT: You SHOULD NOT perform business logic in application service
// just delegate business logic to other service
// HINT: You may refer to `UserManagerServiceImpl` for example
// Exercise 1.2.1D - 5: Your code here. (4 / 6)

// Step 6: Add unit test for your implementation
// HINT: You may use `mockall` crate to "mock" other service you depend on
// HINT: You may use AI tools to generate unit test
// HINT: You may refer to `UserManagerServiceImpl` for example
// Exercise 1.2.1D - 5: Your code here. (5 / 6)

// Step 7: Write document comment and mod comment for your implementation
// HINT: You may use AI tools to generate comment
// HINT: You may refer to `UserManagerServiceImpl` for example
// Exercise 1.2.1D - 5: Your code here. (6 / 6)

// Good! Next, register your application service in `api::main`
