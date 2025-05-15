// Step 1: Read the sentences below.
// Thinking 1.2.1D - 6: 你认为下面的句子来自哪个游戏？其用意是什么？
// 用意是夹带私货，宣传“管理式民主”
// Thinking 1.2.1D - 7: 什么是“管理式民主”（Managed Democracy）？你认为它是真实的民主吗？
// “管理式民主”是一个讽刺的说法，指的是一种表面上看似民主的制度，但实际上是由少数人或特定利益集团控制的。它并不是真正的民主，因为它缺乏真正的选举自由和公民参与。
/*
 * Super Earth.
 * Our home.
 * Prosperity.
 * Liberty.
 * (Hi there)
 * (Hey)
 * Democracy.
 * Our way of life.
 * (Hello)
 * But freedom doesn't come free.
 * No...
 * sweet Liberty...
 * NOOOO!
 * (laughs) Look familiar?
 * Scenes like these are happening all over the galaxy, right now!
 * You could be next.
 * That is, unless you make the most important decision of your life.
 * Prove to yourself that you have the strength and the courage to be free.
 * Join...the Helldivers.
 *  Become part of an elite peacekeeping force!
 * See exotic new lifeforms.
 * And spread Managed Democracy throughout the galaxy.
 * Become a HERO.
 * Become a LEGEND.
 * Become a Helldiver!
 */
use actix_web::{App, HttpServer, web};
use api::MAX_BODY_LENGTH;
use base::domain::model::session_config::SessionConfig;
use base::domain::repository::session::SessionRepositoryConfig;
use base::infrastructure::application::service::user_manager::UserManagerServiceImpl;
use base::infrastructure::repository::session::SessionRepositoryImpl;
use base::infrastructure::repository::user::UserRepositoryImpl;
use base::infrastructure::service::password::Argon2PasswordServiceImpl;
use base::infrastructure::service::session::SessionManagerServiceImpl;
use base::infrastructure::service::user::UserServiceImpl;
use migration::MigratorTrait;
use sea_orm::Database;
use std::sync::Arc;
use std::{env, fs};
use tracing::{instrument, warn};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let database_url = read_file_env("DATABASE_URL").expect("cannot get database url");

    let conn = Database::connect(&database_url)
        .await
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));

    migration::Migrator::up(&conn, None)
        .await
        .unwrap_or_else(|_| panic!("Error applying migration to {}", database_url));

    let session_manager_service =
        web::Data::new(SessionManagerServiceImpl::<SessionRepositoryImpl>::new(
            Arc::new(SessionRepositoryImpl::new(
                SessionRepositoryConfig::default(),
            )),
            SessionConfig::default(),
        ));

    let user_repository = web::Data::new(UserRepositoryImpl::new(conn.clone()));

    let user_service = web::Data::new(UserServiceImpl::<_, Argon2PasswordServiceImpl>::new(
        Arc::clone(&user_repository),
    ));

    let user_manager_service = web::Data::new(UserManagerServiceImpl::new(
        Arc::clone(&user_service),
        Arc::clone(&user_repository),
        Arc::clone(&session_manager_service),
    ));

    // Step 2: Create instance of your application service,
    // and wrap it with `web::Data::new`
    // HINT: You can borrow web::Data<T> as &Arc<T>
    // that means you can pass a &web::Data<T> to `Arc::clone`
    // Exercise 1.2.1D - 6: Your code here. (1 / 2)
    // let train_query_service = web::Data::new(
    //     base::infrastructure::application::service::train_query::TrainQueryServiceImpl::<
    //         base::infrastructure::application::service::train_query,
    //     >::new(Arc::clone(&user_repository)),
    // );

    HttpServer::new(move || {
        App::new()
            .app_data(session_manager_service.clone())
            .app_data(user_repository.clone())
            .app_data(user_service.clone())
            .app_data(user_manager_service.clone())
            // Step 3: Register your application service using `.app_data` function
            // Exercise 1.2.1D - 6: Your code here. (2 / 2)
            // Thinking 1.2.1D - 8: `App::new().app_data(...).app_data(...)`是什么设计模式的体现？
            // Good! Next, build your API endpoint in `api::train::schedule`
            .app_data(web::PayloadConfig::default().limit(MAX_BODY_LENGTH))
            .service(
                web::scope("/api").service(web::scope("/user").configure(api::user::scoped_config)),
            )
        // Step 6: Register your endpoint using `.service()` function
        // Exercise 1.2.1D - 7: Your code here. (5 / 5)
        // Congratulations! You have finished Task 1.2.1D!
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await?;

    Ok(())
}

#[instrument]
fn read_file_env(target_env: &str) -> Option<String> {
    let mut result: Option<String> = None;
    if let Ok(file_path) = env::var(format!("{}_FILE", target_env)) {
        match fs::read_to_string(&file_path) {
            Ok(val) => result = Some(val.trim().to_string()),
            Err(e) => {
                warn!("cannot read env file {}: {}", file_path, e)
            }
        }
    }

    if result.is_none() {
        if let Ok(env_str) = env::var(target_env) {
            result = Some(env_str);
        }
    }

    result
}
