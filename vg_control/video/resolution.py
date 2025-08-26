import ctypes
from ctypes import wintypes

user32 = ctypes.windll.user32
shcore = ctypes.windll.shcore

# --- 1) Set DPI awareness early ---
# Try PER_MONITOR_AWARE_V2 (-4). If it fails (older Windows), fall back.
DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2 = ctypes.c_void_p(-4)
try:
    user32.SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2)
except Exception:
    # Fallback to PROCESS_PER_MONITOR_DPI_AWARE = 2
    try:
        shcore.SetProcessDpiAwareness(2)
    except Exception:
        # Last resort: legacy system-wide awareness
        user32.SetProcessDPIAware()

# --- structs & constants ---
MONITORINFOF_PRIMARY = 0x00000001
ENUM_CURRENT_SETTINGS = -1

class RECT(ctypes.Structure):
    _fields_ = [('left', wintypes.LONG),
                ('top', wintypes.LONG),
                ('right', wintypes.LONG),
                ('bottom', wintypes.LONG)]

class MONITORINFOEXW(ctypes.Structure):
    _fields_ = [('cbSize', wintypes.DWORD),
                ('rcMonitor', RECT),
                ('rcWork', RECT),
                ('dwFlags', wintypes.DWORD),
                ('szDevice', wintypes.WCHAR * 32)]

class DEVMODEW(ctypes.Structure):
    _fields_ = [
        ('dmDeviceName', wintypes.WCHAR * 32),
        ('dmSpecVersion', wintypes.WORD),
        ('dmDriverVersion', wintypes.WORD),
        ('dmSize', wintypes.WORD),
        ('dmDriverExtra', wintypes.WORD),
        ('dmFields', wintypes.DWORD),
        ('dmOrientation', wintypes.SHORT),
        ('dmPaperSize', wintypes.SHORT),
        ('dmPaperLength', wintypes.SHORT),
        ('dmPaperWidth', wintypes.SHORT),
        ('dmScale', wintypes.SHORT),
        ('dmCopies', wintypes.SHORT),
        ('dmDefaultSource', wintypes.SHORT),
        ('dmPrintQuality', wintypes.SHORT),
        ('dmColor', wintypes.SHORT),
        ('dmDuplex', wintypes.SHORT),
        ('dmYResolution', wintypes.SHORT),
        ('dmTTOption', wintypes.SHORT),
        ('dmCollate', wintypes.SHORT),
        ('dmFormName', wintypes.WCHAR * 32),
        ('dmLogPixels', wintypes.WORD),
        ('dmBitsPerPel', wintypes.DWORD),
        ('dmPelsWidth', wintypes.DWORD),
        ('dmPelsHeight', wintypes.DWORD),
        ('dmDisplayFlags', wintypes.DWORD),
        ('dmDisplayFrequency', wintypes.DWORD),
        ('dmICMMethod', wintypes.DWORD),
        ('dmICMIntent', wintypes.DWORD),
        ('dmMediaType', wintypes.DWORD),
        ('dmDitherType', wintypes.DWORD),
        ('dmReserved1', wintypes.DWORD),
        ('dmReserved2', wintypes.DWORD),
        ('dmPanningWidth', wintypes.DWORD),
        ('dmPanningHeight', wintypes.DWORD),
    ]

MonitorEnumProc = ctypes.WINFUNCTYPE(
    wintypes.BOOL,
    wintypes.HMONITOR,
    wintypes.HDC,
    ctypes.POINTER(RECT),
    wintypes.LPARAM
)

# helpers
def get_monitor_info(hmon):
    info = MONITORINFOEXW()
    info.cbSize = ctypes.sizeof(MONITORINFOEXW)
    if not user32.GetMonitorInfoW(hmon, ctypes.byref(info)):
        raise ctypes.WinError()
    return info

def get_native_mode(device_name):
    devmode = DEVMODEW()
    devmode.dmSize = ctypes.sizeof(DEVMODEW)
    if not ctypes.windll.user32.EnumDisplaySettingsW(device_name, ENUM_CURRENT_SETTINGS, ctypes.byref(devmode)):
        raise ctypes.WinError()
    return devmode.dmPelsWidth, devmode.dmPelsHeight, devmode.dmDisplayFrequency, devmode.dmBitsPerPel

def get_scale_factor(hmon):
    # GetDpiForMonitor (Win 8.1+). Scale ~= dpi/96.
    MDT_EFFECTIVE_DPI = 0
    dpiX = wintypes.UINT()
    dpiY = wintypes.UINT()
    try:
        hr = shcore.GetDpiForMonitor(hmon, MDT_EFFECTIVE_DPI, ctypes.byref(dpiX), ctypes.byref(dpiY))
        if hr == 0:  # S_OK
            return dpiX.value / 96.0, dpiY.value / 96.0
    except Exception:
        pass
    return None, None

monitors = []

def enum_proc(hmon, hdc, lprc, lparam):
    info = get_monitor_info(hmon)
    native_w, native_h, hz, bpp = get_native_mode(info.szDevice)
    sx, sy = get_scale_factor(hmon)
    monitors.append({
        "device": info.szDevice,
        "primary": bool(info.dwFlags & MONITORINFOF_PRIMARY),
        "rect_px": (info.rcMonitor.left, info.rcMonitor.top, info.rcMonitor.right, info.rcMonitor.bottom),
        "native_w": int(native_w),
        "native_h": int(native_h),
        "refresh_hz": int(hz),
        "bits_per_pixel": int(bpp),
        "scale_x": sx,
        "scale_y": sy,
    })
    return True

"""
if not user32.EnumDisplayMonitors(0, 0, MonitorEnumProc(enum_proc), 0):
    raise ctypes.WinError()
for m in monitors:
    l, t, r, b = m["rect_px"]
    logical_w = r - l
    logical_h = b - t
    sx = m["scale_x"]
    sy = m["scale_y"]
    print(f"{m['device']}{' (PRIMARY)' if m['primary'] else ''}")
    print(f"  Native:  {m['native_w']} x {m['native_h']}  @ {m['refresh_hz']}Hz, {m['bits_per_pixel']}bpp")
    if sx and sy:
        print(f"  Scale:   {sx*100:.0f}% (X), {sy*100:.0f}% (Y)")
    print(f"  Rect px: {logical_w} x {logical_h}  (monitor rect in this DPI-aware process)")
    print()
"""

def get_primary_monitor_resolution():
    """
    Returns the resolution of the primary monitor as a tuple (width, height).
    Returns None if no primary monitor is found.
    """
    # Clear any existing monitors data
    global monitors
    monitors.clear()
    
    # Enumerate all monitors
    if not user32.EnumDisplayMonitors(0, 0, MonitorEnumProc(enum_proc), 0):
        raise ctypes.WinError()
    
    # Find the primary monitor
    for monitor in monitors:
        if monitor["primary"]:
            return (int(monitor["native_w"]), int(monitor["native_h"]))
    
    return None