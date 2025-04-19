//! Mock 用户仓储实现模块
//!
//! 本模块提供了 `UserRepository` 的 Mock 实现，用于测试和开发环境。
//! 该实现使用内存存储，支持基本的 CRUD 操作，并维护了电话和身份证的索引。
use crate::domain::model::user::{IdentityCardId, Phone, User, UserId};
use crate::domain::repository::user::UserRepository;
use crate::domain::{Identifiable, Repository, RepositoryError};
use std::collections::HashMap;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicU64, Ordering},
};

/// Mock 用户仓储实现
///
/// 这个结构体提供了 `UserRepository` 和 `Repository<User>` 的内存实现，
/// 适用于测试场景。它维护了以下数据：
/// - 用户主存储 (ID -> User)
/// - 电话索引 (Phone -> UserID)
/// - 身份证索引 (IdentityCardId -> UserID)
///
/// # 线程安全
/// 使用 `Arc<Mutex<T>>` 保证线程安全，所有操作都是同步的。
///
/// # Examples
/// ```
/// use base::domain::repository::user::UserRepository;
/// use base::infrastructure::repository::mock::user::MockUserRepository;
///
/// let repo = MockUserRepository::new();
/// // 测试代码...
/// ```
#[derive(Debug, Clone)]
pub struct MockUserRepository {
    users: Arc<Mutex<HashMap<UserId, User>>>,
    phone_index: Arc<Mutex<HashMap<Phone, UserId>>>,
    identity_card_index: Arc<Mutex<HashMap<IdentityCardId, UserId>>>,
    next_id: Arc<AtomicU64>,
}

impl Default for MockUserRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl MockUserRepository {
    /// 创建新的 Mock 仓储实例
    ///
    /// 初始化空的存储和索引，并将 ID 计数器设置为 1。
    pub fn new() -> Self {
        MockUserRepository {
            users: Arc::new(Mutex::new(HashMap::new())),
            phone_index: Arc::new(Mutex::new(HashMap::new())),
            identity_card_index: Arc::new(Mutex::new(HashMap::new())),
            next_id: Arc::new(AtomicU64::new(1)),
        }
    }

    /// 生成下一个用户 ID
    ///
    /// 原子地递增内部计数器并返回新的 ID。
    fn generate_get_id(&self) -> UserId {
        self.next_id.fetch_add(1, Ordering::SeqCst).into()
    }
}

impl UserRepository for MockUserRepository {
    async fn find_by_phone(&self, phone: Phone) -> Result<Option<User>, RepositoryError> {
        let phone_index = self.phone_index.lock().unwrap();
        let users = self.users.lock().unwrap();

        phone_index
            .get(&phone)
            .and_then(|id| users.get(id))
            .cloned()
            .pipe(Ok)
    }

    async fn find_by_identity_card_id(
        &self,
        identity_card_id: IdentityCardId,
    ) -> Result<Option<User>, RepositoryError> {
        let identity_index = self.identity_card_index.lock().unwrap();
        let users = self.users.lock().unwrap();

        identity_index
            .get(&identity_card_id)
            .and_then(|id| users.get(id))
            .cloned()
            .pipe(Ok)
    }

    async fn remove_by_phone(&self, phone: Phone) -> Result<(), RepositoryError> {
        let mut phone_index = self.phone_index.lock().unwrap();
        let id = match phone_index.remove(&phone) {
            Some(id) => id,
            None => return Ok(()),
        };

        let mut users = self.users.lock().unwrap();
        let mut identity_index = self.identity_card_index.lock().unwrap();

        if let Some(user) = users.remove(&id) {
            identity_index.remove(&user.user_info().identity_card_id);
        }

        Ok(())
    }
}

impl Repository<User> for MockUserRepository {
    async fn find(&self, id: UserId) -> Result<Option<User>, RepositoryError> {
        Ok(self.users.lock().unwrap().get(&id).cloned())
    }

    async fn remove(&self, aggregate: User) -> Result<(), RepositoryError> {
        let id = match aggregate.get_id() {
            Some(id) => id,
            None => return Ok(()),
        };

        let mut users = self.users.lock().unwrap();
        let user = match users.remove(&id) {
            Some(user) => user,
            None => return Ok(()),
        };

        let mut phone_index = self.phone_index.lock().unwrap();
        let mut identity_index = self.identity_card_index.lock().unwrap();

        phone_index.remove(&user.user_info().phone);
        identity_index.remove(&user.user_info().identity_card_id);

        Ok(())
    }

