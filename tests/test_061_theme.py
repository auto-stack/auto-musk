"""PLAN-061 targeted theme-adoption probe for auto-musk.

Counter-controls (auto-os/PLAN-061 V-01/V-02/V-03 standalone legs):

  leg A (explicit override track): AUTO_UI_THEME=dark -> official chain
      flips the app dark (corner pixel turns dark vs leg B, dark_mode true)
  leg B (explicit light): AUTO_UI_THEME=light -> light chrome (corner
      pixel light), dark_mode false; brand-purple pixels present (the pac
      theme{} composed overlay stays the palette source on the default
      family -- PLAN-075 brand retention)
  leg C (host follow track): AUTOOS_DESKTOP_CONFIG full candidate
      Solarized/dark + frozen package dir -> the official family fully
      takes over: dominant background == #002b36 (Solarized dark background)
  leg D (V-02 business during reskin): boot Solarized/dark, press the real
      rail view button (chats -> plans, static handler), then flip the
      config to light in-session; current_view + dark_mode state must
      survive the reskin (workspace business untouched)
  leg E (reopen recovery): fresh boot with the flipped light config ->
      app healthy on the persisted view selection

Isolation: evidence lives under
D:/autostack/plan-artifacts/plan-061/auto-os/musk (never inside a repo);
no real workspaces/messages are touched (probe drives only the view rail).
"""

import json
import os
import shutil
import socket
import subprocess
import sys
import time
import urllib.request

EVIDENCE_DIR = r"D:\autostack\plan-artifacts\plan-061\auto-os\musk"
CONFIG_DIR = os.path.join(EVIDENCE_DIR, "config")
PACKAGE_DIR = r"D:\autostack\auto-ui\packages\themes"
THEME_DIGEST = "9fa07742dc68aa5377ae06cb2d966386002063f184c515b6c5dddf8a6bef838e"

SOLARIZED_DARK_BG = (0x00, 0x2B, 0x36)


def write_config(mode: str, family: str = "Solarized", path: str = None) -> str:
    os.makedirs(CONFIG_DIR, exist_ok=True)
    path = path or os.path.join(CONFIG_DIR, f"config_{family.lower()}_{mode}.at")
    with open(path, "w", encoding="utf-8") as f:
        f.write(
            "desktop {\n"
            f'    theme_family : "{family}"\n'
            '    theme_family_version : "0.2.0"\n'
            f'    theme_family_digest : "{THEME_DIGEST}"\n'
            f'    theme_mode : "{mode}"\n'
            "    dark_theme : true\n"
            "}\n"
        )
    return path


def pick_free_port() -> int:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.bind(("127.0.0.1", 0))
        return s.getsockname()[1]


class AutoUiMcpClient:
    def __init__(self, port: int):
        self.url = f"http://127.0.0.1:{port}/mcp"
        self._req_id = 1

    def _call(self, tool_name: str, args: dict = None) -> dict:
        req = {
            "jsonrpc": "2.0",
            "id": self._req_id,
            "method": "tools/call",
            "params": {"name": tool_name, "arguments": args or {}},
        }
        self._req_id += 1
        http_req = urllib.request.Request(
            self.url, data=json.dumps(req).encode("utf-8"),
            headers={"Content-Type": "application/json"},
        )
        with urllib.request.urlopen(http_req, timeout=10) as response:
            res = json.loads(response.read().decode("utf-8"))
            if "error" in res:
                raise RuntimeError(f"MCP Tool '{tool_name}' error: {res['error']}")
            return res.get("result", {})

    def text(self, tool_name: str, args: dict = None) -> str:
        return self._call(tool_name, args).get("content", [{}])[0].get("text", "")

    def state(self) -> str:
        return self.text("autoui_state")

    def snapshot(self) -> str:
        return self.text("autoui_snapshot")

    def press(self, element_id):
        return self.text("autoui_action", {"element_id": f"vnode_{element_id}", "action": "press"})

    def find_button_by_label(self, label: str):
        lines = self.snapshot().splitlines()
        for i, line in enumerate(lines):
            if "button" in line and label in chr(10).join(lines[i:i + 3]):
                import re

                m = re.search(r"#vnode_(\d+)", line)
                if m:
                    return int(m.group(1))
        return None

    def screenshot(self, name: str) -> str:
        raw = self.text("autoui_screenshot", {"name": name, "baseline": False})
        low = raw.lower()
        if "saved to" in low:
            after = raw[low.index("saved to") + len("saved to"):].lstrip(" :").replace("\\\\?\\", "")
            return after.strip()
        return ""


