use crate::{ApiResponse, AppConfig, ApplicationErrorBox, get_session_id};
use actix_web::web::{Data, Payload};
use actix_web::{HttpRequest, HttpResponse, get};
use actix_ws::AggregatedMessage;
use base::application::commands::message::HistoryMessageQuery;
use base::application::service::message::{MessageApplicationService, NotifyDTO};
use base::domain::model::session::SessionId;
use base::domain::service::message::MessageListenerService;
use base::domain::service::session::SessionManagerService;
use base::infrastructure::service::message::MessageListenerImpl;
use dyn_fmt::AsStrFormatExt;
use shared::{
    API_FORBIDDEN_CODE, API_FORBIDDEN_MESSAGE_TEMPLATE, API_INTERNAL_SERVER_ERROR_MESSAGE,
};
use tokio_stream::StreamExt;

#[get("/endpoint")]
async fn get_websocket_endpoint(
    app_config: Data<AppConfig>,
) -> Result<ApiResponse<String>, ApplicationErrorBox> {
    let endpoint = format!("{}/{}", app_config.server_name, "/api/notify/ws");

    ApiResponse::ok(endpoint)
}

async fn ws(
    req: HttpRequest,
    stream: Payload,
    session_manager_service: Data<dyn SessionManagerService>,
    message_listener_service: Data<dyn MessageListenerService>,
) -> Result<HttpResponse, actix_web::Error> {
    if let Ok(session_id) = get_session_id(&req) {
        if let Ok(session_id) = SessionId::try_from(session_id.as_str()) {
            if let Ok(user_id) = session_manager_service
                .get_user_id_by_session(session_id)
                .await
            {
                if let Some(user_id) = user_id {
                    let (res, mut session, stream) = actix_ws::handle(&req, stream)?;

                    let mut stream = stream
                        .aggregate_continuations()
                        // aggregate continuation frames up to 1MiB
                        .max_continuation_size(2_usize.pow(20));

                    let message_listener = MessageListenerImpl::new(session.clone());

                    message_listener_service.add_listener(user_id, Box::new(message_listener));

                    // start task but don't wait for it
                    actix_web::rt::spawn(async move {
                        // receive messages from websocket
                        while let Some(msg) = stream.next().await {
                            if let Ok(AggregatedMessage::Ping(msg)) = msg {
                                // respond to PING frame with PONG frame
                                session.pong(&msg).await.unwrap();
                            }

                            // 忽略客户端发送的其他消息
                        }
                    });

                    // respond immediately with response connected to WS session
                    return Ok(res);
                }
            } else {
                return Ok(
                    HttpResponse::Ok().body(serde_json::to_vec(&ApiResponse::<()> {
                        code: 500,
                        message: API_INTERNAL_SERVER_ERROR_MESSAGE.to_string(),
                        data: None,
                    })?),
                );
            }
        }
    }

    Ok(
        HttpResponse::Ok().body(serde_json::to_vec(&ApiResponse::<()> {
            code: API_FORBIDDEN_CODE,
            message: API_FORBIDDEN_MESSAGE_TEMPLATE.format(&["invalid session id"]),
            data: None,
        })?),
    )
}

#[get("/history")]
pub async fn get_history(
    requests: HttpRequest,
    message_application_service: Data<dyn MessageApplicationService>,
) -> Result<ApiResponse<Vec<NotifyDTO>>, ApplicationErrorBox> {
    let session_id = get_session_id(&requests)?;

    let query = HistoryMessageQuery { session_id };

    let result = message_application_service.get_history(query).await?;

    ApiResponse::ok(result)
}

pub fn scoped_config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(get_websocket_endpoint)
        .service(get_history)
        .service(actix_web::web::resource("/ws").route(actix_web::web::get().to(ws)));
}
