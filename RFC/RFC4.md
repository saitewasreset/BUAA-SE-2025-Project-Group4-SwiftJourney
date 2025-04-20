# Request For Comments 4: API 文档

Version: 4 (2025-04-20 11:15:00)

最近变更：

- Version 4
  - 用户注册：新增了部分校验失败的错误代码
  - 设置个人资料：新增了部分校验失败的错误代码
- Version 3
  - 用户注册：要求填写姓名以及身份证号
  - 用户登录：移除判断是否是第一次登录功能，响应数据变为`null`
  - 获取个人资料：返回值新增姓名和身份证号
- Version 2
  - 用户注册：使用手机号进行注册
  - 用户登录：使用手机号进行登录，响应数据变为`UserLoginInfo`
  - 获取个人资料：获取时`email`可能为空
  - 设置个人资料：请求数据变为`UserUpdateInfo`
  - 支付订单：更改 API 端点为`/api/transaction/pay/{transaction_id}`
  - 订单列表、订单详情：
    - `OrderInfo`新增`orderTime`、`canCancel`、`reason`
    - 拆分人类可读的座位信息为`SeatLocationInfo`
    - 新增`TakeawayOrderInfo`
  - 获取个人信息：新增`default`属性
  - 设置个人信息：修改请求内容，新增`default`属性
  - 火车餐预订：新增外卖预订
  - 车次查询：
    - `StoppingStationInfo`明确了始发站、终到站的离开时间、到达时间处理
    - 支持按城市查询
  - 新增：运行模式
  - 新增：城市车站信息
  - 新增：生成测试订单

关于火车站点及货币的约定，见 RFC3。

