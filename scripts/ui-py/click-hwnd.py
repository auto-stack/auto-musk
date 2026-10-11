#!/usr/bin/env python3
"""click-hwnd.py —— musk 窗口逻辑域原子点击（760 e2e send_click 移植 +
自寻窗：找窗→客户区→前台→真移→PostMessage 单进程原子链，消除探针两步
定位间的窗口漂移窗口期——实测窗口在 rect 探测与点击两步间移动 +362/+16
物理像素）。用法：python click-hwnd.py <logical_x> <logical_y> [dpi]"""
import ctypes, sys, time

def main():
    x = int(sys.argv[1]); y = int(sys.argv[2]); dpi = float(sys.argv[3]) if len(sys.argv) > 3 else 2.0
    u32 = ctypes.windll.user32
    u32.SetProcessDPIAware()

    class RECT(ctypes.Structure):
        _fields_ = [("L", ctypes.c_long), ("T", ctypes.c_long), ("R", ctypes.c_long), ("B", ctypes.c_long)]
    class P(ctypes.Structure):
        _fields_ = [("X", ctypes.c_long), ("Y", ctypes.c_long)]

    # 自寻窗：标题含 Musk × 非零客户区（EnumWindows 委托）。
    EnumWindowsProc = ctypes.WINFUNCTYPE(ctypes.c_bool, ctypes.c_void_p, ctypes.c_void_p)
    hits = []
    def cb(h, l):
        n = ctypes.create_unicode_buffer(256)
        u32.GetWindowTextW(h, n, 256)
        if 'Musk' in n.value and u32.IsWindowVisible(h):
            r = RECT()
            u32.GetClientRect(h, ctypes.byref(r))
            if r.R - r.L > 200 and r.B - r.T > 200:
                hits.append(h)
        return True
    u32.EnumWindows(EnumWindowsProc(cb), 0)
    if not hits:
        print("nowin")
        return
    hwnd = hits[0]

    pt = P(0, 0)
    u32.ClientToScreen(hwnd, ctypes.byref(pt))
    ox, oy = pt.X, pt.Y

    fg = u32.GetForegroundWindow()
    if fg != hwnd:
        ct = ctypes.windll.kernel32.GetCurrentThreadId()
        ft = u32.GetWindowThreadProcessId(fg, None)
        tt = u32.GetWindowThreadProcessId(hwnd, None)
        u32.AttachThreadInput(ct, ft, True)
        u32.AttachThreadInput(ct, tt, True)
        u32.BringWindowToTop(hwnd)
        u32.SetForegroundWindow(hwnd)
        u32.AttachThreadInput(ct, ft, False)
        u32.AttachThreadInput(ct, tt, False)
        time.sleep(0.3)
        # 前台化后重取客户原点（防移动）。
        u32.ClientToScreen(hwnd, ctypes.byref(pt))
        ox, oy = pt.X, pt.Y

    sx, sy = int(ox + x * dpi), int(oy + y * dpi)
    ok = u32.SetCursorPos(sx, sy)
    time.sleep(0.15)
    cur = P(0, 0)
    u32.GetCursorPos(ctypes.byref(cur))
    cx, cy = int(cur.X - ox), int(cur.Y - oy)
    lp = (cy << 16) | (cx & 0xFFFF)
    for _ in range(3):
        ok1 = u32.PostMessageW(hwnd, 0x0200, 0, lp)
        time.sleep(0.06)
        ok2 = u32.PostMessageW(hwnd, 0x0201, 0x0001, lp)
        time.sleep(0.06)
        ok3 = u32.PostMessageW(hwnd, 0x0202, 0, lp)
        time.sleep(0.06)
        u32.PostMessageW(hwnd, 0x0200, 0, lp)
        if ok1 and ok2 and ok3:
            print(f"clicked ox={ox},oy={oy} lp=({cx},{cy}) move={ok} at=({cur.X},{cur.Y})")
            return
        time.sleep(0.2)
    print("post-failed")

main()
