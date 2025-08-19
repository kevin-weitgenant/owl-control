import { ipcRenderer } from 'electron';
import * as fs from 'fs';
import * as path from 'path';
import * as os from 'os';

export interface UploadProgress {
  totalFiles: number;
  uploadedFiles: number;
  currentFile: string;
  bytesUploaded: number;
  totalBytes: number;
  speed: string;
  eta: string;
  isUploading: boolean;
}

export interface UploadStats {
  totalDurationUploaded: number; // in seconds
  totalFilesUploaded: number;
  lastUploadDate: string;
}

export class UploadService {
  private static instance: UploadService;
  private uploadProcess: any = null;
  private statsFilePath: string;

  constructor() {
    // Store stats in temp directory
    this.statsFilePath = path.join(os.tmpdir(), 'owl-control-upload-stats.json');
  }

  public static getInstance(): UploadService {
    if (!UploadService.instance) {
      UploadService.instance = new UploadService();
    }
    return UploadService.instance;
  }

  /**
   * Get upload statistics from temp file
   */
  public getUploadStats(): UploadStats {
    try {
      if (fs.existsSync(this.statsFilePath)) {
        const data = fs.readFileSync(this.statsFilePath, 'utf-8');
        return JSON.parse(data);
      }
    } catch (error) {
      console.error('Error reading upload stats:', error);
    }

    // Return default stats
    return {
      totalDurationUploaded: 0,
      totalFilesUploaded: 0,
      lastUploadDate: 'Never'
    };
  }

  /**
   * Update upload statistics
   */
  private updateUploadStats(additionalDuration: number, additionalFiles: number) {
    try {
      const currentStats = this.getUploadStats();
      const newStats: UploadStats = {
        totalDurationUploaded: currentStats.totalDurationUploaded + additionalDuration,
        totalFilesUploaded: currentStats.totalFilesUploaded + additionalFiles,
        lastUploadDate: new Date().toISOString()
      };

      fs.writeFileSync(this.statsFilePath, JSON.stringify(newStats, null, 2));
    } catch (error) {
      console.error('Error updating upload stats:', error);
    }
  }

  /**
   * Start upload process with progress tracking
   */
  public async startUpload(
    apiToken: string, 
    deleteUploadedFiles: boolean = false,
    progressCallback?: (progress: UploadProgress) => void
  ): Promise<{ success: boolean; message?: string; stats?: UploadStats }> {
    if (this.uploadProcess) {
      return { success: false, message: 'Upload already in progress' };
    }

    try {
      // Start the Python upload process with special flags for progress output
      const result = await ipcRenderer.invoke('start-upload-with-progress', {
        apiToken,
        deleteUploadedFiles,
        progressOutput: true
      });

      if (result.success && result.processId) {
        this.uploadProcess = result.processId;
        
        // Listen for progress updates
        if (progressCallback) {
          this.listenForProgress(progressCallback);
        }

        return { success: true, message: 'Upload started successfully' };
      } else {
        return { success: false, message: result.error || 'Failed to start upload' };
      }
    } catch (error) {
      console.error('Error starting upload:', error);
      return { success: false, message: 'Failed to start upload process' };
    }
  }

  /**
   * Stop current upload process
   */
  public async stopUpload(): Promise<{ success: boolean; message?: string }> {
    if (!this.uploadProcess) {
      return { success: false, message: 'No upload in progress' };
    }

    try {
      await ipcRenderer.invoke('stop-upload-process', this.uploadProcess);
      this.uploadProcess = null;
      return { success: true, message: 'Upload stopped' };
    } catch (error) {
      console.error('Error stopping upload:', error);
      return { success: false, message: 'Failed to stop upload' };
    }
  }

  /**
   * Check if upload is currently in progress
   */
  public isUploading(): boolean {
    return this.uploadProcess !== null;
  }

  /**
   * Listen for progress updates from the upload process
   */
  private listenForProgress(callback: (progress: UploadProgress) => void) {
    // Track progress state across multiple Python progress messages
    let progressState = {
      totalFiles: 0,
      uploadedFiles: 0,
      currentFile: '',
      bytesUploaded: 0,
      totalBytes: 0,
      speed: '0 MB/s',
      eta: 'Calculating...',
      isUploading: true
    };

    // Listen for IPC messages from main process about upload progress
    ipcRenderer.on('upload-progress', (_, progressData) => {
      console.log('Received progress data:', progressData);
      
      // Update progress state based on phase and action
      if (progressData.phase === 'staging') {
        if (progressData.action === 'complete') {
          progressState.totalFiles = progressData.total_files;
        } else if (progressData.action === 'staged') {
          progressState.uploadedFiles = progressData.files_staged;
          progressState.currentFile = progressData.current_file || '';
        } else if (progressData.action === 'processing') {
          progressState.currentFile = progressData.current_file || '';
        }
      } else if (progressData.phase === 'compress') {
        if (progressData.action === 'file') {
          progressState.currentFile = `Compressing ${progressData.current_file}`;
        } else if (progressData.action === 'start') {
          progressState.currentFile = 'Starting compression...';
        } else if (progressData.action === 'complete') {
          progressState.currentFile = 'Compression complete';
        }
      } else if (progressData.phase === 'upload') {
        if (progressData.action === 'start') {
          progressState.currentFile = 'Starting upload...';
        } else if (progressData.action === 'complete') {
          progressState.currentFile = 'Upload complete';
        }
      } else if (progressData.phase === 'finalize') {
        if (progressData.action === 'mark_uploaded') {
          progressState.currentFile = `Finalizing... (${progressData.current}/${progressData.total})`;
        }
      }

      // Send updated progress
      callback({ ...progressState });
    });

    // Listen for upload completion
    ipcRenderer.on('upload-complete', (_, result) => {
      this.uploadProcess = null;
      
      // Update stats if upload was successful
      if (result.success) {
        this.updateUploadStats(result.totalDuration || 0, result.filesUploaded || progressState.totalFiles || 0);
      }

      // Notify completion
      callback({
        totalFiles: progressState.totalFiles || 0,
        uploadedFiles: progressState.totalFiles || 0,
        currentFile: result.success ? 'Upload completed successfully!' : 'Upload failed',
        bytesUploaded: result.totalBytes || 0,
        totalBytes: result.totalBytes || 0,
        speed: '0 MB/s',
        eta: result.success ? 'Complete' : 'Failed',
        isUploading: false
      });
    });
  }

  /**
   * Clean up event listeners
   */
  public cleanup() {
    ipcRenderer.removeAllListeners('upload-progress');
    ipcRenderer.removeAllListeners('upload-complete');
  }
}

export default UploadService;