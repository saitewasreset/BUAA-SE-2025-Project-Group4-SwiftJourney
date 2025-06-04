# SwiftJourney 后端

## 调试

### 直接运行（增量构建速度快，需配置环境）

#### 环境要求

- [Rust 工具链 1.86.0+](https://www.rust-lang.org/learn/get-started)

#### 构建后端

（在`backend`目录下执行）

Debug 模式（不启用优化）：`cargo build`
Release 模式（启用优化）：`cargo build --release`

#### 配置依赖服务

所有用于直接运行的依赖服务已经配置在`docker-compose-local.yaml`中

##### 启动依赖服务

```shell
docker compose -f ./docker-compose-local.yaml up
```

#### 后端运行配置

打开`./backend/.env`配置文件，将其中的`SERVER_NAME`替换为**前端**访问后端服务时的**域名**，**含端口号**。

例如，前端通过`http://127.0.0.1:8080/api/xxx`访问后端，则将`SERVER_NAME`设置为`127.0.0.1:8080`。

#### 运行后端

启动后端前，请完成“配置 MinIO 密钥（只需在第一次启动时配置）”节的配置。

（在`backend`目录下执行）

Debug 模式（不启用优化）：`cargo run --bin api`
Release 模式（启用优化）：`cargo run --release --bin api`

后端启动后，将监听`8080`端口。

### Docker 运行（增量构建速度慢，无需配置环境）

#### 编译 Docker 镜像（注意，若修改了后端代码，需要重新编译）

```shell
docker compose build
```

#### 启动后端

启动后端前，请完成“配置 MinIO 密钥（只需在第一次启动时配置）”节的配置。

```shell
docker compose up
```

后端启动后，将监听`8080`端口。

##### 后端运行配置

打开`docker-compose.yaml`配置文件，将其中的`SERVER_NAME`替换为**前端**访问后端服务时的**域名**，**含端口号**。

例如，前端通过`http://127.0.0.1:8080/api/xxx`访问后端，则将`SERVER_NAME`设置为`127.0.0.1:8080`。

## 配置 MinIO 密钥（只需在第一次启动时配置）

当启动依赖服务，看到类似如下输出时，表示 MinIO 服务已经成功启动：

```text
minio-1     | MinIO Object Storage Server
minio-1     | Copyright: 2015-2025 MinIO, Inc.
minio-1     | License: GNU AGPLv3 - https://www.gnu.org/licenses/agpl-3.0.html
minio-1     | Version: RELEASE.2025-04-22T22-12-26Z (go1.24.2 linux/amd64)
minio-1     |
minio-1     | API: http://172.27.0.3:9000  http://127.0.0.1:9000
minio-1     | WebUI: http://172.27.0.3:9001 http://127.0.0.1:9001
minio-1     |
minio-1     | Docs: https://docs.min.io
```

执行`docker ps`查找`minio`容器的 ID。

例如：

```shell
$ sudo docker  ps
CONTAINER ID   IMAGE                COMMAND                  CREATED          STATUS                    PORTS                                                                                         NAMES
538eee81fe9c   postgres:17-alpine   "docker-entrypoint.s…"   38 minutes ago   Up 38 minutes (healthy)   0.0.0.0:5432->5432/tcp, [::]:5432->5432/tcp                                                   backend-db-1
89244d302faa   rabbitmq             "docker-entrypoint.s…"   38 minutes ago   Up 38 minutes             4369/tcp, 5671/tcp, 15691-15692/tcp, 25672/tcp, 0.0.0.0:5672->5672/tcp, [::]:5672->5672/tcp   backend-rabbitmq-1
627ff67ed99d   minio/minio          "/usr/bin/docker-ent…"   38 minutes ago   Up 38 minutes             0.0.0.0:9000-9001->9000-9001/tcp, [::]:9000-9001->9000-9001/tcp                               backend-minio-1
f73aa59ccf85   redis:alpine         "docker-entrypoint.s…"   38 minutes ago   Up 38 minutes (healthy)   6379/tcp                                                                                      backend-redis-1
```

例子中，`minio`容器的 ID 为`627ff67ed99d`。

使用`docker exec -it <容器ID> bash`进入容器 shell。

**在容器 shell 中**，执行如下命令：

```shell
mc alias set swiftjourney http://127.0.0.1:9000 swiftjourney swiftjourney
mc admin accesskey create --access-key 'ForSuperEarth!' --secret-key 'WorkersOfTheWorld,Unite!' swiftjourney
```

## 加载数据

首先，确保后端以调试模式运行，

- 对于“Dokcer 运行”方式，已经完成相关配置。无需手动操作
- 对于“直接运行方式”，确保设置了`DEBUG`环境变量，例如`DEBUG=1 cargo run --bin api`（Bash）

然后，在`data`目录下**按序**执行如下命令：

```shell
curl -X POST -H "Content-Type: application/json" -d @city.json http://127.0.0.1:8080/api/data/city
curl -X POST -H "Content-Type: application/json" -d @station.json http://127.0.0.1:8080/api/data/station
curl -X POST -H "Content-Type: application/json" -d @train_type.json http://127.0.0.1:8080/api/data/train_type
curl -X POST -H "Content-Type: application/json" -d @train_number.json http://127.0.0.1:8080/api/data/train_number
curl -X POST -H "Content-Type: application/json" -d @hotels.json http://127.0.0.1:8080/api/data/hotel
7z x dish_takeaway.7z
curl -X POST -H "Content-Type: application/json" -d @dish_takeaway.json http://127.0.0.1:8080/api/data/dish_takeaway
rm dish_takeaway.json
```

加载数据后，**请重启一次后端**，以便自动根据加载的车次生成车次调度信息。
