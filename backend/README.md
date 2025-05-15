# SwiftJourney 后端

## 调试

### 直接运行（增量构建速度快，需配置环境）

环境要求：

- [Rust 工具链 1.86.0+](https://www.rust-lang.org/learn/get-started)
- [PostgreSQL 17](https://www.postgresql.org/download/)

数据库设置：

新建用户、设置用户密码、新建用于存储数据的 Database

配置环境变量：

| 名称             | 含义                                                                    |
|----------------|-----------------------------------------------------------------------|
| `DATABASE_URL` | 连接数据库的 URL：`postgres://<db_user>:<db_passwd>@<db_address>/<database>` |
| `RABBITMQ_URL` | 连接RabbitMQ的 URL：`amqp://<amqp_user>:<amqp_passwd>@<db_address>/`      |

请替换`DATABASE_URL`中的`<db_user>`、`<db_passwd>`、`<db_address>`、`<database>`分别为数据库用户、用户密码、数据库地址、Database

构建后端：

（在`backend`目录下执行）

Debug 模式（不启用优化）：`cargo build`
Release 模式（启用优化）：`cargo build --release`

运行后端：

（在`backend`目录下执行）

Debug 模式（不启用优化）：`cargo run --bin api`
Release 模式（启用优化）：`cargo build --release --bin api`

请确保环境变量`DATABASE_URL`的值已经正确设置，或者，在运行时显式地指定值：

`DATABASE_URL=xxx RABBITMQ_URL=xxx cargo run --bin api`

后端启动后，将监听`8080`端口。

### Docker 运行（增量构建速度慢，无需配置环境）

编译 Docker 镜像（注意，若修改了后端代码，需要重新编译）：

```shell
docker compose build
```

启动后端：

```shell
docker compose up
```

后端启动后，将监听`8080`端口。
