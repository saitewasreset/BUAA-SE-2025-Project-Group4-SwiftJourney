import { setupServer } from "msw/node";
import { http } from "msw";
import { beforeAll, afterEach, afterAll } from "vitest";

export const server = setupServer(
  http.get("/api/general/city", () => {
    return Response.json({
      code: 200,
      data: {
        '北京': ['北京', '北京西', '北京南', '北京东'],
        '上海': ['上海', '上海虹桥', '上海南'],
        '广州': ['广州', '广州东', '广州南'],
        '深圳': ['深圳', '深圳北', '深圳东']
      }
    })
  }),
  http.get("/api/general/city_stations", () => {
    return Response.json({
      code: 200,
      data: {
        '北京': ['北京', '北京西', '北京南', '北京东'],
        '上海': ['上海', '上海虹桥', '上海南'],
        '广州': ['广州', '广州东', '广州南'],
        '深圳': ['深圳', '深圳北', '深圳东']
      }
    })
  }),
  http.get("/test", () => {
    return Response.json({ msg: "ok" });
  }),
  http.post("/test", () => {
    return Response.json({ msg: "posted" });
  }),
  http.post("/test/blob", () => {
    return Response.json({ msg: "posted" });
  }),
  http.put("/test", () => {
    return Response.json({ msg: "updated" });
  }),
  http.delete("/test", () => {
    return Response.json({ msg: "deleted" });
  }),
  // 添加通用的 API 路由
  http.get("/api/*", () => {
    return Response.json({ success: true, data: {} });
  }),
  http.post("/api/*", () => {
    return Response.json({ success: true, data: {} });
  }),
  http.put("/api/*", () => {
    return Response.json({ success: true, data: {} });
  }),
  http.delete("/api/*", () => {
    return Response.json({ success: true, data: {} });
  }),
  http.get("/nonexistent", () => {
    return new Response(null, { status: 404 })
  })
);

// 在所有测试前后启动/关闭 server
beforeAll(() => {
  server.listen({ onUnhandledRequest: 'warn' }); // 改为 warn 而不是 error
});

afterEach(() => {
  server.resetHandlers();
});

afterAll(() => {
  server.close();
});
