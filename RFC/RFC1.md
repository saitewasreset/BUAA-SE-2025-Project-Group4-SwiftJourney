# Request For Comments 1: 项目约定与要求

**本文档包含代码编写、代码提交与合并、任务安排等的重要内容，请认真阅读，以免影响开发进度以及团队、个人成绩**

Version: 5 20250406 195300

最近变更：

- Version 5
  - 调整开发时间安排
  - 明确 commit message、代码注释的语言
  - 新增分支命名要求
  - 新增任务进度管理

## 项目进度管理

| 周号  | 任务                         |
| ----- | ---------------------------- |
| 6     | 明确大作业题目，规划每周任务 |
| 7--13 | 正式开发                     |
| 14    | Debug，文档，PPT 等          |

第**6**周：解析项目需求，明确分工以及每周**每人**的**任务**，初步确定后端与前端交互的 API。

## 一般约定

[版本号约定](https://semver.org/lang/zh-CN/)

[commit message 约定](https://www.conventionalcommits.org/en/v1.0.0/#specification)

格式：

```text
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]

```

例如：

不含 scope：

```text
docs: correct spelling of CHANGELOG
```

包含 scope，breaking change：

```text
feat(api)!: send an email to the customer when a product is shipped
```

scope 约定：

Conventional Commits 中的原始规范：

> A scope MAY be provided after a type. A scope MUST consist of a noun describing a section of the codebase surrounded by parenthesis

1. 若 description 中已经明确指出 scope，则不应附加 scope
   例如：`docs(README):  update README.md`应修改为`docs: update README.md`；
2. scope 不一定是文件名；若要使用文件名，**不应**带扩展名。

作为参考，以下是部分[来自 Angular 的规范](https://github.com/angular/angular/blob/22b96b9/CONTRIBUTING.md#-commit-message-guidelines)：

### Type

Must be one of the following:

- **build**: Changes that affect the build system or external dependencies (example scopes: gulp, broccoli, npm)
- **ci**: Changes to our CI configuration files and scripts (example scopes: Travis, Circle, BrowserStack, SauceLabs)
- **docs**: Documentation only changes
- **feat**: A new feature
- **fix**: A bug fix
- **perf**: A code change that improves performance
- **refactor**: A code change that neither fixes a bug nor adds a feature
- **style**: Changes that do not affect the meaning of the code (white-space, formatting, missing semi-colons, etc)
- **test**: Adding missing tests or correcting existing tests

### Scope

The scope should be the name of the npm package affected (as perceived by the person reading the changelog generated from commit messages.

The following is the list of supported scopes:

- **animations**
- **common**
- **compiler**
- **compiler-cli**
- **core**
- **elements**
- **forms**
- **http**
- **language-service**
- **platform-browser**
- **platform-browser-dynamic**
- **platform-server**
- **platform-webworker**
- **platform-webworker-dynamic**
- **router**
- **service-worker**
- **upgrade**

### 语言

commit message 可使用简体中文或英文，但需要遵循 commit message 约定，例如：

- docs(RFC4): 在"火车餐推荐"中新增关于外卖的内容
- docs(RFC4): add detailed explanation for `isFirstLogin` in `UserLoginInfo`

代码注释可使用简体中文或英文。

### 代码格式化

为防止混乱，应当配置 IDE 按文件类型使用如下格式化工具，并使用仓库根目录中的相关配置文件：

- `html,css,js,ts,vue,json,yaml,markdown`: Prettier（配置文件：`.prettierrc.json`）
- `rust`: Rustfmt（配置文件：`rustfmt.toml`）

## 分支管理

- `main`：包含经整体测试没有 Bug，可正常运行的代码；
- `dev`：包含通过基本单元测试的代码；
- topic 分支：包含正在实现中的功能。

每个**Task**应当在单独的 topic 分支上完成，当完成相应任务后（请确保符合**“要求”**，详见下），提交到`dev`分支的 Pull Request，经过 code review 无误后，方可合并到`dev`分支。

Topic 分支的命名要求按照指定格式，参见`/RFC/attachment/RFC1/topic-branch-name.txt`

`main`和`dev`分支应当被保护，确保只接收经过检查的代码。

PR 的合并按如下顺序进行：按 PR 发起时间依次检查每个 PR，若 PR 符合合并条件，则合并；否则，通知**任务负责人**进行相应修改，并继续检查下一个 PR，直到**任务负责人**完成修改并通知 reviewer，可再次检查修改后的 PR 是否符合条件。

若应前序 PR 的合并导致后续 PR 冲突，后续 PR 的**任务负责人**应当及时解决冲突。

## 任务进度管理

建议华为云的邮件通知功能（[设置地址](https://devcloud.cn-north-4.huaweicloud.com/msgset)），若因未设置邮件通知而错过项目活动，后果自负。

原则上，每个 Task 只分配给一人完成。

在 CodeArts 上的工作时间安排中，User Stroy、Task 层级任务安排的完成时间仅供参考，可根据实际情况调整。

但，请尽量保证在 Feature、Epic 对应的截止日期前完成分配的所有 User Stroy、Task。

若确有特殊情况，可适当延迟。但，请在进度检查日期到达前完成对应截止日期的所有工作，**以免影响团队成绩和个人成绩**。

进度检查日期时间列表如下：

| 次数 | 日期         | 具体截止时间             |
| ---- | ------------ | ------------------------ |
| 1    | 第 8 周周日  | 2025-04-20 23:59:59UTC+8 |
| 2    | 第 11 周周一 | 2025-05-05 23:59:59UTC+8 |
| 3    | 第 12 周周日 | 2025-05-18 23:59:59UTC+8 |
| 4    | 第 15 周周一 | 2025-06-02 23:59:59UTC+8 |

若在完成任务过程中存在任何困难，及时在**QQ 群聊**、**微信群聊**中询问，不应因技术困难等原因影响工作完成。

任务完成的标志是相应的合并请求被合并到`dev`分支，**不是发起合并请求**。因此，请考虑代码审核时间，不要卡点发起合并请求。

合并请求发起者可以指定合并请求审核与合入的**最长空闲时间**，即，若在该时间内没有人提出有效修改意见，即使检视、审核人数未达到门禁要求（2 人），也可由**项目经理、项目管理员**强制合入。详细要求如下：

- 指定的**最长空闲时间**不应小于 24 小时
- 指定的**最长空闲时间**不应大于 72 小时
- 指定的**最长空闲时间**对应的截止时刻不应在该 Task 的截止时刻之后
- 若在发起合并请求后向原分支提交了新的 commit，则应当从最后一次提交开始计算**最长空闲时间**
- 若有有效修改意见，则应当从有效修改意见解决后开始计算**最长空闲时间**
  - 完成对有效修改意见的修改后（提交修改 commit 后），及时通过**QQ 群聊**、**微信群聊**、**QQ 私聊**、**微信私聊**、**评论回复**的方式联系意见发起者
  - 意见发起者应当在 24 小时内标记意见为解决，或提出新的有效修改意见
- 合并请求发起者在发起合并请求并选择**最长空闲时间**后，及时**QQ 群聊**、**微信群聊**中发送相关信息，示例如下：
  - 对于合并请求!5、!6、!7（RFC4 API 文档 Version 2）、!8（RFC3 工作项 Version 6）：若同意变更，在 CodeArts 上检视、审核；若有需要补充、修改的地方，在 CodeArts 上提出评审意见（或直接在本群提出）；若在 2025-04-07 23:59:59UTC+8 前未检视、审核或提出修改，默认同意上述变更

原则上，课程结束提交项目时，**每个参与者的权重相同**，**不依照**总计提交代码行数、提交次数等调整。

但，若**因个人原因严重延误项目进度**，导致 RFC3 中的要求未能在**第 15 周周日（2025-06-08 23:59:59UTC+8）**（如果课程组指定的项目答辩时间早于该时间点，则以答辩开始时间为准），**保留调整权重的可能性**。

注意，课程结束提交项目时将包含完整 Git 仓库（`.git`文件夹），并设置 CodeArts 上的仓库为公开，请注意 commit message 的内容。

## 项目要求

在后文中，粗体的“**要求**”和“**建议**”的语义如下：

- **要求**：其中的约定能够保证代码的基本质量，若不遵守，则对应的 PR 可能**不会被合并**，为了避免反复审核浪费时间，请在发起合并请求前检查是否符合其中的要求；
- **建议**：其中的约定可能可以提升开发、调试效率，但在初期可能需要投入额外精力。

### 通用

**要求**：

应当：

- 编写符合规定的 commit message；

不应：

- 提交仅用作本地临时测试的代码；
- 提交与项目代码格式化配置不符的代码；
- 直接提交到`main`和`dev`分支（已通过规则阻止）；
- 进行 force push（已通过规则阻止）**在执行 git push 前，请三思**。

### 前端

**要求**：

应当：

- 编写完整的数据获取（调用后端 API）、处理、展示的逻辑；

不应：

- 使用硬编码的数据达到展示效果；

**建议**：

应当：

- 使用 TypeScript；
- 编写测试（[参考](https://vuejs.org/guide/scaling-up/testing.html)）

### 后端

**要求**：

应当：

- 编写单元测试（[参考](https://doc.rust-lang.org/book/ch11-00-testing.html)）；
- 在项目配置文件下，`cargo clippy`检查无警告；
- 为每个函数编写注释，说明函数功能（例如，例子、Precondition、Postcondition、Side effect、Panics、Errors）（[参考](https://doc.rust-lang.org/book/ch14-02-publishing-to-crates-io.html)）

不应：

- 在异步上下文中阻塞或进行大量 CPU 密集型操作；
- 在持有互斥锁时调用`.await`；
- 在没有明确理由的情况下使用`#[allow(xxx)]`忽略警告；
- 在没有明确理由的情况下使用`unsafe`；
