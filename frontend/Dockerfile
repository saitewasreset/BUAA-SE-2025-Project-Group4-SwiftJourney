# frontend

# build stage
FROM node:alpine AS build-deps
WORKDIR /app
RUN --mount=type=bind,source=./package.json,target=package.json \
    --mount=type=bind,source=./package-lock.json,target=package-lock.json \
    --mount=type=cache,target=/root/.npm \
    npm ci

FROM build-deps AS fbuild

COPY ./ ./
RUN npm run build

FROM nginx:alpine
RUN rm -rf /usr/share/nginx/html/*
COPY --from=fbuild /app/dist /usr/share/nginx/html

EXPOSE 80