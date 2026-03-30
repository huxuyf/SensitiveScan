import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

export interface ScanConfig {
  scan_paths: string[]
  exclude_paths: string[]
  max_file_size: number
  sensitive_types: string[]
}

export interface ScanResult {
  id: string
  file_path: string
  sheet_name?: string
  row: number
  column: number
  sensitive_type: string
  content: string
  masked_content: string
  found_at: string
}

export interface ScanProgress {
  current_file: string
  files_scanned: number
  results_found: number
  progress_percentage: number
  elapsed_seconds: number
  estimated_remaining_seconds: number
  scan_speed: number
}

export class TauriAPI {
  /**
   * Start a new scan task
   */
  static async startScan(config: ScanConfig): Promise<string> {
    try {
      const result = await invoke('start_scan', {
        scan_paths: config.scan_paths,
        exclude_paths: config.exclude_paths,
        max_file_size: config.max_file_size,
        sensitive_types: config.sensitive_types
      })
      return JSON.parse(result as string)
    } catch (error) {
      console.error('Failed to start scan:', error)
      throw error
    }
  }

  /**
   * Pause current scan
   */
  static async pauseScan(): Promise<void> {
    try {
      await invoke('pause_scan')
    } catch (error) {
      console.error('Failed to pause scan:', error)
      throw error
    }
  }

  /**
   * Resume paused scan
   */
  static async resumeScan(): Promise<void> {
    try {
      await invoke('resume_scan')
    } catch (error) {
      console.error('Failed to resume scan:', error)
      throw error
    }
  }

  /**
   * Stop current scan
   */
  static async stopScan(): Promise<void> {
    try {
      await invoke('stop_scan')
    } catch (error) {
      console.error('Failed to stop scan:', error)
      throw error
    }
  }

  /**
   * Get scan results
   */
  static async getScanResults(
    limit?: number,
    offset?: number,
    file_path_filter?: string,
    sensitive_type_filter?: string
  ): Promise<ScanResult[]> {
    try {
      const result = await invoke('get_scan_results', {
        limit,
        offset,
        file_path_filter,
        sensitive_type_filter
      })
      return JSON.parse(result as string)
    } catch (error) {
      console.error('Failed to get scan results:', error)
      throw error
    }
  }

  /**
   * Export scan results
   */
  static async exportResults(format: string, file_path: string): Promise<void> {
    try {
      await invoke('export_results', {
        format,
        file_path
      })
    } catch (error) {
      console.error('Failed to export results:', error)
      throw error
    }
  }

  /**
   * Get scan history
   */
  static async getHistory(limit?: number): Promise<any[]> {
    try {
      const result = await invoke('get_history', { limit })
      return JSON.parse(result as string)
    } catch (error) {
      console.error('Failed to get history:', error)
      throw error
    }
  }

  /**
   * Delete scan history
   */
  static async deleteHistory(history_id: string): Promise<void> {
    try {
      await invoke('delete_history', { history_id })
    } catch (error) {
      console.error('Failed to delete history:', error)
      throw error
    }
  }

  /**
   * Add whitelist entry
   */
  static async addWhitelist(
    content: string,
    sensitive_type: string,
    description?: string
  ): Promise<any> {
    try {
      const result = await invoke('add_whitelist', {
        content,
        sensitive_type,
        description
      })
      return JSON.parse(result as string)
    } catch (error) {
      console.error('Failed to add whitelist:', error)
      throw error
    }
  }

  /**
   * Get whitelist
   */
  static async getWhitelist(): Promise<any[]> {
    try {
      const result = await invoke('get_whitelist')
      return JSON.parse(result as string)
    } catch (error) {
      console.error('Failed to get whitelist:', error)
      throw error
    }
  }

  /**
   * Delete whitelist entry
   */
  static async deleteWhitelist(entry_id: string): Promise<void> {
    try {
      await invoke('delete_whitelist', { entry_id })
    } catch (error) {
      console.error('Failed to delete whitelist:', error)
      throw error
    }
  }

  /**
   * Get scan statistics
   */
  static async getScanStats(): Promise<any> {
    try {
      const result = await invoke('get_scan_stats')
      return JSON.parse(result as string)
    } catch (error) {
      console.error('Failed to get scan stats:', error)
      throw error
    }
  }

  /**
   * Listen to scan progress events
   */
  static async onScanProgress(callback: (progress: ScanProgress) => void): Promise<() => void> {
    try {
      const unlisten = await listen('scan-progress', (event) => {
        callback(event.payload as ScanProgress)
      })
      return unlisten
    } catch (error) {
      console.error('Failed to listen to scan progress:', error)
      throw error
    }
  }

  /**
   * Listen to scan result events
   */
  static async onScanResult(callback: (result: ScanResult) => void): Promise<() => void> {
    try {
      const unlisten = await listen('scan-result', (event) => {
        callback(event.payload as ScanResult)
      })
      return unlisten
    } catch (error) {
      console.error('Failed to listen to scan result:', error)
      throw error
    }
  }

  /**
   * Listen to scan completion events
   */
  static async onScanComplete(callback: (stats: any) => void): Promise<() => void> {
    try {
      const unlisten = await listen('scan-complete', (event) => {
        callback(event.payload)
      })
      return unlisten
    } catch (error) {
      console.error('Failed to listen to scan complete:', error)
      throw error
    }
  }
}
