# Request For Comments 1: 项目约定与要求

Version: 4 20250330 235900

## 项目进度管理

| 周号  | 任务                         |
| ----- | ---------------------------- |
| 6     | 明确大作业题目，规划每周任务 |
| 7--12 | 正式开发                     |
| 13    | Debug，文档，PPT 等          |

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

### 代码格式化

为防止混乱，应当配置 IDE 按文件类型使用如下格式化工具：

- `html,css,js,ts,vue,json,yaml,markdown`: Prettier
- `rust`: Rustfmt

## 分支管理

- `main`：包含经整体测试没有 Bug，可正常运行的代码；
- `dev`：包含通过基本单元测试的代码；
- topic 分支：包含正在实现中的功能。

每个**任务**应当在单独的 topic 分支上完成，当完成相应任务后（请确保符合**“要求”**，详见下），提交到`dev`分支的 Pull Request，经过 code review 无误后，方可合并到`dev`分支。

`main`和`dev`分支应当被保护，确保只接收经过检查的代码。

PR 的合并按如下顺序进行：按 PR 发起时间依次检查每个 PR，若 PR 符合合并条件，则合并；否则，通知**任务负责人**进行相应修改，并继续检查下一个 PR，直到**任务负责人**完成修改并通知 reviewer，可再次检查修改后的 PR 是否符合条件。

若应前序 PR 的合并导致后续 PR 冲突，后续 PR 的**任务负责人**应当及时解决冲突。

## 项目要求

在后文中，粗体的“**要求**”和“**建议**”的语义如下：

- **要求**：其中的约定能够保证代码的基本质量，若不遵守，则对应的 PR 可能**不会被合并**；
- **建议**：其中的约定可能可以提升开发、调试效率，但在初期可能需要投入额外精力。

### 通用

**要求**：

应当：

- 编写符合规定的 commit message；
- 使用英语编写 commit message；
- 使用符合规定的版本号；
- 在`CODEOWNERS`文件中正确标注自己负责的部分（[详见](https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/customizing-your-repository/about-code-owners#codeowners-file-location)）。

不应：

- 提交仅用作本地临时测试的代码；
- 提交与项目代码格式化配置不符的代码；
- 直接提交到`main`和`dev`分支（已通过规则阻止）；
- 进行 force push（已通过规则阻止）。

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
