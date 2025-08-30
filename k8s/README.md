# k8s部署方式

> 注：使用`kubectl`命令

<!-- 为了使用自动扩容，需安装VPA（Vertical Pod Autoscaler），参见：<https://github.com/kubernetes/autoscaler/blob/master/vertical-pod-autoscaler/docs/installation.md> -->

启动`minikube`：

```shell
minikube start
```

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
kubectl -n swiftjourney port-forward svc/gateway 8080:80
```
