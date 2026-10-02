请用 /auto-plan:new 技能新建一个计划。

需求：建立一个本地密码生成器 web app（纯前端，无后端、无构建步骤）。
- 功能：密码长度可选（8–64）；字符集开关（大写 / 小写 / 数字 / 符号）；一键生成；一键复制到剪贴板；简单强度提示（弱/中/强）。
- 随机源必须用密码学安全 API（Web Crypto `crypto.getRandomValues`），要求无偏差取样。
- 技术形态：单个 `index.html`（内联 CSS/UI 骨架）+ 独立 ES module `pwgen.js`（纯逻辑：字符集组装、安全随机取样、强度评级）+ `pwgen.test.js`（Node 内置 `node:test`，`node --test` 直接可跑，不引第三方依赖）。
- 同时新建本仓库第一份规范 `docs/specs/password-generator.md`（生成算法契约：字符集、无偏差取样算法、强度评级规则）。

仓库背景说明：本工作区（demo）是一个独立的 git 仓库（main 分支），不是 auto-musk 主仓；后续各阶段请按技能规则在本仓库范围内操作。

如果需要澄清，直接在回复里列出问题与采用的默认假设即可；无需等待问卷，按合理默认假设推进，把计划写到 docs/plans/ 下。
