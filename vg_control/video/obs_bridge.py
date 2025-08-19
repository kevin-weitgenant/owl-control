#!/usr/bin/env python3
"""
OBS Bridge Script
Simplified bridge to interface with OBS from Rust, using existing OBS client functionality.
Communicates via stdin/stdout JSON messages.
"""

import sys
import json
import time
import threading
from typing import Optional
from pathlib import Path

from .obs_client import OBSClient

class OBSBridge:
    def __init__(self):
        self.obs_client: Optional[OBSClient] = None
        self.recording_active = False
        self.start_time: Optional[float] = None
        self.stop_time: Optional[float] = None
        
    def send_message(self, message_type: str, data: dict = None):
        """Send JSON message to stdout"""
        message = {
            "type": message_type,
            "data": data or {},
            "timestamp": time.time()
        }
        print(json.dumps(message), flush=True)
        
    def handle_start_callback(self, timestamp: float):
        """Callback when recording starts"""
        self.start_time = timestamp
        self.recording_active = True
        self.send_message("recording_started", {"timestamp": timestamp})
        
    def handle_stop_callback(self, timestamp: float):
        """Callback when recording stops"""
        self.stop_time = timestamp
        self.recording_active = False
        self.send_message("recording_stopped", {"timestamp": timestamp})
        
    def initialize_obs(self, recording_path: str):
        """Initialize OBS client with recording path"""
        try:
            self.obs_client = OBSClient(
                recording_path=recording_path,
                start_callback=self.handle_start_callback,
                end_callback=self.handle_stop_callback
            )
            self.send_message("initialized", {"recording_path": recording_path})
            return True
        except Exception as e:
            self.send_message("error", {"message": f"Failed to initialize OBS: {str(e)}"})
            return False
            
    def start_recording(self):
        """Start OBS recording"""
        if not self.obs_client:
            self.send_message("error", {"message": "OBS not initialized"})
            return
            
        try:
            self.obs_client.start_recording()
            self.send_message("start_requested")
        except Exception as e:
            self.send_message("error", {"message": f"Failed to start recording: {str(e)}"})
            
    def stop_recording(self):
        """Stop OBS recording"""
        if not self.obs_client:
            self.send_message("error", {"message": "OBS not initialized"})
            return
            
        try:
            self.obs_client.stop_recording()
            self.send_message("stop_requested")
        except Exception as e:
            self.send_message("error", {"message": f"Failed to stop recording: {str(e)}"})
            
    def status(self):
        """Get current status"""
        status_data = {
            "recording_active": self.recording_active,
            "start_time": self.start_time,
            "stop_time": self.stop_time,
            "initialized": self.obs_client is not None
        }
        self.send_message("status", status_data)
        
    def handle_command(self, command: dict):
        """Handle incoming command"""
        cmd_type = command.get("type")
        data = command.get("data", {})
        
        if cmd_type == "initialize":
            recording_path = data.get("recording_path")
            if not recording_path:
                self.send_message("error", {"message": "No recording_path provided"})
                return
            self.initialize_obs(recording_path)
            
        elif cmd_type == "start":
            self.start_recording()
            
        elif cmd_type == "stop":
            self.stop_recording()
            
        elif cmd_type == "status":
            self.status()
            
        elif cmd_type == "shutdown":
            self.send_message("shutdown_ack")
            return False  # Signal to exit
            
        else:
            self.send_message("error", {"message": f"Unknown command: {cmd_type}"})
            
        return True
        
    def run(self):
        """Main loop - read commands from stdin"""
        self.send_message("ready")
        
        try:
            for line in sys.stdin:
                line = line.strip()
                if not line:
                    continue
                    
                try:
                    command = json.loads(line)
                    if not self.handle_command(command):
                        break  # Shutdown requested
                        
                except json.JSONDecodeError as e:
                    self.send_message("error", {"message": f"Invalid JSON: {str(e)}"})
                    
        except KeyboardInterrupt:
            pass
        except Exception as e:
            self.send_message("error", {"message": f"Unexpected error: {str(e)}"})
            
        # Cleanup
        if self.obs_client and self.recording_active:
            try:
                self.obs_client.stop_recording()
            except:
                pass


def main():
    """Entry point for standalone execution"""
    bridge = OBSBridge()
    bridge.run()


if __name__ == "__main__":
    main()