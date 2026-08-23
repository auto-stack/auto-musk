# 高亮三方案一致性矩阵（PLAN-038 T15）

- 引擎：prismjs ^1.29（web/vue 轨现状,PrismCodeBlock 同语言注册面）·
  lowlight@3 common 集（@autodown/vue 内置同装配）·
  syntect 5 + two-face 0.4（auto-lang code_editor 同内核同版本,scripts/highlight-rs）
- 方法：同一 fixtures 代码,三引擎各产出逐字符 token 流,经 scope→token 近似映射表
  （CATEGORY_MAP,见脚本头）归一类别集后两两逐字符一致率。**近似映射,决策粒度数据,
  非全等断言**。
- fixtures：scripts/lib-parity/fixtures/highlight/code/（musk 实际语言集 14 语言,
  覆盖计划点名 11 语言 + PrismCodeBlock 实际注册的 cpp/go）
- 再生：node scripts/lib-parity/highlight-compare.mjs（内含 cargo 侧调用）

## 汇总（各语言一致率均值）

| 对比 | 一致率 |
|---|---|
| prism vs lowlight | 71.3% |
| prism vs syntect | 60.2% |
| lowlight vs syntect | 58.6% |

## 分语言矩阵

| 语言 | prism | lowlight | syntect | p–l | p–s | l–s |
|---|---|---|---|---|---|---|
| bash | ✓ | ✓ | ✓ | 62.8% | 52.8% | 54.3% |
| c | ✓ | ✓ | ✓ | 65% | 62.8% | 56% |
| cpp | ✓ | ✓ | ✓ | 74.9% | 75.5% | 64% |
| go | ✓ | ✓ | ✓ | 70.6% | 48.1% | 37.1% |
| java | ✓ | ✓ | ✓ | 76.2% | 77.1% | 65.3% |
| javascript | ✓ | ✓ | ✓ | 60.3% | 33.3% | 46.6% |
| json | ✓ | ✓ | ✓ | 94% | 54.6% | 58.7% |
| markdown | ✓ | ✓ | ✓ | 57.2% | 75.5% | 58.2% |
| python | ✓ | ✓ | ✓ | 84.2% | 78% | 70.7% |
| rust | ✓ | ✓ | ✓ | 77.8% | 71.9% | 63.5% |
| sql | ✓ | ✓ | ✓ | 81.6% | 76.6% | 89.7% |
| toml | ✓ | ✓ | ✓ | 78.1% | 44.4% | 44.8% |
| typescript | ✓ | ✓ | ✓ | 70.7% | 40.6% | 44.8% |
| yaml | ✓ | ✓ | ✓ | 45% | 51.1% | 66.5% |

长度校验（逐字符流长度应 = 代码长度,三引擎一致）:
- bash: {"prism":341,"lowlight":341,"syntect":341}
- c: {"prism":511,"lowlight":511,"syntect":511}
- cpp: {"prism":650,"lowlight":650,"syntect":650}
- go: {"prism":561,"lowlight":561,"syntect":561}
- java: {"prism":672,"lowlight":672,"syntect":672}
- javascript: {"prism":541,"lowlight":541,"syntect":541}
- json: {"prism":218,"lowlight":218,"syntect":218}
- markdown: {"prism":208,"lowlight":208,"syntect":208}
- python: {"prism":692,"lowlight":692,"syntect":692}
- rust: {"prism":537,"lowlight":537,"syntect":537}
- sql: {"prism":380,"lowlight":380,"syntect":380}
- toml: {"prism":270,"lowlight":270,"syntect":270}
- typescript: {"prism":556,"lowlight":556,"syntect":556}
- yaml: {"prism":278,"lowlight":278,"syntect":278}

（决策解读见 PLAN-038 T16 复审记录登记。）
