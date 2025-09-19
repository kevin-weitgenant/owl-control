import os
from typing import List, Optional
from datetime import datetime
import requests
import subprocess
import shlex
from tqdm import tqdm
import time

from ..constants import API_BASE_URL


def get_upload_url(
    api_key: str,
    archive_path: str,
    tags: Optional[List[str]] = None,
    base_url: str = API_BASE_URL,
    video_filename: Optional[str] = None,
    control_filename: Optional[str] = None,
    video_duration_seconds: Optional[float] = None,
    video_width: Optional[int] = None,
    video_height: Optional[int] = None,
    video_codec: Optional[str] = None,
    video_fps: Optional[float] = None,
) -> str:
    """Request a pre-signed S3 URL for uploading a tar archive."""

    file_size = os.path.getsize(archive_path)
    file_size_mb = file_size // (1024 * 1024)
    payload = {
        "filename": os.path.basename(archive_path),
        "content_type": "application/x-tar",
        "file_size_mb": file_size_mb,
        "expiration": 14400,
        "uploader_hwid": get_hwid(),
        "upload_timestamp": datetime.now().isoformat(),
    }
    if tags:
        payload["tags"] = tags
    if video_filename:
        payload["video_filename"] = video_filename
    if control_filename:
        payload["control_filename"] = control_filename
    if video_duration_seconds is not None:
        payload["video_duration_seconds"] = video_duration_seconds
    if video_width is not None:
        payload["video_width"] = video_width
    if video_height is not None:
        payload["video_height"] = video_height
    if video_codec:
        payload["video_codec"] = video_codec
    if video_fps is not None:
        payload["video_fps"] = video_fps

    headers = {"Content-Type": "application/json", "X-API-Key": api_key}
    url = f"{base_url}/tracker/upload/game_control"

    with requests.Session() as session:
        response = session.post(url, headers=headers, json=payload, timeout=30)
        response.raise_for_status()
        data = response.json()
        return data.get("url") or data.get("upload_url") or data["uploadUrl"]


