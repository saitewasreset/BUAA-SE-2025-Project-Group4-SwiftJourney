# 北航 软件工程基础课程 团队项目：“风行旅途”火车购票旅游系统

## 项目要求

见`docs/项目4：“风行旅途”需求描述文档.pdf`

## 项目结构

- `frontend`：前端：Vue + Typescript
- `backend`：后端：Rust + Actix Web + SeaORM，数据库：PostgreSQL，消息队列：RabbitMQ
- `data`：项目使用到的城市、火车站、车次、酒店、火车餐数据
- `docs`：UI 设计文档以及需求描述文档
- `k8s`：K8S 微服务部署有关内容
- `JMeter`：压力测试有关内容
- `Postman`：接口集成测试有关内容
- `RFC`：项目成员要求、需求分析文档、接口文档

## 部署方式

项目部署完成后，请使用`./data/load.sh`加载需要的数据（需要 bash、curl）。

加载数据后，**请重启一次后端**，以便自动根据加载的车次生成车次调度信息。

### 单体应用

使用项目根目录的`Dockerfile`可构建应用容器（包含后端、前端，前端文件通过 Actix Web 提供）

要启动完整应用（包含依赖的数据库、对象存储、消息队列等），使用`docker-compose-monolithic.yaml`配置文件启动 Docker Compose。

### 微服务（Docker Compose）

使用`docker-compose.yaml`配置文件启动 Docker Compose 即可。

本配置中使用了单个 DBMS 实例，但不同微服务使用该实例上的不同 Database。

### 微服务（k8s）

详见`k8s`子目录内的文档。

本配置中每个微服务使用不同的 DBMS 实例。

## 团队成员

前端：

- [Ma-HR](https://github.com/Ma-HR)
- [Lancezer](https://github.com/Lancezer)
- [Arekaldi](https://github.com/Arekaldi)

后端：

- [saitewasreset](https://github.com/saitewasreset)
- [DeepChirp](https://github.com/DeepChirp)

## 许可

- 本项目所有代码采用 GNU Affero General Public License 3.0 许可，许可文本见`LICENSE_AGPL.txt`
- 本项目所有文档采用 CC BY-NC-SA 4.0 许可，许可文本见`LICENSE_BYNCSA.txt`

## 项目经验

见[博客文章](https://blog.deepchirp.com/2025/09/06/Beihang-Software-Engineering-Notes-Building-SwiftJourney-with-Rust/)
