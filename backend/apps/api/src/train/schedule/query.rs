use crate::{ApiResponse, ApplicationErrorBox, parse_request_body};
use actix_web::post;
use actix_web::web::Bytes;

// Step 1: Define your API endpoint
// HINT: You may refer to RFC4 "直达车次查询（US1.2.1）" and "中转车次查询（US3.1.1）" for endpoint URL
// HINT: You may refer to https://actix.rs/docs/application#using-an-application-scope-to-compose-applications
// Thinking 1.2.1D - 9: 为何#[post("/query_direct")]中定义的URL只含真实URL的最后一部分？

// Step 2: Using Extractor to get needed user request data and your application service instance
// HINT: You may refer to https://actix.rs/docs/extractors
// HINT: You may refer to `api::user::user_manager` for example
// HINT: You may use `parse_request_body` function to parse request body as specified type `T`
// Exercise 1.2.1D - 7: Your code here. (1 / 5)

// Step 3: Implement `query_direct`
// HINT: You may refer to `api::user::user_manager` for example
// Exercise 1.2.1D - 7: Your code here. (2 / 5)
// To `api/train/schedule/mod.rs` for following exercise
#[post("/query_direct")]
async fn query_direct(body: Bytes) -> Result<ApiResponse<()>, ApplicationErrorBox> {
    todo!()
}

#[post("/query_indirect")]
async fn query_indirect(body: Bytes) -> Result<ApiResponse<()>, ApplicationErrorBox> {
    todo!()
}
