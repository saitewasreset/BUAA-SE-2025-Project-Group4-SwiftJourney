use crate::application::ApplicationError;
use crate::application::commands::hotel_data::LoadHotelCommand;
use async_trait::async_trait;

#[async_trait]
pub trait HotelDataService: 'static + Send + Sync {
    /// 检查服务是否处于调试模式
    fn is_debug_mode(&self) -> bool;

    /// 加载酒店数据
    ///
    /// 根据[`LoadHotelCommand`]提供的信息创建或更新酒店数据。
    ///
    /// # Arguments
    /// * `command` - 包含酒店数据的命令对象
    async fn load_hotel(&self, command: LoadHotelCommand) -> Result<(), Box<dyn ApplicationError>>;
}


#[cfg(test)]
mod tests {
    use super::*;
    use shared::data::HotelInfo;
    use std::collections::HashMap;

    // ---- 测试专用实现 ----
    struct TestHotelDataService {
        debug_mode: bool,
        fail_load: bool,
    }

    #[async_trait]
    impl HotelDataService for TestHotelDataService {
        fn is_debug_mode(&self) -> bool {
            self.debug_mode
        }

        async fn load_hotel(&self, _command: LoadHotelCommand) -> Result<(), Box<dyn ApplicationError>> {
            if self.fail_load {
                Err(Box::new(TestError { code: 500, msg: "Load failed".into() }))
            } else {
                Ok(())
            }
        }
    }

    // ---- 测试用的简单 ApplicationError ----
    #[derive(Debug)]
    struct TestError {
        code: u32,
        msg: String,
    }

    impl std::fmt::Display for TestError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.msg)
        }
    }

    impl std::error::Error for TestError {}

    impl ApplicationError for TestError {
        fn error_code(&self) -> u32 { self.code }
        fn error_message(&self) -> String { self.msg.clone() }
    }

    // ---- 测试用例 ----

    #[test]
    fn test_is_debug_mode_true() {
        let service = TestHotelDataService { debug_mode: true, fail_load: false };
        assert!(service.is_debug_mode());
    }

    #[test]
    fn test_is_debug_mode_false() {
        let service = TestHotelDataService { debug_mode: false, fail_load: false };
        assert!(!service.is_debug_mode());
    }

    #[tokio::test]
    async fn test_load_hotel_success() {
        let service = TestHotelDataService { debug_mode: false, fail_load: false };

        let hotel = HotelInfo {
            name: "Test Hotel".into(),
            address: "123 Test St".into(),
            city: "Guangzhou".into(),
            station: None,
            images: vec![],
            phone: vec!["12345678".into()],
            info: "Nice hotel".into(),
            room_info: HashMap::new(),
            comments: vec![],
        };

        let command: LoadHotelCommand = vec![hotel];

        let res = service.load_hotel(command).await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_load_hotel_failure() {
        let service = TestHotelDataService { debug_mode: false, fail_load: true };

        let hotel = HotelInfo {
            name: "Test Hotel".into(),
            address: "123 Test St".into(),
            city: "Guangzhou".into(),
            station: None,
            images: vec![],
            phone: vec!["12345678".into()],
            info: "Nice hotel".into(),
            room_info: HashMap::new(),
            comments: vec![],
        };

        let command: LoadHotelCommand = vec![hotel];

        let res = service.load_hotel(command).await;
        assert!(res.is_err());
        let err = res.err().unwrap();
        assert_eq!(err.error_message(), "Load failed");
    }

}
