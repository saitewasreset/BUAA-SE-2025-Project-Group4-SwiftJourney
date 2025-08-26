# frontend

# --- build stage ---
FROM node:alpine AS builder

# 设置工作目录
WORKDIR /app

# 1. 拷贝 package.json 和 package-lock.json
# 这样只有当这两个文件变化时，才会重新安装依赖，从而利用Docker的层缓存
COPY package.json package-lock.json ./

# 2. 安装依赖
# npm ci 会根据 package-lock.json 精确安装依赖
RUN npm ci

# 3. 拷贝所有源代码
# 将这一步放在安装依赖之后，这样修改代码不会导致重新安装依赖
COPY . .

# 4. 执行构建
RUN npm run build

# --- production stage ---
FROM nginx:alpine

# 从构建阶段（builder）拷贝编译好的静态文件到 Nginx 的 web 根目录
COPY --from=builder /app/dist /usr/share/nginx/html

# 暴露 80 端口
EXPOSE 80