    async fn save(&self, aggregate: &mut User) -> Result<UserId, RepositoryError> {
        // 处理ID生成和索引更新
        let id = match aggregate.get_id() {
            Some(id) => id,
            None => {
                let new_id = self.generate_get_id();
                aggregate.set_id(new_id);
                new_id
            }
        };

        // 获取所有锁
        let mut users = self.users.lock().unwrap();
        let mut phone_index = self.phone_index.lock().unwrap();
        let mut identity_index = self.identity_card_index.lock().unwrap();

        // 更新前清理旧索引
        if let Some(existing) = users.get(&id) {
            phone_index.remove(&existing.user_info().phone);
            identity_index.remove(&existing.user_info().identity_card_id);
        }

        // 插入新数据
        phone_index.insert(aggregate.user_info().phone.clone(), id);
        identity_index.insert(aggregate.user_info().identity_card_id.clone(), id);
        users.insert(id, aggregate.clone());

        Ok(id)
    }
}

// 工具函数：管道操作
trait Pipe: Sized {
    fn pipe<F, T>(self, f: F) -> T
    where
        F: FnOnce(Self) -> T,
    {
        f(self)
    }
}
impl<T> Pipe for T {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::model::password::{HashedPassword, PasswordSalt};
    use crate::domain::model::user::{Gender, PasswordAttempts, UserInfo};

    const IDENTITY_CARD_ID_STR: [&str; 100] = [
        "110108197703065171",
        "110108195809108150",
        "110108198712170744",
        "110108201012033284",
        "110108200909295962",
        "110108199409234850",
        "110108199307107324",
        "110108197610254571",
        "11010819520924444X",
        "110108200402242554",
        "110108197106063573",
        "110108195308311468",
        "110108196205106925",
        "110108199801150176",
        "110108199411183765",
        "110108196604225075",
        "110108198912295656",
        "11010819500228222X",
        "110108194705043108",
        "110108197205107316",
        "110108194609010672",
        "11010819860420461X",
        "110108195411073496",
        "110108197505135263",
        "110108196812023574",
        "110108200407244778",
        "110108198608256492",
        "110108198309018849",
        "110108199107192915",
        "110108198909165981",
        "110108198908058922",
        "11010820181031905X",
        "110108194507240290",
        "11010819760902623X",
        "110108196705219539",
        "110108195807051277",
        "110108195005278779",
        "110108200606127320",
        "110108198501068408",
        "110108197706225804",
        "110108202105261678",
        "110108201302131632",
        "110108195303069772",
        "11010819910127264X",
        "110108201702178085",
        "110108196004072271",
        "110108201901026148",
        "110108199912136094",
        "110108198708162119",
        "110108197502235728",
        "110108199108171315",
        "110108200809129510",
        "110108194901232635",
        "110108198009089362",
        "110108196912135653",
        "11010819861030473X",
        "11010819780602353X",
        "110108201402057521",
        "11010819700507891X",
        "110108196912196333",
        "110108196110250932",
        "11010819830514213X",
        "110108197306033520",
        "110108200803121702",
        "110108197811150066",
        "110108198012310961",
        "110108198109071493",
        "110108196509077027",
        "110108201301176660",
        "110108199707219683",
        "110108202104140778",
        "110108194608144783",
        "110108194901107834",
        "110108199004268884",
        "110108201106238336",
        "110108202310193053",
        "11010819480508779X",
        "110108196704078594",
        "11010819720104926X",
        "110108198205210174",
        "11010819830623217X",
        "110108201902285563",
        "110108202309129185",
        "110108201703126236",
        "110108195801153360",
        "110108195503095131",
        "110108201608195102",
        "110108199508176262",
        "110108199208291429",
        "110108199703156312",
        "110108201504246286",
        "110108197211149467",
        "110108196707088069",
        "110108202110013943",
        "110108201611298997",
        "110108198505198015",
        "110108196501131666",
        "110108194811073212",
        "110108196608025361",
        "110108198501140940",
    ];