API 请求/返回类型采用 TypeScript 表达，[参见](https://www.typescriptlang.org/docs/handbook/intro.html)。

`./RFC/attachment/RFC4/typing.ts`中包含了下文中的所有类型定义，可直接在前端使用。

对于所有 API，若无特别说明：

- 对于`GET`请求，请求体为空
- 对于`POST`请求，请求体为 JSON
- 响应体为 JSON，且符合如下格式：

  ```typescript
  interface APIResponse<T> {
    code: number;
    message: string;
    data: T;
  }
  ```

  - 下文中的“响应**代码**”指的是`APIResponse`的`code`属性
  - 下文中的“响应**消息**”指的是`APIResponse`的`message`属性
  - 下文中的“响应**数据**”指的是`APIResponse`的`data`属性

“响应**代码**”类别：

- `<1000` 的响应代码为通用响应代码，所有请求通用
- `>=1000` 的响应代码为业务响应代码，依据不同请求而不同

通用响应代码表：

其中`{}`为占位符。

| 代码 | 可能的响应消息                                                     | 含义                                                                                                                                                         |
| ---- | ------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| 200  | `For Super Earth!`                                                 | 请求已被成功执行，可访问响应数据                                                                                                                             |
| 400  | `{reason}`                                                         | 请求存在错误，具体地：传入内容无法作为 JSON 解析；传入内容能作为 JSON 解析但不符合本文定义的格式。若格式符合，但内容存在问题，不返回 400，而返回业务响应代码 |
| 403  | `Sorry, but this was meant to be a private game: {reason}`         | 没有权限执行该请求                                                                                                                                           |
| 404  | `Sorry, but this was meant to be a private game: {reason}`         | 访问的资源不存在，或没有权限访问该资源                                                                                                                       |
| 500  | `Multiplayer Session Ended: an internal server error has occurred` | 内部服务器错误                                                                                                                                               |

日期与时间：

- 日期时间（精确到某一天的时分秒）处理：采用 ISO 8601 格式，例如：`2023-10-05T14:30:00Z`，JS 中请使用`new Date("2023-10-05T14:30:00Z")`处理
- 日期（精确到某一天）处理：采用 ISO 8601 格式，例如：`2023-10-05`，JS 中请使用`new Date("2023-10-05")`
- 时间（任意一天中的特定时刻）：采用从 00:00:00 开始的秒数

## 全局状态

### 运行模式

`GET /api/general/mode`

若后端运行在“debug”模式，前端可显示调试用的数据、操作。

需要 Cookie：

- 无

响应代码表：

| 代码 | 可能的响应消息     | 含义                             |
| ---- | ------------------ | -------------------------------- |
| 200  | `For Super Earth!` | 请求已被成功执行，可访问响应数据 |

响应**数据**：

```typescript
type ResponseData = "debug" | "release";
```

### 城市车站信息

`GET /api/general/city_stations`

返回从城市到该城市车站列表的映射。

需要 Cookie：

- 无

响应代码表：

| 代码 | 可能的响应消息     | 含义                             |
| ---- | ------------------ | -------------------------------- |
| 200  | `For Super Earth!` | 请求已被成功执行，可访问响应数据 |

响应**数据**：

```typescript
// 城市 -> 车站 []
type ResponseData = Map<string, string[]>;
```

## 用户管理（FE1.3 FE1.5）

### 用户注册（US1.5.1）

`POST /api/user/register`

需要 Cookie：

- 无

请求：

```typescript
type Request = UserRegisterRequest;

interface UserRegisterRequest {
  // 手机号
  phone: string;
  username: string;
  // 明文密码
  password: string;
  // 姓名
  name: string;
  // 身份证号
  identityCardId: string;
}
```

响应代码表：

| 代码  | 可能的响应消息                 | 含义                       |
| ----- | ------------------------------ | -------------------------- |
| 200   | `For Super Earth!`             | 注册成功                   |
| 13001 | `Identity card id format`      | 身份证号格式错误           |
| 15001 | `Phone {phone} already exists` | 该手机号对应的用户已经存在 |
| 15003 | `Invalid username`             | 用户名格式错误             |
| 15004 | `Invalid password`             | 密码格式错误               |
| 15005 | `Invalid name`                 | 姓名格式错误               |

响应**数据**：

```typescript
type ResponseData = null;
```

### 用户登录（US1.5.2）

`POST /api/user/login`

需要 Cookie：

- 无

请求：

```typescript
type Request = UserLoginRequest;

interface UserLoginRequest {
  phone: string;
  // 明文密码
  password: string;
}
```

响应代码表：

| 代码  | 可能的响应消息                     | 含义             |
| ----- | ---------------------------------- | ---------------- |
| 200   | `For Super Earth!`                 | 登录成功         |
| 15002 | `Invalid phone number or password` | 用户名或密码错误 |

响应**数据**：

```typescript
type ResponseData = null;
```

设置 Cookie：

- session_id

### 用户注销（US1.5.3）

`POST /api/user/logout`

需要 Cookie：

- session_id

请求：

```typescript
type Request = null;
```

响应代码表：

| 代码 | 可能的响应消息                                                       | 含义     |
| ---- | -------------------------------------------------------------------- | -------- |
| 200  | `For Super Earth!`                                                   | 注销成功 |
| 403  | `Sorry, but this was meant to be a private game: invalid session_id` | 会话无效 |

响应**数据**：

```typescript
type ResponseData = null;
```

设置 Cookie：

- 无

### 获取个人资料（US1.5.4）

`GET /api/user/user_info`

需要 Cookie：

- session_id

响应代码表：

| 代码 | 可能的响应消息                                                       | 含义                             |
| ---- | -------------------------------------------------------------------- | -------------------------------- |
| 200  | `For Super Earth!`                                                   | 请求已被成功执行，可访问响应数据 |
| 403  | `Sorry, but this was meant to be a private game: invalid session_id` | 会话无效                         |

响应**数据**：

```typescript
type ResponseData = UserInfo;
interface UserInfo {
  username: string;
  gender?: "male" | "female";
  age?: number;
  phone: string;
  email?: string;
  // 当前用户是否设置了支付密码
  havePaymentPasswordSet: boolean;
  // 姓名
  name: string;
  // 身份证号
  identityCardId: string;
}
```

设置 Cookie：

- 无

### 设置个人资料（US1.5.4）

`POST /api/user/user_info`

注：设置资料时必须设置`email`。

需要 Cookie：

- session_id

请求：

```typescript
type Request = UserUpdateInfo;
interface UserUpdateInfo {
  username: string;
  gender?: "male" | "female";
  age?: number;
  email: string;
}
```

响应代码表：

| 代码  | 可能的响应消息                                                       | 含义                             |
| ----- | -------------------------------------------------------------------- | -------------------------------- |
| 200   | `For Super Earth!`                                                   | 请求已被成功执行，可访问响应数据 |
| 403   | `Sorry, but this was meant to be a private game: invalid session_id` | 会话无效                         |
| 15003 | `Invalid username`                                                   | 用户名格式错误                   |
| 15006 | `Invalid age`                                                        | 年龄格式错误                     |
| 15007 | `Invalid email`                                                      | 邮箱格式错误                     |

响应**数据**：

```typescript
type ResponseData = null;
```

设置 Cookie：

- 无

### 获取个人信息（US1.3.1）

`GET /api/user/personal_info`

需要 Cookie：

- session_id

响应代码表：

| 代码 | 可能的响应消息                                                       | 含义                             |
| ---- | -------------------------------------------------------------------- | -------------------------------- |
| 200  | `For Super Earth!`                                                   | 请求已被成功执行，可访问响应数据 |
| 403  | `Sorry, but this was meant to be a private game: invalid session_id` | 会话无效                         |

响应**数据**：

```typescript
type ResponseData = PersonalInfo[];
interface PersonalInfo {
  // 该用户身份的 UUID
  personalId: string;
  // 姓名
  name: string;
  // 身份证号
  identityCardId: string;
  // 偏好座位位置
  preferredSeatLocation?: "A" | "B" | "C" | "D" | "F";
  // 是否为默认个人资料，即，当前用户的身份
  default: boolean;
}
```

### 设置个人信息（US1.3.1）

`POST /api/user/personal_info`

需要 Cookie：

- session_id

请求：

```typescript
type Request = type Request = UpdatePersonalInfo;

interface UpdatePersonalInfo {
  // 姓名
  name?: string;
  // 身份证号
  identityCardId: string;
  // 偏好座位位置
  preferredSeatLocation?: "A" | "B" | "C" | "D" | "F";
  // 是否为默认个人资料，即，当前用户的身份
  default?: boolean;
}

```

若要新增/更新信息，至少设置`name`、`identityCardId`、`default`字段。
若要删除信息，只设置`identityCardId`字段。

响应代码表：

| 代码  | 可能的响应消息                                                       | 含义                                           |
| ----- | -------------------------------------------------------------------- | ---------------------------------------------- |
| 200   | `For Super Earth!`                                                   | 请求已被成功执行，可访问响应数据               |
| 403   | `Sorry, but this was meant to be a private game: invalid session_id` | 会话无效                                       |
| 13001 | `Identity card id format`                                            | 身份证号格式错误                               |
| 13002 | `Invalid identity card id`                                           | 该身份证号对应的个人信息不存在，或没有权限设置 |

响应**数据**：

```typescript
type ResponseData = null;
```

设置 Cookie：

- 无

## 缴费系统（FE1.1）

### 充值（US1.1.1）

`POST /api/payment/recharge`

需要 Cookie：

- session_id

请求：

```typescript
type Request = RechargeInfo;

interface RechargeInfo {
  amount: number;
  // 由于无需访问外部支付系统，故`externalPaymentId`设置为`null`即可。
  externalPaymentId: null;
}
```

响应代码表：

| 代码 | 可能的响应消息                                                       | 含义                             |
| ---- | -------------------------------------------------------------------- | -------------------------------- |
| 200  | `For Super Earth!`                                                   | 请求已被成功执行，可访问响应数据 |
| 403  | `Sorry, but this was meant to be a private game: invalid session_id` | 会话无效                         |

响应**数据**：

```typescript
type ResponseData = null;
```

设置 Cookie：

- 无

### 余额查询（US1.1.2）

`GET /api/payment/balance`

需要 Cookie：

- session_id

响应代码表：

| 代码 | 可能的响应消息                                                       | 含义                             |
| ---- | -------------------------------------------------------------------- | -------------------------------- |
| 200  | `For Super Earth!`                                                   | 请求已被成功执行，可访问响应数据 |
| 403  | `Sorry, but this was meant to be a private game: invalid session_id` | 会话无效                         |

响应**数据**：

```typescript
type ResponseData = BalanceInfo;

interface BalanceInfo {
  balance: number;
}
```

设置 Cookie：

- 无

### 交易信息查询（US1.1.3）

`GET /api/transaction/{transaction_id}`

需要 Cookie：

- session_id

响应代码表：

| 代码 | 可能的响应消息                                                             | 含义                                   |
| ---- | -------------------------------------------------------------------------- | -------------------------------------- |
| 200  | `For Super Earth!`                                                         | 请求已被成功执行，可访问响应数据       |
| 403  | `Sorry, but this was meant to be a private game: invalid session_id`       | 会话无效                               |
| 404  | `Sorry, but this was meant to be a private game: transaction_id not found` | 访问的交易不存在，或没有权限访问该交易 |

响应**数据**：

```typescript
type ResponseData = TransactionInfo;

interface TransactionInfo {
  transactionId: string;
  amount: number;
  status: "unpaid" | "paid";
}
```

设置 Cookie：

- 无

### 设置支付密码（US1.1.4）

`POST /api/payment/payment_password`

需要 Cookie：

- session_id

请求：

```typescript
type Request = PaymentPasswordInfo;

interface PaymentPasswordInfo {
  // 修改支付密码时，需要传入用户密码进行验证
  userPassword: string;
  paymentPassword: number;
}
```

响应代码表：

| 代码 | 可能的响应消息                                                       | 含义                             |
| ---- | -------------------------------------------------------------------- | -------------------------------- |
| 200  | `For Super Earth!`                                                   | 请求已被成功执行，可访问响应数据 |
| 403  | `Sorry, but this was meant to be a private game: invalid session_id` | 会话无效                         |

响应**数据**：

```typescript
type ResponseData = null;
```

设置 Cookie：

- 无

### 支付订单（US1.1.6 US1.3.2）

`POST /api/transaction/pay/{transaction_id}`

需要 Cookie：

- session_id

请求：

```typescript
type Request = PaymentConfirmation;

interface PaymentConfirmation {
  userPassword?: string;
  paymentPassword?: number;
}
```

提示：

- 初始时，用户可选择使用支付密码或是用户密码进行认证，在请求中，只需要发送用户选择的认证方式的数据；
- 若用户未设置支付密码，则只能使用用户密码认证（“获取个人资料”API 可获取是否设置了支付密码，但应当将结果保存到全局状态中，而不是每次支付都调用“获取个人资料”API 获取）；
- 若同时发送`userPassword`和`paymentPassword`，将选择`userPassword`进行认证；
- 若未发送任何密码，返回`400`错误；
- 若收到`11003`错误，禁用使用支付密码认证的功能。

响应代码表：

| 代码  | 可能的响应消息                                                             | 含义                                          |
| ----- | -------------------------------------------------------------------------- | --------------------------------------------- |
| 200   | `For Super Earth!`                                                         | 请求已被成功执行，可访问响应数据              |
| 400   | `No password provided`                                                     | 请求中`userPassword`和`paymentPassword`都为空 |
| 403   | `Sorry, but this was meant to be a private game: invalid session_id`       | 会话无效                                      |
| 11001 | `Wrong payment password`                                                   | 支付密码错误                                  |
| 11002 | `Wrong user password`                                                      | 用户密码错误                                  |
| 11003 | `Too many failed payment password attempts. Please use your user password` | 支付密码输入错误次数过多                      |

响应**数据**：

```typescript
type ResponseData = null;
```

设置 Cookie：

- 无

### （Debug）生成测试订单（US1.1.3）

`POST /api/transaction/generate`

注意：本 API 仅在 Debug 模式下可用

需要 Cookie：

- session_id

请求：

```typescript
type Request = TransactionGenerateRequest;

interface TransactionGenerateRequest {
  amount: number;
}
```

响应代码表：

| 代码 | 可能的响应消息     | 含义                             |
| ---- | ------------------ | -------------------------------- |
| 200  | `For Super Earth!` | 请求已被成功执行，可访问响应数据 |

响应**数据**：

```typescript
type ResponseData = TransactionInfo;
// TransactionInfo 定义见“交易信息查询”
```

设置 Cookie：

- 无

## 车次查询系统（FE1.2 FE3.1）

### 直达车次查询（US1.2.1）

`POST /api/train/schedule/query_direct`

需要 Cookie：

- session_id

请求：

```typescript
type Request = TrainScheduleQuery;

interface TrainScheduleQuery {
  depatureStation?: string;
  arrivalStation?: string;
  depatureCity?: string;
  arrivalCity?: string;
  // deparuteDate：YYYY-MM-DD
  deparuteDate: string;
}
```

支持按城市查询或者按车站查询。

查询一致性要求：

- `depatureStation`和`depatureCity`有且仅有一个存在
- `arrivalStation`和`arrivalCity`有且仅有一个存在

响应代码表：

| 代码  | 可能的响应消息                                                               | 含义                                             |
| ----- | ---------------------------------------------------------------------------- | ------------------------------------------------ |
| 200   | `For Super Earth!`                                                           | 请求已被成功执行，可访问响应数据                 |
| 403   | `Sorry, but this was meant to be a private game: invalid session_id`         | 会话无效                                         |
| 404   | `Sorry, but this was meant to be a private game: invalid station: {station}` | 查询的`depatureStation`或`depatureStation`不存在 |
| 404   | `Sorry, but this was meant to be a private game: invalid city: {station}`    | 查询的`depatureCity`或`arrivalCity`不存在        |
| 12001 | `Inconsistent query`                                                         | 不满足上述查询一致性要求                         |

响应**数据**：

```typescript
type ResponseData = TrainScheduleInfo[];

// 站点停靠信息
interface StoppingStationInfo {
  stationName: string;
  // 到达该站点的日期时间，若为始发站，不包含该属性
  arrivalTime?: string;
  // 离开该站点的日期时间，若为终到站，不包含该属性
  depatureTime?: string;
}

interface SeatTypeInfo {
  // 该类型座位总计容量
  capacity: number;
  // 该类型座位剩余量
  remainCount: number;
  // 这种座位的价格
  price: number;
}

interface TrainScheduleInfo {
  depatureStation: string;
  // 离开“起始站”的日期时间
  depatureTime: string;
  arrivalStation: string;
  // 到达“到达站”的日期时间
  arrivalTime: string;
  originStation: string;
  // 离开“始发站”的日期时间
  originDepatureTime: string;
  terminalStation: string;
  // 到达“终到站”的日期时间
  terminalArrivalTime: string;
  // 车次号，例如：“G53”
  trainNumber: string;
  // 行程时间：到达“到达站”的时间 - 离开“起始站”的时间，单位：秒
  travelTime: number;
  price: number;
  // 车次经停车站信息
  route: StoppingStationInfo[];
  // 座位类型，如：二等座 -> SeatTypeInfo
  seatInfo: Map<string, SeatTypeInfo>;
}
```

设置 Cookie：

- 无

### 中转车次查询（US3.1.1）

`POST /api/train/schedule/query_indirect`

需要 Cookie：

- session_id

请求：

```typescript
type Request = TrainScheduleQuery;

interface TrainScheduleQuery {
  depatureStation?: string;
  arrivalStation?: string;
  depatureCity?: string;
  arrivalCity?: string;
  // deparuteDate：YYYY-MM-DD
  deparuteDate: string;
}
```

支持按城市查询或者按车站查询。

查询一致性要求：

- `depatureStation`和`depatureCity`有且仅有一个存在
- `arrivalStation`和`arrivalCity`有且仅有一个存在

响应代码表：

| 代码  | 可能的响应消息                                                               | 含义                                             |
| ----- | ---------------------------------------------------------------------------- | ------------------------------------------------ |
| 200   | `For Super Earth!`                                                           | 请求已被成功执行，可访问响应数据                 |
| 403   | `Sorry, but this was meant to be a private game: invalid session_id`         | 会话无效                                         |
| 404   | `Sorry, but this was meant to be a private game: invalid station: {station}` | 查询的`depatureStation`或`depatureStation`不存在 |
| 404   | `Sorry, but this was meant to be a private game: invalid city: {station}`    | 查询的`depatureCity`或`arrivalCity`不存在        |
| 12001 | `Inconsistent query`                                                         | 不满足上述查询一致性要求                         |

响应**数据**：

```typescript
type ResponseData = IndirectTrainScheduleInfo[];

interface IndirectTrainScheduleInfo {
  // 中转乘车第一程的信息
  firstRide: TrainScheduleInfo;
  // 中转乘车第二程的信息
  secondRide: TrainScheduleInfo;
  // 中间换乘可用的时间，单位：秒
  relaxingTime: number;
}
```

设置 Cookie：

- 无

## 购票系统（FE1.3）

### 提交订单（US1.3.2）

`POST /api/train/order/new`

注意：

- 提交订单后，订单为“未支付”状态，本接口将返回`TransactionInfo`，需要根据`TransactionInfo`中的信息调用`支付订单`接口进行支付。支付后订单才会真正被处理
- 对于中转订单，请将`OrderPack`中的`atomic`属性设置为`true`

需要 Cookie：

- session_id

请求：

```typescript
type Request = OrderPack[];

interface OrderPack {
  // 原子操作，若为 true，则`orderList`中任意订单失败将回滚已成功的订单
  atomic: boolean;
  orderList: TrainOrderRequest[];
}

interface TrainOrderRequest {
  // 车次号，例如：“G53”
  trainNumber: string;
  // 离开“始发站”的日期时间
  originDepatureTime: string;

  // 起始站
  depatureStation: string;
  // 到达站
  arrivalStation: string;

  // 乘车人 Id（见`PersonalInfo`）
  personalId: string;
  // 座位类别，如：二等座
  seatType: string;
}
```

响应代码表：

| 代码 | 可能的响应消息                                                                                         | 含义                                                 |
| ---- | ------------------------------------------------------------------------------------------------------ | ---------------------------------------------------- |
| 200  | `For Super Earth!`                                                                                     | 请求已被成功执行，可访问响应数据                     |
| 403  | `Sorry, but this was meant to be a private game: invalid session_id`                                   | 会话无效                                             |
| 404  | `Sorry, but this was meant to be a private game: invalid train: {train_number} {origin_depature_time}` | 车次号不存在，或车次号与离开“始发站”的时间的组合非法 |
| 404  | `Sorry, but this was meant to be a private game: invalid station: {station_name}`                      | 起始站/到达站不存在                                  |
| 404  | `Sorry, but this was meant to be a private game: invalid personal id: {personalId}`                    | 乘车人 Id 不存在，或未与当前用户绑定                 |

响应**数据**：

```typescript
type ResponseData = TransactionInfo;
// TransactionInfo 定义见“交易信息查询”
```

设置 Cookie：

- 无

## 订单管理（FE1.4）

### 订单列表、订单详情（US1.4.1 US1.4.2）

`GET /api/order/list`

需要 Cookie：

- session_id

响应代码表：

| 代码 | 可能的响应消息                                                       | 含义                             |
| ---- | -------------------------------------------------------------------- | -------------------------------- |
| 200  | `For Super Earth!`                                                   | 请求已被成功执行，可访问响应数据 |
| 403  | `Sorry, but this was meant to be a private game: invalid session_id` | 会话无效                         |

响应**数据**：

```typescript
type ResponseData = OrderInfo[];

interface OrderInfo {
  // 订单的 UUID
  orderId: string;
  // 订单对应的交易的 UUID
  transactionId: string;
  // 订单状态：详见 RFC3“关于订单状态的约定”
  status: "Unpaid" | "Paid" | "Ongoing" | "Active" | "Completed" | "Failed" | "Canceled";
  // 订单创建日期时间
  orderTime: string;
  // 支付日期时间
  payTime?: string;
  // 订单金额
  amount: number;
  // 订单类型
  orderType: "train" | "hotel" | "dish" | "takeaway";
  // 订单是否能够取消
  canCancel: boolean;
  // 人类可读的不能取消订单的原因（若适用）
  reason?: string;
}

interface SeatLocationInfo {
  // 车厢号，例如：“03 车 12A 二等座”中的“3”
  carriage: number;
  // 座位行数，例如：“03 车 12A 二等座”中的“12”
  row: number;
  // 座位位置代码，例如：“03 车 12A 二等座”中的“A”
  location: string;
  // 座位类型，例如：“03 车 12A 二等座”中的“二等座”
  type: string;
}

interface TrainOrderInfo extends OrderInfo {
  // 车次，例如：“G53”
  trainNumber: string;
  // 离开起始站日期时间
  depatureTime: string;
  // 乘车人姓名
  name: string;
  // 人类可读的座位号
  seat: SeatLocationInfo;
}

interface HotelOrderInfo extends OrderInfo {
  // 酒店名称
  hotelName: string;
  // 酒店 UUID
  hotelId: string;
  // 旅客姓名
  name: string;
  // 人类可读的房间类型，例如：“大床房”
  roomType: string;
  // 住宿开始日期
  beginDate: string;
  // 住宿结束日期
  endDate: string;
}

interface DishOrderInfo extends OrderInfo {
  // 车次，例如：“G53”
  trainNumber: string;
  // 离开起始站日期时间
  depatureTime: string;
  // 用餐“时间”
  dishTime: "lunch" | "dinner";
  // 用餐人姓名
  name: string;
  // 人类可读的餐品名称
  dishName: string;
}

interface TakeawayOrderInfo extends OrderInfo {
  // 车次，例如：“G53”
  trainNumber: string;
  // 离开起始站日期时间
  depatureTime: string;
  // 车站
  station: string;
  // 店铺名称
  shopName: string;
  // 用餐人姓名
  name: string;
  // 人类可读的外卖餐品名称
  takeawayName: string;
}
```

设置 Cookie：

- 无

### 取消订单（US1.4.3）

`POST /api/order/cancel`

需要 Cookie：

- session_id

请求：

```typescript
type Request = CancelOrderInfo;

interface CancelOrderInfo {
  // 订单的 UUID
  orderId: string;
}
```

响应代码表：

| 代码  | 可能的响应消息                                                       | 含义                               |
| ----- | -------------------------------------------------------------------- | ---------------------------------- |
| 200   | `For Super Earth!`                                                   | 请求已被成功执行，可访问响应数据   |
| 403   | `Sorry, but this was meant to be a private game: invalid session_id` | 会话无效                           |
| 404   | `Sorry, but this was meant to be a private game: invalid order id`   | 订单号不存在，或没有权限访问该订单 |
| 14001 | `Order already cancelled`                                            | 订单已被取消                       |
| 14002 | `Order doesn't fulfill cancellation condition: {reason}`             | 订单不满足取消条件                 |

响应**数据**：

```typescript
type ResponseData = null;
```

设置 Cookie：

- 无

## 酒店服务（FE2.1）

### 酒店查询（US2.1.1）

`POST /api/hotel/query`

需要 Cookie：

- session_id

请求：

```typescript
type Request = HotelQuery;

interface HotelQuery {
  // 目标城市/火车站，由`target_type`属性指定
  target: string;
  target_type: "city" | "station";
  // 通过酒店名称进行匹配，可不存在
  search?: string;
  // 入住日期
  beginDate?: string;
  // 离开日期
  endDate?: string;
}
```

响应代码表：

| 代码  | 可能的响应消息                                                             | 含义                                                                 |
| ----- | -------------------------------------------------------------------------- | -------------------------------------------------------------------- |
| 200   | `For Super Earth!`                                                         | 请求已被成功执行，可访问响应数据                                     |
| 403   | `Sorry, but this was meant to be a private game: invalid session_id`       | 会话无效                                                             |
| 404   | `Sorry, but this was meant to be a private game: invalid target: {target}` | 查询的目标城市/火车站不存在                                          |
| 21001 | `Invalid begin/end date`                                                   | 入住/离开日期不合法：离开比入住早；只设置其中一个；入住时间超过 7 天 |

响应**数据**：

```typescript
type ResponseData = HotelGeneralInfo[];

// 酒店的总体信息
interface HotelGeneralInfo {
  // 酒店 UUID
  hotelId: string;
  // 酒店名称
  name: string;
  // 酒店图片，URL
  picture?: string;
  // 酒店评分
  rating: number;
  // 评分人数
  ratingCount: number;
  // 累计预订人次
  totalBookings: number;
  price: number;
}
```

设置 Cookie：

- 无

### 酒店详情查询（US2.1.2）

`GET /api/hotel/info/{hotel_id}`

需要 Cookie：

- session_id

响应代码表：

| 代码 | 可能的响应消息                                                       | 含义                             |
| ---- | -------------------------------------------------------------------- | -------------------------------- |
| 200  | `For Super Earth!`                                                   | 请求已被成功执行，可访问响应数据 |
| 403  | `Sorry, but this was meant to be a private game: invalid session_id` | 会话无效                         |
| 404  | `Sorry, but this was meant to be a private game: invalid hotel id`   | 查询的酒店不存在                 |

响应**数据**：

```typescript
type ResponseData = HotelDetailInfo;

// 用户评价
interface HotelComment {
  // 用户名
  username: string;
  // 留言日期时间
  commentTime: string;
  // 评分
  rating: number;
  // 留言内容
  comment: string;
}

// 酒店的总体信息
interface HotelDetailInfo {
  // 酒店 UUID
  hotelId: string;
  // 酒店名称
  name: string;
  // 酒店详细地址
  address: string;
  // 联系电话
  phone: string[];

  // 酒店图片列表，URL
  picture?: string[];
  // 酒店评分
  rating: number;
  // 评分人数
  ratingCount: number;
  // 累计预订人次
  totalBookings: number;
  // 用户留言
  comments: HotelComment[];
}
```

设置 Cookie：

- 无

### 酒店预订信息查询（US2.1.2）

`POST /api/hotel/order_info`

需要 Cookie：

- session_id

请求：

```typescript
type Request = HotelOrderQuery;

interface HotelOrderQuery {
  // 欲查询酒店的 UUID
  hotelId: string;
  // 入住日期
  beginDate?: string;
  // 离开日期
  endDate?: string;
}
```

响应代码表：

| 代码  | 可能的响应消息                                                       | 含义                                                                 |
| ----- | -------------------------------------------------------------------- | -------------------------------------------------------------------- |
| 200   | `For Super Earth!`                                                   | 请求已被成功执行，可访问响应数据                                     |
| 403   | `Sorry, but this was meant to be a private game: invalid session_id` | 会话无效                                                             |
| 404   | `Sorry, but this was meant to be a private game: invalid hotel id`   | 查询的酒店不存在                                                     |
| 21001 | `Invalid begin/end date`                                             | 入住/离开日期不合法：离开比入住早；只设置其中一个；入住时间超过 7 天 |

响应**数据**：

```typescript
// 人类可读的房间类型（例如，大床房） -> HotelOrderInfo
type ResponseData = Map<string, HotelRoomDetailInfo>;

// 房间预订信息
interface HotelRoomDetailInfo {
  // 该类型房间总可入住人数
  capacity: number;
  // 该类型房间剩余容量
  remainCount: number;
  // 价格
  price: number;
}
```

设置 Cookie：

- 无

### 酒店预订（US2.1.3）

`POST /api/hotel/order`

注意：提交订单后，订单为“未支付”状态，本接口将返回`TransactionInfo`，需要根据`TransactionInfo`中的信息调用`支付订单`接口进行支付。支付后订单才会真正被处理。

需要 Cookie：

- session_id

请求：

```typescript
type Request = HotelOrderRequest[];

interface HotelOrderRequest {
  // 欲预订酒店的 UUID
  hotelId: string;

  // 人类可读的房间类型
  roomType: string;

  // 入住日期
  beginDate?: string;
  // 离开日期
  endDate?: string;

  // 旅客 UUID（见`PersonalInfo`）
  personalId: string;
}
```

响应代码表：

| 代码  | 可能的响应消息                                                                      | 含义                                                                 |
| ----- | ----------------------------------------------------------------------------------- | -------------------------------------------------------------------- |
| 200   | `For Super Earth!`                                                                  | 请求已被成功执行，可访问响应数据                                     |
| 403   | `Sorry, but this was meant to be a private game: invalid session_id`                | 会话无效                                                             |
| 404   | `Sorry, but this was meant to be a private game: invalid hotel id`                  | 预订的酒店不存在                                                     |
| 404   | `Sorry, but this was meant to be a private game: invalid personal id: {personalId}` | 旅客 Id 不存在，或未与当前用户绑定                                   |
| 21001 | `Invalid begin/end date`                                                            | 入住/离开日期不合法：离开比入住早；只设置其中一个；入住时间超过 7 天 |

响应**数据**：

```typescript
type ResponseData = TransactionInfo;
// TransactionInfo 定义见“交易信息查询”
```

设置 Cookie：

- 无

### 获取评价配额（US2.1.5）

`GET /api/hotel/quota/{hotel_id}`

需要 Cookie：

- session_id

响应代码表：

| 代码 | 可能的响应消息                                                       | 含义                             |
| ---- | -------------------------------------------------------------------- | -------------------------------- |
| 200  | `For Super Earth!`                                                   | 请求已被成功执行，可访问响应数据 |
| 403  | `Sorry, but this was meant to be a private game: invalid session_id` | 会话无效                         |
| 404  | `Sorry, but this was meant to be a private game: invalid hotel id`   | 查询的酒店不存在                 |

响应**数据**：

```typescript
type ResponseData = HotelCommentQuota;

interface HotelCommentQuota {
  // 用户可为该酒店撰写评价数量配额（该酒店的已完成订单数量）
  quota: number;
  // 已使用配额
  used: number;
}
```

设置 Cookie：

- 无

### 撰写评价（US2.1.5）

`POST /api/hotel/comment`

需要 Cookie：

- session_id

请求：

```typescript
type Request = NewHotelComment;

interface NewHotelComment {
  // 欲评价酒店的 UUID
  hotelId: string;
  // 评分
  rating: number;
  // 留言内容
  comment: string;
}
```

响应代码表：

| 代码  | 可能的响应消息                                                       | 含义                                               |
| ----- | -------------------------------------------------------------------- | -------------------------------------------------- |
| 200   | `For Super Earth!`                                                   | 请求已被成功执行，可访问响应数据                   |
| 403   | `Sorry, but this was meant to be a private game: invalid session_id` | 会话无效                                           |
| 404   | `Sorry, but this was meant to be a private game: invalid hotel id`   | 评价的酒店不存在                                   |
| 21002 | `Invalid rating: {rating}`                                           | 评分不合法，不属于[0.0, 5.0]范围                   |
| 21003 | `Comment length exceed`                                              | 评价长度过长                                       |
| 21004 | `Comment count exceed`                                               | 用户撰写的评价过多（超过了该酒店的已完成订单数量） |

响应**数据**：

```typescript
type ResponseData = null;
```

设置 Cookie：

- 无

## 火车餐服务（FE2.2）

### 火车餐查询（按车次）（US2.2.1）

`POST /api/dish/query_by_train_number`

需要 Cookie：

- session_id

请求：

```typescript
type Request = DishQuery;

interface DishQuery {
  // 车次
  trainNumber: string;
  // 离开“始发站”的日期时间
  originDepatureTime: string;
}
```

响应代码表：

| 代码 | 可能的响应消息                                                                  | 含义                             |
| ---- | ------------------------------------------------------------------------------- | -------------------------------- |
| 200  | `For Super Earth!`                                                              | 请求已被成功执行，可访问响应数据 |
| 403  | `Sorry, but this was meant to be a private game: invalid session_id`            | 会话无效                         |
| 404  | `Sorry, but this was meant to be a private game: invalid trainNumber: {target}` | 查询的车次不存在                 |

响应**数据**：

```typescript
type ResponseData = TrainDishInfo;

interface DishInfo {
  // 该火车餐在哪些时段提供？
  availableTime: Array<"launch" | "dinner">;
  // 火车餐名称
  name: string;
  // 火车餐类别，例如：主食、饮料、零食
  type: string;
  // 火车餐图片，URL
  picture: string;
  // 价格
  price: number;
}

interface TakeawayDishInfo {
  // 餐品名称
  name: string;
  // 餐品图片，URL
  picture: string;
  // 价格
  price: number;
}

interface Takeaway {
  // 店铺名称
  shopName: string;

  dishes: TakeawayDishInfo[];
}

interface TrainDishInfo {
  // 车次
  trainNumber: string;
  // 离开“始发站”的日期时间
  originDepatureTime: string;
  // 到达“终到站”的日期时间
  terminalArrivalTime: string;

  dishes: DishInfo[];

  // 车站名称 -> 可点的外卖列表
  takeaway: Map<string, Takeaway[]>;

  // 能否预订
  canBooking: boolean;
  // 不能预订原因
  reason?: string;
}
```

设置 Cookie：

- 无

### 火车餐查询（按起始到达站）（US2.2.1）

`POST /api/dish/query_by_station`

需要 Cookie：

- session_id

请求：

```typescript
type Request = DishStationQuery;

interface DishStationQuery {
  // 起始站
  depatureStation: string;
  // 到达站
  arrivalStation: string;
  // 查询日期
  targetDate: string;
}
```

响应代码表：

| 代码 | 可能的响应消息                                                               | 含义                             |
| ---- | ---------------------------------------------------------------------------- | -------------------------------- |
| 200  | `For Super Earth!`                                                           | 请求已被成功执行，可访问响应数据 |
| 403  | `Sorry, but this was meant to be a private game: invalid session_id`         | 会话无效                         |
| 404  | `Sorry, but this was meant to be a private game: invalid station: {station}` | 查询的起始/到达站不存在          |

响应**数据**：

```typescript
type ResponseData = FullTrainDishInfo[];

interface FullTrainDishInfo {
  // 车次
  trainNumber: string;

  depatureStation: string;
  // 离开“起始站”的日期时间
  depatureTime: string;
  arrivalStation: string;
  // 到达“到达站”的日期时间
  arrivalTime: string;
  originStation: string;
  // 离开“始发站”的日期时间
  originDepatureTime: string;
  terminalStation: string;
  // 到达“终到站”的日期时间
  terminalArrivalTime: string;

  dishes: DishInfo[];
  // 车站名称 -> 可点的外卖列表
  takeaway: Map<string, Takeaway[]>;

  // 能否预订
  canBooking: boolean;
  // 不能预订原因
  reason?: string;
}
```

设置 Cookie：

- 无

### 火车餐预订（US2.2.2）

`POST /api/dish/order`

注意：提交订单后，订单为“未支付”状态，本接口将返回`TransactionInfo`，需要根据`TransactionInfo`中的信息调用`支付订单`接口进行支付。支付后订单才会真正被处理。

需要 Cookie：

- session_id

请求：

```typescript
type Request = TrainDishOrderRequest;

interface DishOrder {
  // 火车餐名称
  name: string;
  // 用餐人 UUID
  personalId: string;
  // 份数
  amount: number;
  // 用餐时间
  dishTime: "launch" | "dinner";
}

interface TakeawayOrder {
  // 车站名称
  station: string;
  // 店铺名称
  shopName: string;
  // 餐品名称
  name: string;
  // 用餐人 UUID
  personalId: string;
  // 份数
  amount: number;
}

interface TrainDishOrderRequest {
  // 车次
  trainNumber: string;
  // 离开“始发站”的日期时间
  originDepatureTime: string;

  // 要预订的火车餐列表
  dishes: DishOrder[];
  // 要预订的外卖列表
  takeaway: TakeawayOrder[];
}
```

响应代码表：

| 代码  | 可能的响应消息                                                                      | 含义                                 |
| ----- | ----------------------------------------------------------------------------------- | ------------------------------------ |
| 200   | `For Super Earth!`                                                                  | 请求已被成功执行，可访问响应数据     |
| 403   | `Sorry, but this was meant to be a private game: invalid session_id`                | 会话无效                             |
| 404   | `Sorry, but this was meant to be a private game: invalid trainNumber: {target}`     | 目标车次不存在                       |
| 404   | `Sorry, but this was meant to be a private game: invalid personal id: {personalId}` | 乘车人 Id 不存在，或未与当前用户绑定 |
| 22001 | `Invalid dish name: {name}`                                                         | 火车餐不存在                         |
| 22002 | `Invalid dish amount: {amount}`                                                     | 非法份数                             |
| 22003 | `Invalid takeaway station: {station}`                                               | 非法车站名称                         |
| 22004 | `Invalid takeaway shop name: {shop_name}`                                           | 非法店铺名称                         |
| 22005 | `Invalid takeaway name: {name}`                                                     | 非法外卖名称                         |

响应**数据**：

```typescript
type ResponseData = TransactionInfo;
// TransactionInfo 定义见“交易信息查询”
```

设置 Cookie：

- 无

## 通知系统（FE2.3）

为了避免消息轮询，通知系统的连接使用 WebSocket，[参考](https://developer.mozilla.org/en-US/docs/Web/API/WebSockets_API)。

若无特别说明，WebSocket 消息为 JSON，且**不依照**`APIResponse<T>`格式，而依照如下`Message<T>`格式。

```typescript
interface Message<T> {
  // 标识该消息的类型，即`T`，由于 WebSocket 在同一个端点中传递多种类型的消息，需要通过`type`确定收到的消息类型。
  type: string;
  data: T;
}
```

下文中的“消息**数据**”，指的是`Message`的`data`属性。

### （非 WebSocket）获取 WebSocket 端点

`GET /api/notify/endpoint`

获取 WebSocket 连接端点，用于建立连接。

需要 Cookie：

- 无

响应代码表：

| 代码 | 可能的响应消息     | 含义                             |
| ---- | ------------------ | -------------------------------- |
| 200  | `For Super Earth!` | 请求已被成功执行，可访问响应数据 |

响应**数据**：

```typescript
type ResponseData = string;
// string 中为 WebSocket 端点，格式：`ws://xxx.xxx/xxx/xxx`
```

设置 Cookie：

- 无

### 订购通知（US2.3.1）

方向：`Server -> Client`

消息数据：

```typescript
type MessageType = OrderNotify;

interface Notify {
  // 消息标题
  title: string;
  // 发送的日期时间
  messageTime: string;
  // 提醒类型
  type: "order" | "trip";
}

// OrderInfo 定义详见“订单列表、订单详情”
interface OrderNotify extends Notify {
  order: OrderInfo;
}
```

### 行程通知（US2.3.1）

方向：`Server -> Client`

消息数据：

```typescript
type MessageType = TripNotify;

interface TripNotify extends Notify {
  // 车次，例如：“G53”
  trainNumber: string;
  // 离开起始站日期时间
  depatureTime: string;
  // 起始站
  depatureStation: string;
  // 到达站
  arrivalStation: string;
}
```

### （非 WebSocket）获取历史通知（US2.3.2）

`GET /api/notify/history`

需要 Cookie：

- session_id

响应代码表：

| 代码 | 可能的响应消息                                                       | 含义                             |
| ---- | -------------------------------------------------------------------- | -------------------------------- |
| 200  | `For Super Earth!`                                                   | 请求已被成功执行，可访问响应数据 |
| 403  | `Sorry, but this was meant to be a private game: invalid session_id` | 会话无效                         |

响应**数据**：

```typescript
type ResponseData = Notify[];
```

设置 Cookie：

- 无

## 智能行程推荐系统（FE3.2）

### 附近酒店推荐（US3.2.1）

`GET /api/hotel/recommend`

需要 Cookie：

- session_id

响应代码表：

| 代码 | 可能的响应消息                                                       | 含义                             |
| ---- | -------------------------------------------------------------------- | -------------------------------- |
| 200  | `For Super Earth!`                                                   | 请求已被成功执行，可访问响应数据 |
| 403  | `Sorry, but this was meant to be a private game: invalid session_id` | 会话无效                         |

响应**数据**：

```typescript
// 人类可读的目的地名称 -> HotelGeneralInfo[]
type ResponseData = Map<string, HotelGeneralInfo[]>;

// HotelGeneralInfo 定义详见“酒店查询”
```

设置 Cookie：

- 无

### 火车餐推荐（US3.2.2）

`GET /api/dish/recommend`

需要 Cookie：

- session_id

响应代码表：

| 代码 | 可能的响应消息                                                       | 含义                             |
| ---- | -------------------------------------------------------------------- | -------------------------------- |
| 200  | `For Super Earth!`                                                   | 请求已被成功执行，可访问响应数据 |
| 403  | `Sorry, but this was meant to be a private game: invalid session_id` | 会话无效                         |

响应**数据**：

```typescript
// 车次 -> DishInfo[]
type ResponseData = DishTakeawayInfo;

interface DishTakeawayInfo {
  dishes: DishInfo[];
  // 车站 -> Takeaway[]
  takeaway: Map<string, Takeaway[]>;
}

// DishInfo、Takeaway 定义详见“火车餐查询”
```

设置 Cookie：

- 无
