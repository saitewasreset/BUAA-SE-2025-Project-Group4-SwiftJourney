use actix_web::web;
use order::new;
use schedule::query;

pub mod order;
pub mod schedule;

// 配置train模块的路由
pub fn scoped_config(cfg: &mut web::ServiceConfig) {
    // 注册schedule模块的查询接口
    cfg.service(query::query_direct);
    // cfg.service(query::query_indirect);

    // 注册order模块的创建订单接口
    cfg.service(new::create_train_order);
}
