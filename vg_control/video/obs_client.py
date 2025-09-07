from typing import Callable

import obsws_python as obs
import time
import sys

import os
import subprocess

from ..constants import FPS, RECORDING_WIDTH, RECORDING_HEIGHT, VIDEO_BITRATE, SET_ENCODER
from .resolution import get_primary_monitor_resolution
"""
OBS websocket association with params on OBS client is finnicky. Listing a few so I don't forget.

"SimpleOutput" "FilePath" <-- recording path
"SimpleOutput" "VBitrate" <-- video bitrate
- "Preset" = "veryfast"
- "RecQuality" = "Stream" 

You can see the rest in the %appdata% folder for obs-studio
There are ini files for the presets
"""

def try_launch_obs_if_not_open():
    # First try to find OBS via registry (most reliable)
    try:
        import winreg
        # Check both 32-bit and 64-bit registry views
        reg_paths = [
            (winreg.HKEY_LOCAL_MACHINE, r"SOFTWARE\OBS Studio"),
            (winreg.HKEY_LOCAL_MACHINE, r"SOFTWARE\WOW6432Node\OBS Studio"),
        ]
        for hkey, reg_path in reg_paths:
            try:
                with winreg.OpenKey(hkey, reg_path) as key:
                    install_path = winreg.QueryValueEx(key, "")[0]
                    obs_exe = os.path.join(install_path, "bin", "64bit", "obs64.exe")
                    if os.path.exists(obs_exe):
                        obs_dir = os.path.dirname(obs_exe)
                        subprocess.Popen([obs_exe], cwd=obs_dir)
                        time.sleep(10)
                        return
            except (WindowsError, OSError):
                continue
    except ImportError:
        pass  # winreg not available, fall back to file search
    
    # Fall back to checking common installation paths on all drives
    import string
    obs_paths = []
    
    # Get all available drive letters
    drives = [f"{d}:\\" for d in string.ascii_uppercase if os.path.exists(f"{d}:\\")]
    
    # Check common paths on all drives
    for drive in drives:
        obs_paths.extend([
            os.path.join(drive, "Program Files", "obs-studio", "bin", "64bit", "obs64.exe"),
            os.path.join(drive, "Program Files (x86)", "obs-studio", "bin", "64bit", "obs64.exe"),
        ])
    
    for path in obs_paths:
        if os.path.exists(path):
            # Get the directory containing obs64.exe
            obs_dir = os.path.dirname(path)
            # Launch OBS from its directory
            subprocess.Popen([path], cwd=obs_dir)
            # Give OBS a moment to start up
            time.sleep(10)
            return
            
    raise FileNotFoundError("Could not find OBS installation. Please ensure OBS Studio is installed and try launching it manually first.")

