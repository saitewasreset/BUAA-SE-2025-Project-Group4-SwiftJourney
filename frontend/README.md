# “风行旅途”前端项目

本项目是“风行旅途”在线火车购票旅游系统的前端部分，基于 Vue 3 和 Vite 构建，使用 TypeScript 进行开发。

## 技术栈

- 核心框架: [Vue 3](https://vuejs.org/)
- 构建工具: [Vite](https://vitejs.dev/)
- 路由管理: [Vue Router](https://router.vuejs.org/)
- 状态管理: [Pinia](https://pinia.vuejs.org/)
- UI 组件库:
  - [Element Plus](https://element-plus.org/)
  - [Ant Design Vue](https://antdv.com/)
- HTTP 请求: [Axios](https://axios-http.com/)
- 单元测试: [Vitest](https://vitest.dev/)
- 端到端测试: [Cypress](https://www.cypress.io/)

## 代码结构

```
├── src/
│   ├── api/         # API 请求模块
│   ├── assets/      # 静态资源（图片、样式等）
│   ├── components/  # 可复用的 Vue 组件
│   ├── constant/    # 常量定义
│   ├── interface/   # TypeScript 类型定义
│   ├── views/       # 页面级视图组件
│   ├── main.ts      # 应用入口文件
│   └── App.vue      # 根组件
├── public/          # 不会经过 Vite 处理的静态资源
├── test/            # 单元测试文件
├── cypress/         # 端到端测试文件
├── Dockerfile       # 用于构建生产环境镜像
├── vite.config.ts   # Vite 配置文件
└── package.json     # 项目依赖与脚本配置
```

## 构建与调试

### 本地开发

启动本地开发服务器，支持热更新。

```bash
npm install
npm run dev
```

服务将默认在 `http://localhost:5173` 启动。

### 构建生产版本

将项目打包为静态文件，输出到 `dist` 目录。

```bash
npm run build
```

### 单元测试

使用 Vitest 运行单元测试。

```bash
npm run test:unit
```

### 端到端（E2E）测试

使用 Cypress 运行端到端测试。

```bash
# 启动测试服务器并打开 Cypress GUI
npm run test:e2e:dev

# 在无头模式下运行所有 E2E 测试
npm run test:e2e
```

### Docker 部署

本项目提供了 `Dockerfile` 用于构建生产环境的 Docker 镜像。该镜像基于 Nginx，用于托管构建好的前端静态文件。

1.  构建镜像:

    ```bash
    docker build -t frontend-app .
    ```

2.  运行容器:

    ```bash
    docker run -d -p 80:80 frontend-app
    ```

    应用将在 `http://localhost` 上可用。
