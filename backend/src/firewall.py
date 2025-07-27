from server import SERVER_PORT
import platform
import ctypes
import tempfile

FIREWALL_RULE = f"""
$ruleName = "Allow Clipboard Server"
Get-NetFirewallRule -DisplayName $ruleName -ErrorAction SilentlyContinue

if ($error) {{
    New-NetFirewallRule -DisplayName $ruleName `
        -Direction Inbound -Protocol TCP -LocalPort {SERVER_PORT} `
        -Action Allow -Profile Any
}}
"""


def update_firewall():
    if platform.system() == "Windows":
        # Save the script to a temp .ps1 file
        with tempfile.NamedTemporaryFile(
            suffix=".ps1", mode="w", encoding="utf-8"
        ) as f:
            f.write(FIREWALL_RULE)
            script_path = f.name

            # Use ShellExecute with "runas" to elevate
            ret = ctypes.windll.shell32.ShellExecuteW(
                None,
                "runas",
                "powershell.exe",
                f'-ExecutionPolicy Bypass -File "{script_path}"',
                None,
                0,
            )

            if ret <= 32:
                print("Failed to elevate. User may have canceled the UAC prompt.")
            else:
                print("PowerShell script launched with admin privileges.")