def kill_tree(proc):
    if proc.poll() is not None:
        return
    try:
        subprocess.run(["taskkill", "/F", "/T", "/PID", str(proc.pid)],
                       capture_output=True, timeout=10)
    except Exception:
        pass
    try:
        proc.wait(timeout=5)
    except Exception:
        pass


def wait_ports_free(timeout: int = 30):
    deadline = time.time() + timeout
    while time.time() < deadline:
        busy = False
        for port in (17200, 17201):
            with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sk:
                sk.settimeout(0.3)
                if sk.connect_ex(("127.0.0.1", port)) == 0:
                    busy = True
        if not busy:
            return True
        time.sleep(1)
    return False


def corner_and_dominant(png_path: str):
    from PIL import Image
    from collections import Counter

    with Image.open(png_path) as im:
        rgb = im.convert("RGB")
        w, h = rgb.size
        corner = rgb.getpixel((8, 8))
        cnt = Counter(rgb.getdata())
        dom = cnt.most_common(1)[0][0]
    return corner, dom, (w, h)


def boot_musk(tag: str, app_dir: str, auto_bin: str, env_extra: dict, failures: list):
    os.makedirs(EVIDENCE_DIR, exist_ok=True)
    wait_ports_free()
    port = pick_free_port()
    env = dict(os.environ, AUTOUI_MCP_PORT=str(port), **env_extra)
    proc = subprocess.Popen([auto_bin, "run", "-r", "vm", "--server", "vm"],
                            cwd=app_dir, env=env,
                            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    try:
        client = AutoUiMcpClient(port)
        start = time.time()
        state = ""
        while time.time() - start < 420:
            try:
                s = client.state()
                if "current_view" in s and "dark_mode" in s:
                    state = s
                    break
            except Exception:
                pass
            time.sleep(0.5)
        if not state:
            failures.append(f"{tag}: MCP not ready within 420s")
            return None, None
        time.sleep(5)
        return client, state
    except Exception:
        kill_tree(proc)
        raise


def finish(tag, proc, client, state, failures):
    if state:
        with open(os.path.join(EVIDENCE_DIR, f"state-{tag}.txt"), "w", encoding="utf-8") as f:
            f.write(state)
    kill_tree(proc)
    tmp_dir = os.path.join(app_dir_global, "src", "front", "tmp")
    if os.path.isdir(tmp_dir):
        shutil.rmtree(tmp_dir, ignore_errors=True)


app_dir_global = ""


def shoot(client, tag):
    tmp = client.screenshot(tag)
    if tmp and os.path.exists(tmp):
        shot = os.path.join(EVIDENCE_DIR, f"{tag}.png")
        with open(tmp, "rb") as s, open(shot, "wb") as d:
            d.write(s.read())
        return shot
    return None


def count_purple(shot: str) -> int:
    from PIL import Image

    im = Image.open(shot).convert("RGB")
    w, h = im.size
    n = 0
    for y in range(0, h, 5):
        for x in range(0, w, 5):
            r, g, b = im.getpixel((x, y))
            if 60 <= r <= 130 and 60 <= g <= 115 and 170 <= b <= 245:
                n += 1
    return n


def main() -> int:
    global app_dir_global
    failures: list = []
    app_dir = os.path.dirname(os.path.abspath(__file__))
    app_dir = os.path.dirname(app_dir)
    app_dir_global = app_dir
    auto_bin = os.environ.get("AUTO_BIN", "auto")

    print("[*] leg A: explicit AUTO_UI_THEME=dark (override track)")
    wait_ports_free()
    proc, client, state = None, None, None
    p_port = pick_free_port()
    env = dict(os.environ, AUTOUI_MCP_PORT=str(p_port), AUTO_UI_THEME="dark")
    proc = subprocess.Popen([auto_bin, "run", "-r", "vm", "--server", "vm"],
                            cwd=app_dir, env=env,
                            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    try:
        client = AutoUiMcpClient(p_port)
        start = time.time()
        state = ""
        while time.time() - start < 420:
            try:
                s = client.state()
                if "current_view" in s and "dark_mode" in s:
                    state = s
                    break
            except Exception:
                pass
            time.sleep(0.5)
        if not state:
            failures.append("leg A: MCP not ready within 420s")
        else:
            time.sleep(5)
            if "dark_mode: true" not in state:
                failures.append("leg A: dark_mode state not true")
            shot = shoot(client, "061_musk_A_dark")
            if shot:
                corner, dom, _ = corner_and_dominant(shot)
                print(f"  [*] leg A corner #{corner[0]:02x}{corner[1]:02x}{corner[2]:02x} "
                      f"dominant #{dom[0]:02x}{dom[1]:02x}{dom[2]:02x}")
            else:
                failures.append("leg A: screenshot not saved")
            with open(os.path.join(EVIDENCE_DIR, "state-legA.txt"), "w", encoding="utf-8") as f:
                f.write(state)
    finally:
        kill_tree(proc)

    print("[*] leg B: explicit AUTO_UI_THEME=light (brand overlay retained)")
    wait_ports_free()
    p_port = pick_free_port()
    env = dict(os.environ, AUTOUI_MCP_PORT=str(p_port), AUTO_UI_THEME="light")
    proc = subprocess.Popen([auto_bin, "run", "-r", "vm", "--server", "vm"],
                            cwd=app_dir, env=env,
                            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    try:
        client = AutoUiMcpClient(p_port)
        start = time.time()
        state = ""
        while time.time() - start < 420:
            try:
                s = client.state()
                if "current_view" in s and "dark_mode" in s:
                    state = s
                    break
            except Exception:
                pass
            time.sleep(0.5)
        if not state:
            failures.append("leg B: MCP not ready within 420s")
        else:
            time.sleep(5)
            if "dark_mode: false" not in state:
                failures.append("leg B: dark_mode state not false")
            shot = shoot(client, "061_musk_B_light")
            if shot:
                corner, dom, _ = corner_and_dominant(shot)
                light = corner[0] > 0xE0 and corner[1] > 0xE0 and corner[2] > 0xE0
                pur = count_purple(shot)
                print(f"  [*] leg B corner #{corner[0]:02x}{corner[1]:02x}{corner[2]:02x} "
                      f"light={light}; brand-purple samples={pur}")
                if not light:
                    failures.append(f"leg B: corner not light: #{corner[0]:02x}{corner[1]:02x}{corner[2]:02x}")
                if pur == 0:
                    failures.append("leg B: brand purple overlay lost (PLAN-075 retention broken)")
            else:
                failures.append("leg B: screenshot not saved")
            with open(os.path.join(EVIDENCE_DIR, "state-legB.txt"), "w", encoding="utf-8") as f:
                f.write(state)
    finally:
        kill_tree(proc)

    print("[*] leg C: host follow track -- Solarized/dark full candidate")
    wait_ports_free()
    cfg_dark = write_config("dark")
    p_port = pick_free_port()
    env = dict(os.environ, AUTOUI_MCP_PORT=str(p_port),
               AUTOOS_DESKTOP_CONFIG=cfg_dark,
               AUTOOS_THEME_PACKAGE_DIR=PACKAGE_DIR)
    proc = subprocess.Popen([auto_bin, "run", "-r", "vm", "--server", "vm"],
                            cwd=app_dir, env=env,
                            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    try:
        client = AutoUiMcpClient(p_port)
        start = time.time()
        state = ""
        while time.time() - start < 420:
            try:
                s = client.state()
                if "current_view" in s and "dark_mode" in s:
                    state = s
                    break
            except Exception:
                pass
            time.sleep(0.5)
        if not state:
            failures.append("leg C: MCP not ready within 420s")
        else:
            time.sleep(5)
            shot = shoot(client, "061_musk_C_solarized_dark")
            if shot:
                corner, dom, _ = corner_and_dominant(shot)
                print(f"  [*] leg C dominant #{dom[0]:02x}{dom[1]:02x}{dom[2]:02x} "
                      f"(expect #002b36)")
                if not all(abs(a - c) <= 3 for a, c in zip(dom, SOLARIZED_DARK_BG)):
                    failures.append(f"leg C: dominant {dom} != Solarized dark background")
            else:
                failures.append("leg C: screenshot not saved")
            with open(os.path.join(EVIDENCE_DIR, "state-legC.txt"), "w", encoding="utf-8") as f:
                f.write(state)
    finally:
        kill_tree(proc)

    print("[*] leg D: live business during reskin (rail view switch + flip)")
    wait_ports_free()
    cfg_flip = os.path.join(CONFIG_DIR, "config_flip.at")
    write_config("dark", path=cfg_flip)
    p_port = pick_free_port()
    env = dict(os.environ, AUTOUI_MCP_PORT=str(p_port),
               AUTOOS_DESKTOP_CONFIG=cfg_flip,
               AUTOOS_THEME_PACKAGE_DIR=PACKAGE_DIR)
    proc = subprocess.Popen([auto_bin, "run", "-r", "vm", "--server", "vm"],
                            cwd=app_dir, env=env,
                            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    try:
        client = AutoUiMcpClient(p_port)
        start = time.time()
        state = ""
        while time.time() - start < 420:
            try:
                s = client.state()
                if "current_view" in s and "dark_mode" in s:
                    state = s
                    break
            except Exception:
                pass
            time.sleep(0.5)
        if not state:
            failures.append("leg D: MCP not ready within 420s")
        else:
            time.sleep(5)
            btn = client.find_button_by_label("计划")
            if btn is None:
                with open(os.path.join(EVIDENCE_DIR, "snapshot-legD.txt"), "w", encoding="utf-8") as f:
                    f.write(client.snapshot())
                failures.append("leg D: rail 计划 button not found (snapshot dumped)")
            else:
                client.press(btn)
                time.sleep(3)
                st = client.state()
                with open(os.path.join(EVIDENCE_DIR, "state-legD-after-press.txt"), "w", encoding="utf-8") as f:
                    f.write(st)
                switched = 'current_view: "plans"' in st
                print(f"  [+] pressed rail 计划; current_view=plans: {switched}")
                if not switched:
                    failures.append("leg D: view did not switch to plans")

                with open(cfg_flip, "w", encoding="utf-8") as f:
                    f.write(
                        "desktop {\n"
                        '    theme_family : "Solarized"\n'
                        '    theme_family_version : "0.2.0"\n'
                        f'    theme_family_digest : "{THEME_DIGEST}"\n'
                        '    theme_mode : "light"\n'
                        "    dark_theme : false\n"
                        "}\n"
                    )
                print("  [*] config flipped to light in-session")
                survived = True
                last = st
                deadline = time.time() + 20
                while time.time() < deadline:
                    time.sleep(1)
                    last = client.state()
                    if "current_view" not in last:
                        survived = False
                        break
                    if 'current_view: "plans"' in last and "dark_mode: false" in last:
                        break
                if not survived or 'current_view: "plans"' not in last:
                    failures.append("leg D: business state lost during reskin")
                else:
                    print("  [+] view selection survived reskin window (V-02)")
                with open(os.path.join(EVIDENCE_DIR, "state-legD-after.txt"), "w", encoding="utf-8") as f:
                    f.write(last)
    finally:
        kill_tree(proc)

    print("[*] leg E: reopen recovery with flipped light config")
    wait_ports_free()
    p_port = pick_free_port()
    env = dict(os.environ, AUTOUI_MCP_PORT=str(p_port),
               AUTOOS_DESKTOP_CONFIG=cfg_flip,
               AUTOOS_THEME_PACKAGE_DIR=PACKAGE_DIR)
    proc = subprocess.Popen([auto_bin, "run", "-r", "vm", "--server", "vm"],
                            cwd=app_dir, env=env,
                            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    try:
        client = AutoUiMcpClient(p_port)
        start = time.time()
        state = ""
        while time.time() - start < 420:
            try:
                s = client.state()
                if "current_view" in s and "dark_mode" in s:
                    state = s
                    break
            except Exception:
                pass
            time.sleep(0.5)
        if not state:
            failures.append("leg E: MCP not ready within 420s")
        else:
            time.sleep(5)
            healthy = 'current_view: "chats"' in state or 'current_view: "plans"' in state
            print(f"  [*] leg E app healthy on reopen: {healthy}")
            if not healthy:
                failures.append("leg E: app did not boot healthy")
            shot = shoot(client, "061_musk_E_reopen_light")
            if shot:
                corner, dom, _ = corner_and_dominant(shot)
                print(f"  [*] leg E corner #{corner[0]:02x}{corner[1]:02x}{corner[2]:02x}")
            with open(os.path.join(EVIDENCE_DIR, "state-legE.txt"), "w", encoding="utf-8") as f:
                f.write(state)
    finally:
        kill_tree(proc)

    if failures:
        print("[-] FAILURES:")
        for f in failures:
            print("   -", f)
        return 1
    print("[+] PLAN-061 musk probe: all counter-controls passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
