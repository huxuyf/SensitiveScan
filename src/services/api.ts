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
   * 启动全新的扫描任务
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
      console.error('启动扫描失败:', error)
      throw error
    }
  }

  /**
   * 暂停正在进行的扫描
   */
  static async pauseScan(): Promise<void> {
    try {
      await invoke('pause_scan')
    } catch (error) {
      console.error('暂停扫描失败:', error)
      throw error
    }
  }

  /**
   * 继续恢复扫描过程
   */
  static async resumeScan(): Promise<void> {
    try {
      await invoke('resume_scan')
    } catch (error) {
      console.error('恢复扫描失败:', error)
      throw error
    }
  }

  /**
   * 强行中断并停止扫描任务
   */
  static async stopScan(): Promise<void> {
    try {
      await invoke('stop_scan')
    } catch (error) {
      console.error('停止扫描任务失败:', error)
      throw error
    }
  }

  /**
   * 请求后端提供归总后的敏感结果
   */
  static async getAggregatedResults(threshold?: number): Promise<any[]> {
    try {
      const result = await invoke('get_aggregated_results', {
        threshold
      })
      return JSON.parse(result as string)
    } catch (error) {
      console.error('获取结果列表数据失败:', error)
      throw error
    }
  }

  /**
   * 通知系统以此路径打开文件
   */
  static async openFile(path: string): Promise<void> {
    try {
      await invoke('open_file', { path })
    } catch (error) {
      console.error('调起原生应用打开文件失败:', error)
      throw error
    }
  }

  /**
   * 发出指令清理此文件相关磁盘和数据库内容
   */
  static async deleteFile(path: string): Promise<void> {
    try {
      await invoke('delete_file', { path })
    } catch (error) {
      console.error('删除指定文件失败:', error)
      throw error
    }
  }

  /**
   * 下载或导出当前的判定结果
   */
  static async exportResults(format: string, filePath: string): Promise<void> {
    try {
      await invoke('export_results', {
        format,
        filePath
      })
    } catch (error) {
      console.error('保存导出数据失败:', error)
      throw error
    }
  }

  /**
   * 获取概况统计数字
   */
  static async getScanStats(): Promise<any> {
    try {
      const result = await invoke('get_scan_stats')
      return JSON.parse(result as string)
    } catch (error) {
      console.error('读取全库统计信息失败:', error)
      throw error
    }
  }

  /**
   * 订阅获取后端进度同步事件
   */
  static async onScanProgress(callback: (progress: ScanProgress) => void): Promise<() => void> {
    try {
      const unlisten = await listen('scan-progress', (event) => {
        callback(event.payload as ScanProgress)
      })
      return unlisten
    } catch (error) {
      console.error('注册进度事件监听回调发生异常:', error)
      throw error
    }
  }

  /**
   * 订阅后端底层返回的结果事件
   */
  static async onScanResult(callback: (result: ScanResult) => void): Promise<() => void> {
    try {
      const unlisten = await listen('scan-result', (event) => {
        callback(event.payload as ScanResult)
      })
      return unlisten
    } catch (error) {
      console.error('注册单结果回调通道失败:', error)
      throw error
    }
  }

  /**
   * 监听完毕事件通知
   */
  static async onScanComplete(callback: (stats: any) => void): Promise<() => void> {
    try {
      const unlisten = await listen('scan-complete', (event) => {
        callback(event.payload)
      })
      return unlisten
    } catch (error) {
      console.error('注册扫描任务全域完成事件监听失败:', error)
      throw error
    }
  }
}

export const API = TauriAPI
