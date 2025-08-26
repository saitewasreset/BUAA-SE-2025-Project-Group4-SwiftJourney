# k8s部署方式

> 注：使用kubectl命令

创建命名空间：

```shell
kubectl create namespace swiftjourney
```

构建镜像：

```shell
eval $(minikube docker-env) && docker build -t swiftjourney-backend:local -f backend/Dockerfile backend
eval $(minikube docker-env) && docker build -t swiftjourney-frontend:local -f frontend/Dockerfile frontend
```

应用：

```shell
kubectl apply -k k8s/overlays/local
```

端口转发：

```shell
kubectl -n swiftjourney port-forward svc/frontend 8080:80
```
