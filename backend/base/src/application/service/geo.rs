use crate::application::ApplicationError;
use async_trait::async_trait;
use std::collections::HashMap;

// province -> vec<city>
pub type CityInfoDTO = HashMap<String, Vec<String>>;

// city -> vec<station>
pub type CityStationInfoDTO = HashMap<String, Vec<String>>;

#[async_trait]
pub trait GeoApplicationService: 'static + Send + Sync {
    async fn get_city_info(&self) -> Result<CityInfoDTO, Box<dyn ApplicationError>>;
    async fn get_city_station_info(&self) -> Result<CityStationInfoDTO, Box<dyn ApplicationError>>;
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt;

    /// 一个简单的错误类型，满足 ApplicationError
    #[derive(Debug)]
    struct TestError {
        code: u32,
        msg: String,
    }

    impl fmt::Display for TestError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{} (code: {})", self.msg, self.code)
        }
    }
    impl std::error::Error for TestError {}
    impl ApplicationError for TestError {
        fn error_code(&self) -> u32 {
            self.code
        }
        fn error_message(&self) -> String {
            self.msg.clone()
        }
    }

    /// 一个临时的实现，用于测试
    struct TestGeoService {
        fail_city_info: bool,
        fail_station_info: bool,
    }

    #[async_trait]
    impl GeoApplicationService for TestGeoService {
        async fn get_city_info(&self) -> Result<CityInfoDTO, Box<dyn ApplicationError>> {
            if self.fail_city_info {
                Err(Box::new(TestError { code: 1001, msg: "city info failed".into() }))
            } else {
                let mut map = HashMap::new();
                map.insert("Guangdong".into(), vec!["Guangzhou".into(), "Shenzhen".into()]);
                Ok(map)
            }
        }

        async fn get_city_station_info(&self) -> Result<CityStationInfoDTO, Box<dyn ApplicationError>> {
            if self.fail_station_info {
                Err(Box::new(TestError { code: 2001, msg: "station info failed".into() }))
            } else {
                let mut map = HashMap::new();
                map.insert("Guangzhou".into(), vec!["StationA".into(), "StationB".into()]);
                Ok(map)
            }
        }
    }

    // ----------- 正反两个测试用例 -------------

    #[tokio::test]
    async fn test_get_city_info_success() {
        let service = TestGeoService { fail_city_info: false, fail_station_info: false };
        let result = service.get_city_info().await;
        assert!(result.is_ok());
        let map = result.unwrap();
        assert_eq!(map["Guangdong"], vec!["Guangzhou".to_string(), "Shenzhen".to_string()]);
    }

    #[tokio::test]
    async fn test_get_city_info_failure() {
        let service = TestGeoService { fail_city_info: true, fail_station_info: false };
        let result = service.get_city_info().await;
        assert!(result.is_err());
        let err = result.err().unwrap();
        assert_eq!(err.error_code(), 1001);
        assert_eq!(err.error_message(), "city info failed");
    }

    #[tokio::test]
    async fn test_get_city_station_info_success() {
        let service = TestGeoService { fail_city_info: false, fail_station_info: false };
        let result = service.get_city_station_info().await;
        assert!(result.is_ok());
        let map = result.unwrap();
        assert_eq!(map["Guangzhou"], vec!["StationA".to_string(), "StationB".to_string()]);
    }

    #[tokio::test]
    async fn test_get_city_station_info_failure() {
        let service = TestGeoService { fail_city_info: false, fail_station_info: true };
        let result = service.get_city_station_info().await;
        assert!(result.is_err());
        let err = result.err().unwrap();
        assert_eq!(err.error_code(), 2001);
        assert_eq!(err.error_message(), "station info failed");
    }
}