    // 测试辅助函数：创建测试用户
    fn create_test_user(phone: &str, identity: &str) -> User {
        let password_salt = PasswordSalt::from(vec![0u8; 32]);

        let hashed_password = HashedPassword {
            salt: password_salt.clone(),
            hashed_password: vec![0u8; 64],
        };

        let user_info = UserInfo::new(
            "Test User".into(),
            Some(Gender::Male),
            Some(30.try_into().unwrap()),
            Phone::try_from(phone.to_owned()).unwrap(),
            None,
            IdentityCardId::try_from(identity.to_owned()).unwrap(),
        );

        User::new(
            None,
            "test_user".into(),
            hashed_password,
            None,
            PasswordAttempts::new(),
            user_info,
        )
    }

    #[tokio::test]
    async fn test_save_new_user_generates_get_id() {
        let repo = MockUserRepository::new();
        let user = create_test_user("13800000001", IDENTITY_CARD_ID_STR[0]);

        // 首次保存
        let saved_user = {
            let mut user = user.clone();
            repo.save(&mut user).await.unwrap();
            user
        };

        assert!(saved_user.get_id().is_some(), "应该生成用户ID");

        // 验证存储
        let users = repo.users.lock().unwrap();
        assert_eq!(users.len(), 1, "用户应该被存储");
    }

    #[tokio::test]
    async fn test_find_by_get_id() {
        let repo = MockUserRepository::new();
        let user = create_test_user("13800000002", IDENTITY_CARD_ID_STR[1]);

        // 保存用户
        let user_id = {
            let mut user = user.clone();
            repo.save(&mut user).await.unwrap()
        };

        // 正常查找
        let found = repo.find(user_id).await.unwrap();
        assert!(found.is_some(), "应该能找到用户");
        assert_eq!(found.unwrap().user_info().phone, user.user_info().phone);

        // 查找不存在的ID
        let non_existent_id = UserId::from(9999);
        let not_found = repo.find(non_existent_id).await.unwrap();
        assert!(not_found.is_none(), "不应该找到不存在的用户");
    }

    #[tokio::test]
    async fn test_phone_index_maintenance() {
        let repo = MockUserRepository::new();
        let phone = "13800000003";
        let user = create_test_user(phone, IDENTITY_CARD_ID_STR[2]);

        // 初始保存
        {
            let mut user = user.clone();
            repo.save(&mut user).await.unwrap();
        }

        // 验证电话索引
        let found = repo
            .find_by_phone(Phone::try_from(phone.to_owned()).unwrap())
            .await
            .unwrap();
        assert!(found.is_some(), "应该通过电话号码找到用户");

        // 更新电话号码
        let new_phone = "13800000004";
        {
            let mut user = repo
                .find_by_phone(Phone::try_from(phone.to_owned()).unwrap())
                .await
                .unwrap()
                .unwrap();
            user.user_info_mut().phone = Phone::try_from(new_phone.to_owned()).unwrap();
            repo.save(&mut user).await.unwrap();
        }

        // 旧号码应该不存在
        let old_lookup = repo
            .find_by_phone(Phone::try_from(phone.to_owned()).unwrap())
            .await
            .unwrap();
        assert!(old_lookup.is_none(), "旧电话号码应该被移除");

        // 新号码应该存在
        let new_lookup = repo
            .find_by_phone(Phone::try_from(new_phone.to_owned()).unwrap())
            .await
            .unwrap();
        assert!(new_lookup.is_some(), "新电话号码应该被索引");
    }

    #[tokio::test]
    async fn test_identity_card_index() {
        let repo = MockUserRepository::new();
        let identity = IDENTITY_CARD_ID_STR[3];
        let user = create_test_user("13800000005", identity);

        // 保存用户
        {
            let mut user = user.clone();
            repo.save(&mut user).await.unwrap();
        }

        // 验证身份证索引
        let found = repo
            .find_by_identity_card_id(IdentityCardId::try_from(identity.to_owned()).unwrap())
            .await
            .unwrap();
        assert!(found.is_some(), "应该通过身份证号找到用户");

        // 更新身份证号
        let new_identity = IDENTITY_CARD_ID_STR[5];
        {
            let mut user = repo
                .find_by_phone(Phone::try_from("13800000005".to_owned()).unwrap())
                .await
                .unwrap()
                .unwrap();
            user.user_info_mut().identity_card_id =
                IdentityCardId::try_from(new_identity.to_owned()).unwrap();
            repo.save(&mut user).await.unwrap();
        }

        // 旧身份证应该不存在
        let old_lookup = repo
            .find_by_identity_card_id(IdentityCardId::try_from(identity.to_owned()).unwrap())
            .await
            .unwrap();
        assert!(old_lookup.is_none(), "旧身份证号应该被移除");

        // 新身份证应该存在
        let new_lookup = repo
            .find_by_identity_card_id(IdentityCardId::try_from(new_identity.to_owned()).unwrap())
            .await
            .unwrap();
        assert!(new_lookup.is_some(), "新身份证号应该被索引");
    }

