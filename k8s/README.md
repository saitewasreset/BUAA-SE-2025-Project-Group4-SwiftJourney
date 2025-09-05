# K8s 部署方式（微服务 + 网关）

> 使用 `kubectl` 命令；后端已拆分为多个微服务，并通过网关统一对外。

为使用自动扩容可安装 VPA（Vertical Pod Autoscaler）：
<https://github.com/kubernetes/autoscaler/blob/master/vertical-pod-autoscaler/docs/installation.md>

## 1. 启动集群

```bash
minikube start
```

命名空间可由 Kustomize 自动创建；如需手动：

```bash
kubectl create namespace swiftjourney || true
```

## 2. 构建镜像（Minikube 内置 Docker）

将 Docker 指向 Minikube 守护进程，这样构建的镜像可直接被集群使用：

```bash
eval $(minikube docker-env)
```

后端每个微服务使用通用 `backend/Dockerfile.ms`，通过 `--build-arg BIN=...` 指定产物；标签需与 `k8s/base/kustomization.yaml` 对齐（默认 `v0.1.0`）。

```bash
# 后端微服务
docker build -f backend/Dockerfile.ms --build-arg BIN=user_api            -t saitewasreset/swiftjourney-user:v0.1.0           backend &&
docker build -f backend/Dockerfile.ms --build-arg BIN=geo_api             -t saitewasreset/swiftjourney-geo:v0.1.0            backend &&
docker build -f backend/Dockerfile.ms --build-arg BIN=train_api           -t saitewasreset/swiftjourney-train:v0.1.0          backend &&
docker build -f backend/Dockerfile.ms --build-arg BIN=hotel_api           -t saitewasreset/swiftjourney-hotel:v0.1.0          backend &&
docker build -f backend/Dockerfile.ms --build-arg BIN=dish_api            -t saitewasreset/swiftjourney-dish:v0.1.0           backend &&
docker build -f backend/Dockerfile.ms --build-arg BIN=order_api           -t saitewasreset/swiftjourney-order:v0.1.0          backend &&
docker build -f backend/Dockerfile.ms --build-arg BIN=object_storage_api  -t saitewasreset/swiftjourney-object-storage:v0.1.0 backend

# 前端
docker build -f frontend/Dockerfile -t saitewasreset/swiftjourney-frontend:v0.1.0 frontend
```

提示：若 overlays/local 修改了镜像标签，请同步调整上面命令或 overlay 配置。

## 3. 应用清单

```bash
# 首先运行 metrics-server 统计使用率
kubectl pply -f k8s/bse/metrics-server-officil.yaml
kubectl apply -k k8s/overlays/local
```

网关（`gateway` Service）会将请求按路径分发到对应微服务：

- `/api/user` → user
- `/api/general` → geo
- `/api/train` → train
- `/api/hotel` → hotel
- `/api/dish` → dish
- `/api/order` → order
- `/resource` → object-storage

## 4. 端口转发

```bash
kubectl -n swiftjourney port-forward svc/gateway 8080:80
```

访问：

- 前端：<http://localhost:8080>
- API：经网关按上述前缀访问。

## 5. 重新加载

重新应用配置：

```bash
kubectl apply -k k8s/overlays/local
```

重启网关：

```bash
kubectl -n swiftjourney rollout restart deploy/gateway
```

重新构建镜像：

```bash
eval $(minikube docker-env)

# build missing images
BIN_LIST=(dish_api geo_api hotel_api order_api object_storage_api train_api user_api)
for bin in "${BIN_LIST[@]}"; do
  docker build -f backend/Dockerfile.ms --build-arg BIN="$bin" -t saitewasreset/swiftjourney-"${bin%_api}":v0.1.0 backend || exit 1
done
```

应用新镜像：

```bash
kubectl -n swiftjourney rollout restart deploy
```