class OBSClient:
    """
    Very simple client to interact with OBS. Assumes OBS is already focused on game (just set soruce to game capture and it should do this for you)
    """
    def __init__(
        self, 
        recording_path: str,
        start_callback : Callable,
        end_callback : Callable
    ):
        
        
        try:
            self.req_client = obs.ReqClient()
        except:
            try_launch_obs_if_not_open()
            self.req_client = obs.ReqClient()

        self.event_client = obs.EventClient()
        
        def on_record_state_changed(data):
            if str(data.output_state) == "OBS_WEBSOCKET_OUTPUT_STARTED":
                start_callback(time.perf_counter())
            if str(data.output_state) == "OBS_WEBSOCKET_OUTPUT_STOPPING":
                end_callback(time.perf_counter())

        self.event_client.callback.register(on_record_state_changed)
        

        profile_list = self.req_client.get_profile_list()
        crnt_profile = profile_list.current_profile_name
        self.default_profile = crnt_profile
        if "owl_data_recorder" not in profile_list.profiles:
            self.req_client.create_profile("owl_data_recorder")

        # Init the scene with game
        scene_name_list = self.req_client.get_scene_list().scenes
        scene_name_list = [d['sceneName'] for d in scene_name_list]
        if not "owl_data_collection_scene" in scene_name_list:
            self.req_client.create_scene("owl_data_collection_scene")
        self.req_client.set_current_program_scene("owl_data_collection_scene")

        # Check if a game capture source exists
        inputs = self.req_client.get_input_list().inputs
        input_names = [d['inputName'] for d in inputs]
        if not 'owl_game_capture' in input_names:
            self.req_client.create_input(
                "owl_data_collection_scene",
                "owl_game_capture",
                "game_capture",
                {
                    'capture_mode' : 'any_fullscreen',
                    'capture_audio' : True
                },
                True
            )

        # Audio settings - handle missing sources gracefully
        try:
            self.req_client.set_input_volume('Mic/Aux', None, -100)
        except Exception as e:
            print(f"Warning: Could not set Mic/Aux volume (source may not exist): {e}", file=sys.stderr)
        
        try:
            self.req_client.set_input_volume('Desktop Audio', None, -100)
        except Exception as e:
            print(f"Warning: Could not set Desktop Audio volume (source may not exist): {e}", file=sys.stderr)

        # Init the profile, make sure all params are right
        self.req_client.set_current_profile('owl_data_recorder')
        self.req_client.set_profile_parameter("SimpleOutput", "RecQuality", "Stream")
        self.req_client.set_profile_parameter("SimpleOutput", "VBitrate", str(VIDEO_BITRATE))
        self.req_client.set_profile_parameter("Output", "Mode", "Simple")
        self.req_client.set_profile_parameter("SimpleOutput", "RecFormat2", "mp4")

        # Convert forward slashes to backslashes for Windows
        # Write recording path to debug log
        recording_path = recording_path.replace("\\\\?\\", "")
        self.req_client.set_profile_parameter("SimpleOutput", "FilePath", recording_path)
        
        # Give OBS a moment to process the path change
        time.sleep(0.5)
        
        # Verify the path was set correctly
        try:
            current_path = self.req_client.get_profile_parameter("SimpleOutput", "FilePath").parameter_value
            print(f"OBS confirmed recording path: {current_path}", file=sys.stderr)
        except Exception as e:
            print(f"Warning: Could not verify recording path: {e}", file=sys.stderr)

        # Monitor/resolution info
        ml = self.req_client.get_monitor_list().monitors
        #primary_monitor = next(m for m in ml if m['monitorIndex'] == 0)
        #monitor_height = primary_monitor['monitorHeight']
        #monitor_width = primary_monitor['monitorWidth']
        monitor_width, monitor_height = get_primary_monitor_resolution()
        
        self.req_client.set_video_settings(
            FPS,
            1,
            monitor_width,
            monitor_height,
            RECORDING_WIDTH,
            RECORDING_HEIGHT,
        )
        
        # Find the owl game capture scene id
        item_list = self.req_client.get_scene_item_list("owl_data_collection_scene").scene_items
        owl_gc_id = None
        for item in item_list:
            if item['sourceName'] == 'owl_game_capture':
                owl_gc_id = item['sceneItemId']
                break
        # in a classic pro gamer move, the structure of this transform is not documented
        # outside of the code: <https://github.com/obsproject/obs-websocket/blob/40d26dbf4d29137bf88cd393a3031adb04d68bba/src/requesthandler/RequestHandler_SceneItems.cpp#L399-L440>
        new_tform_dict = {
            'positionX': 0.0,
            'positionY': 0.0,
            'scaleX': 1.0,
            'scaleY': 1.0,
            'rotation': 0.0,
        }
        self.req_client.set_scene_item_transform("owl_data_collection_scene", owl_gc_id, new_tform_dict)
        
        # Conditional encoder settings - only override if SET_ENCODER is True
        if SET_ENCODER:
            print("Setting custom encoder settings", file=sys.stderr)
            self.req_client.set_profile_parameter("SimpleOutput", "StreamEncoder", "x264")
            self.req_client.set_profile_parameter("SimpleOutput", "Preset", "veryfast")
            # Keep commented NVENC code for future use
            #self.req_client.set_profile_parameter("SimpleOutput", "NVENCPreset2", "p7")
        else:
            print("Using user's default encoder settings", file=sys.stderr)
        
        #self.req_client.set_current_profile(self.default_profile)




    def init_configuration(self):
        self.req_client.set_current_profile('owl_data_recorder')

    def reset_configuration(self):
        self.req_client.set_current_profile(self.default_profile)
        

    def start_recording(self):
        #self.init_configuration()
        self.req_client.start_record()

    def stop_recording(self):
        #self.reset_configuration()
        self.req_client.stop_record()