def upload_archive(
    api_key: str,
    archive_path: str,
    tags: Optional[List[str]] = None,
    base_url: str = API_BASE_URL,
    progress_mode: bool = False,
    video_filename: Optional[str] = None,
    control_filename: Optional[str] = None,
    video_duration_seconds: Optional[float] = None,
    video_width: Optional[int] = None,
    video_height: Optional[int] = None,
    video_codec: Optional[str] = None,
    video_fps: Optional[float] = None,
) -> None:
    """Upload an archive to the storage bucket via a pre-signed URL."""

    upload_url = get_upload_url(
        api_key,
        archive_path,
        tags=tags,
        base_url=base_url,
        video_filename=video_filename,
        control_filename=control_filename,
        video_duration_seconds=video_duration_seconds,
        video_width=video_width,
        video_height=video_height,
        video_codec=video_codec,
        video_fps=video_fps,
    )

    # Get file size for progress bar
    import os

    file_size = os.path.getsize(archive_path)

    # Initialize progress file
    if progress_mode:
        import tempfile
        import json

        progress_file = os.path.join(
            tempfile.gettempdir(), "owl-control-upload-progress.json"
        )
        initial_progress = {
            "phase": "upload",
            "action": "start",
            "bytes_uploaded": 0,
            "total_bytes": file_size,
            "percent": 0,
            "speed_mbps": 0,
            "eta_seconds": 0,
            "timestamp": time.time(),
        }
        try:
            with open(progress_file, "w") as f:
                json.dump(initial_progress, f)
        except Exception as e:
            print(f"Warning: Could not initialize progress file: {e}")

    def emit_upload_progress(bytes_uploaded, total_bytes, speed_bps=0):
        """Write JSON progress data to file for UI consumption"""
        if progress_mode:
            import json
            import tempfile
            import os

            progress_data = {
                "phase": "upload",
                "action": "progress",
                "bytes_uploaded": bytes_uploaded,
                "total_bytes": total_bytes,
                "percent": min((bytes_uploaded / total_bytes) * 100, 100)
                if total_bytes > 0
                else 0,
                "speed_mbps": speed_bps / (1024 * 1024) if speed_bps > 0 else 0,
                "eta_seconds": ((total_bytes - bytes_uploaded) / speed_bps)
                if speed_bps > 0
                else 0,
                "timestamp": time.time(),
            }

            # Write to temp file for UI to read
            progress_file = os.path.join(
                tempfile.gettempdir(), "owl-control-upload-progress.json"
            )
            try:
                with open(progress_file, "w") as f:
                    json.dump(progress_data, f)
            except Exception as e:
                print(f"Warning: Could not write progress file: {e}")

            # Also print for console (keep existing behavior)
            print(f"PROGRESS: {json.dumps(progress_data)}")

    # Use -# for a simpler progress indicator that's easier to parse
    # Build curl args explicitly for Windows compatibility; avoid shlex splitting
    # Harden upload with longer timeouts, HTTP/1.1, disabled Expect: 100-continue, keepalives, retries, and slow-speed detection
    curl_args = [
        "curl",
        "-X",
        "PUT",
        f"{upload_url}",
        "-k",
        "-H",
        "Content-Type: application/x-tar",
        "-H",
        "Expect:",
        "--http1.1",
        "--keepalive-time",
        "60",
        "--connect-timeout",
        "30",
        "--max-time",
        "5400",
        "--retry",
        "3",
        "--retry-delay",
        "2",
        # Use numeric bytes/sec for maximum compatibility (102400 = 100 KB/s)
        "--speed-limit",
        "102400",
        "--speed-time",
        "120",
        "-T",
        f"{archive_path}",
        "-#",
        "--no-buffer",
    ]

    # Debug: log the upload URL (hide sensitive parts)
    from urllib.parse import urlparse

    parsed_url = urlparse(upload_url)

    # Write to debug log file
    import tempfile

    debug_log_path = os.path.join(tempfile.gettempdir(), "owl-control-debug.log")
    try:
        with open(debug_log_path, "a") as debug_file:
            debug_file.write(
                f"[{datetime.now().isoformat()}] PYTHON: Uploading to host: {parsed_url.netloc}\n"
            )
            debug_file.write(
                f"[{datetime.now().isoformat()}] PYTHON: Full URL length: {len(upload_url)} chars\n"
            )
    except:
        pass  # Don't fail if debug logging fails

    with tqdm(total=file_size, unit="B", unit_scale=True, desc="Uploading") as pbar:
        process = subprocess.Popen(
            curl_args,
            stderr=subprocess.PIPE,
            bufsize=1,  # Line buffered
            universal_newlines=True,
        )

        last_update = 0
        start_time = time.time() if progress_mode else 0

        # Read curl's progress output
        stderr_tail = []  # Keep last ~100 lines for diagnostics
        while True:
            line = process.stderr.readline()
            if not line:
                break

            # Update progress bar based on curl's output
            if "#" in line:
                try:
                    # Extract percentage from the number of # characters
                    percent = min((line.count("#") / 50) * 100, 100)  # Cap at 100%
                    current = min(
                        int(file_size * (percent / 100)), file_size
                    )  # Cap at file size

                    # Only update if we've made progress to avoid unnecessary refreshes
                    if current > last_update:
                        pbar.n = current
                        pbar.refresh()

                        # Calculate speed and emit progress for UI
                        if progress_mode and start_time > 0:
                            elapsed_time = time.time() - start_time
                            speed_bps = (
                                current / elapsed_time if elapsed_time > 0 else 0
                            )
                            emit_upload_progress(current, file_size, speed_bps)

                        last_update = current
                except:
                    continue
            # Maintain a tail buffer for error diagnostics
            stderr_tail.append(line.rstrip("\n"))
            if len(stderr_tail) > 100:
                stderr_tail.pop(0)

        # Wait for process to complete
        return_code = process.wait()

        # Cleanup progress file
        if progress_mode:
            import tempfile
            import os

            progress_file = os.path.join(
                tempfile.gettempdir(), "owl-control-upload-progress.json"
            )
            try:
                if os.path.exists(progress_file):
                    # Write final completion state
                    final_progress = {
                        "phase": "upload",
                        "action": "complete",
                        "bytes_uploaded": file_size,
                        "total_bytes": file_size,
                        "percent": 100,
                        "speed_mbps": 0,
                        "eta_seconds": 0,
                        "timestamp": time.time(),
                    }
                    with open(progress_file, "w") as f:
                        json.dump(final_progress, f)
            except Exception as e:
                print(f"Warning: Could not write final progress: {e}")

        if return_code != 0:
            # Append tail of stderr to debug log to help diagnose e.g. unknown options (exit 2)
            try:
                with open(debug_log_path, "a") as debug_file:
                    debug_file.write(
                        f"[{datetime.now().isoformat()}] CURL exited {return_code}. Last stderr lines:\n"
                    )
                    for ln in stderr_tail[-20:]:
                        debug_file.write(f"    {ln}\n")
            except:
                pass

            # Fallback: retry once with minimal, broadly compatible curl flags
            minimal_args = [
                "curl",
                "-X",
                "PUT",
                f"{upload_url}",
                "-H",
                "Content-Type: application/x-tar",
                "-H",
                "Expect:",
                "--http1.1",
                "-T",
                f"{archive_path}",
                "-#",
            ]

            try:
                with open(debug_log_path, "a") as debug_file:
                    debug_file.write(
                        f"[{datetime.now().isoformat()}] Retrying upload with minimal curl flags...\n"
                    )
            except:
                pass

            process2 = subprocess.Popen(
                minimal_args,
                stderr=subprocess.PIPE,
                bufsize=1,
                universal_newlines=True,
            )

            stderr_tail2 = []
            while True:
                line2 = process2.stderr.readline()
                if not line2:
                    break
                stderr_tail2.append(line2.rstrip("\n"))
                if len(stderr_tail2) > 100:
                    stderr_tail2.pop(0)

            return_code2 = process2.wait()
            if return_code2 != 0:
                try:
                    with open(debug_log_path, "a") as debug_file:
                        debug_file.write(
                            f"[{datetime.now().isoformat()}] Fallback CURL exited {return_code2}. Last stderr lines:\n"
                        )
                        for ln in stderr_tail2[-20:]:
                            debug_file.write(f"    {ln}\n")
                except:
                    pass
                raise Exception(f"Upload failed with return code {return_code2}")


def get_hwid():
    try:
        with open("/sys/class/dmi/id/product_uuid", "r") as f:
            hardware_id = f.read().strip()
    except:
        try:
            # Fallback for Windows
            import subprocess

            output = subprocess.check_output("wmic csproduct get uuid").decode()
            hardware_id = output.split("\n")[1].strip()
        except:
            hardware_id = None
    return hardware_id