    #[tokio::test]
    async fn test_remove_user() {
        let repo = MockUserRepository::new();
        let user = create_test_user("13800000006", IDENTITY_CARD_ID_STR[4]);

        // 保存用户

        let mut user = user.clone();
        let user_id = repo.save(&mut user).await.unwrap();

        *user.id_mut() = Some(user_id);

        // 删除用户
        repo.remove(user.clone()).await.unwrap();

        // 验证所有存储
        {
            let users = repo.users.lock().unwrap();
            assert!(users.is_empty(), "用户应该被移除");
        }

        {
            let phone_index = repo.phone_index.lock().unwrap();
            assert!(phone_index.is_empty(), "电话号码索引应该被清除");
        }

        {
            let identity_index = repo.identity_card_index.lock().unwrap();
            assert!(identity_index.is_empty(), "身份证索引应该被清除");
        }

        // 二次删除应该无害
        let result = repo.remove(user.clone()).await;
        assert!(result.is_ok(), "重复删除应该成功");
    }

    #[tokio::test]
    async fn test_remove_by_phone() {
        let repo = MockUserRepository::new();
        let phone = "13800000007";
        let user = create_test_user(phone, IDENTITY_CARD_ID_STR[0]);

        // 保存用户
        {
            let mut user = user.clone();
            repo.save(&mut user).await.unwrap();
        }

        // 通过电话删除
        repo.remove_by_phone(Phone::try_from(phone.to_owned()).unwrap())
            .await
            .unwrap();

        // 验证删除
        let found = repo
            .find_by_phone(Phone::try_from(phone.to_owned()).unwrap())
            .await
            .unwrap();
        assert!(found.is_none(), "用户应该已被删除");

        let users = repo.users.lock().unwrap();
        assert!(users.is_empty(), "主存储应该为空");
    }

    #[tokio::test]
    async fn test_concurrent_access() {
        let repo = Arc::new(MockUserRepository::new());
        let phone_prefix = "1380000";

        let handles: Vec<_> = (0..100)
            .map(|i| {
                let repo = Arc::clone(&repo);
                tokio::spawn(async move {
                    let phone = format!("{}{:04}", phone_prefix, i);
                    let user = create_test_user(&phone, IDENTITY_CARD_ID_STR[i]);

                    // 保存用户
                    let mut user = user.clone();
                    repo.save(&mut user).await.unwrap();

                    // 验证查找
                    let found = repo
                        .find_by_phone(Phone::try_from(phone).unwrap())
                        .await
                        .unwrap();
                    assert!(found.is_some(), "应该能找到用户 {}", i);

                    // 删除用户
                    repo.remove(user.clone()).await.unwrap();
                })
            })
            .collect();

        for handle in handles {
            handle.await.unwrap();
        }

        // 最终验证
        let users = repo.users.lock().unwrap();
        assert!(users.is_empty(), "所有用户应该已被删除");

        let phone_index = repo.phone_index.lock().unwrap();
        assert!(phone_index.is_empty(), "电话号码索引应该为空");
    }

    #[tokio::test]
    async fn test_id_generation_sequence() {
        let repo = MockUserRepository::new();

        let mut last_id = 0;
        for i in 0..10 {
            let mut user = create_test_user(&format!("1380000{:04}", i), IDENTITY_CARD_ID_STR[i]);
            repo.save(&mut user).await.unwrap();

            let current_id = user.get_id().unwrap().into();
            assert!(current_id > last_id, "ID应该单调递增");
            last_id = current_id;
        }
    }
}
