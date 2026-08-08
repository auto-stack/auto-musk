#!/usr/bin/env bash
# musk-ui-smoke.sh — auto-musk 前端冒烟测试套件（playwright-cli）
#
# 用法：
#   ./musk-ui-smoke.sh [url] [user] [pass] [label]
#   例: ./musk-ui-smoke.sh http://127.0.0.1:4444 admin admin "原生版"
#       ./musk-ui-smoke.sh http://127.0.0.1:5173 admin admin "Auto版"
#
# 前置：musk serve (:8080) + aaid (:17654) 已启动。
# 测试覆盖：登录 → Specs 加载 → Wiki 新建 → Chat 会话列表。

URL="${1:-http://127.0.0.1:5173}"
USER="${2:-admin}"
PASS="${3:-admin}"
LABEL="${4:-Auto版}"
PASS_COUNT=0; FAIL_COUNT=0; TOTAL=0

ok()   { TOTAL=$((TOTAL+1)); echo "  ✅ PASS: $1"; PASS_COUNT=$((PASS_COUNT+1)); }
fail() { TOTAL=$((TOTAL+1)); echo "  ❌ FAIL: $1 — $2"; FAIL_COUNT=$((FAIL_COUNT+1)); }

# 从 snapshot 提取第一个匹配行的 ref
snap_ref() { playwright-cli snapshot 2>/dev/null | grep -iE "$1" | grep -oE 'ref=[a-z0-9]+' | head -1 | cut -d= -f2; }
snap_has() { playwright-cli snapshot 2>/dev/null | grep -iE "$1" | head -1; }

echo "========================================"
echo "  auto-musk UI 冒烟测试 — $LABEL"
echo "  URL: $URL"
echo "========================================"

# ── 1. 登录 ──
echo ""; echo "── 1. 登录 ──"
playwright-cli open "$URL/" > /dev/null 2>&1; sleep 2
playwright-cli eval "() => { localStorage.clear(); return 1 }" > /dev/null 2>&1
playwright-cli goto "$URL/" > /dev/null 2>&1; sleep 2

U=$(snap_ref 'textbox.*(username|用户名)')
P=$(snap_ref 'textbox.*(password|密码)')
L=$(snap_ref 'button.*(Login|登录)')
if [ -z "$U" ] || [ -z "$P" ] || [ -z "$L" ]; then
  fail "登录页" "找不到输入框/按钮"
  echo ""; echo "结果: $PASS_COUNT/$TOTAL — 登录页未加载，终止"; exit 1
fi
playwright-cli fill "$U" "$USER" > /dev/null 2>&1
playwright-cli fill "$P" "$PASS" > /dev/null 2>&1
playwright-cli click "$L" > /dev/null 2>&1; sleep 3
[ -n "$(snap_ref 'button.*(Chat|Specs|Wiki|聊天|规范|知识库)')" ] && ok "登录 → 进主页" || fail "登录" "未见主页导航"

# ── 2. Specs ──
echo ""; echo "── 2. Specs 加载 ──"
S=$(snap_ref 'button.*(Specs|规范)')
if [ -n "$S" ]; then
  playwright-cli click "$S" > /dev/null 2>&1; sleep 2
  [ -n "$(snap_has 'Overview|goals|architecture|目标|设计')" ] && ok "Specs 加载" || fail "Specs 加载" "未见 section"
  ERR=$(playwright-cli console error 2>/dev/null | grep -c 'TypeError\|Unexpected token' || true)
  [ "$ERR" = "0" ] && ok "Specs 无 JS error" || fail "Specs console" "$ERR 个 error"
else
  fail "Specs" "找不到导航按钮"
fi

# ── 3. Wiki 新建 ──
echo ""; echo "── 3. Wiki 新建 ──"
W=$(snap_ref 'button.*(Wiki|知识库)')
if [ -n "$W" ]; then
  playwright-cli click "$W" > /dev/null 2>&1; sleep 2
  [ -n "$(snap_has 'New|新建|wiki-nav|Select a page')" ] && ok "Wiki 加载" || fail "Wiki 加载" "未见页面区"
  N=$(snap_ref 'button.*(\+ New|新建|New Page)')
  if [ -n "$N" ]; then
    playwright-cli click "$N" > /dev/null 2>&1; sleep 1
    SL=$(snap_ref 'textbox.*(slug|page-slug)')
    TI=$(snap_ref 'textbox.*(Title|标题)')
    CR=$(snap_ref 'button.*(Create|Save|保存|创建)')
    if [ -n "$SL" ] && [ -n "$TI" ] && [ -n "$CR" ]; then
      TS=$(date +%s)
      playwright-cli fill "$SL" "smoke-$TS" > /dev/null 2>&1
      playwright-cli fill "$TI" "Smoke$TS" > /dev/null 2>&1
      playwright-cli click "$CR" > /dev/null 2>&1; sleep 2
      [ -n "$(snap_has "smoke-$TS|Smoke$TS")" ] && ok "Wiki 新建页面" || fail "Wiki 新建" "侧栏未见新页面"
    else
      fail "Wiki 新建" "找不到表单 (slug=$SL title=$TI create=$CR)"
    fi
  else
    fail "Wiki 新建" "找不到 New 按钮"
  fi
else
  fail "Wiki" "找不到导航按钮"
fi

# ── 4. Chat ──
echo ""; echo "── 4. Chat 会话列表 ──"
C=$(snap_ref 'button.*(Chat|聊天)')
if [ -n "$C" ]; then
  playwright-cli click "$C" > /dev/null 2>&1; sleep 2
  [ -n "$(snap_has 'Session|会话|New chat|Describe|描述|No session')" ] && ok "Chat 加载" || fail "Chat 加载" "未见会话/输入区"
  ERR=$(playwright-cli console error 2>/dev/null | grep -c 'TypeError\|Unexpected token|404.*message' || true)
  [ "$ERR" = "0" ] && ok "Chat 无 JS error" || fail "Chat console" "$ERR 个 error"
else
  fail "Chat" "找不到导航按钮"
fi

# ── 汇总 ──
echo ""; echo "========================================"
echo "  $LABEL 结果: $PASS_COUNT/$TOTAL passed"
[ "$FAIL_COUNT" -gt 0 ] && echo "  ❌ $FAIL_COUNT 项失败"
echo "========================================"
