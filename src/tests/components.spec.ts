import { describe, it, expect, beforeEach, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import VirtualScroll from '@/components/VirtualScroll.vue'

describe('VirtualScroll', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('renders correctly with initial props', () => {
    const items = Array.from({ length: 100 }, (_, i) => ({ id: i, name: `Item ${i}` }))
    const wrapper = mount(VirtualScroll, {
      props: {
        items,
        itemHeight: 60,
        containerHeight: 600
      }
    })

    expect(wrapper.exists()).toBe(true)
    expect(wrapper.find('.virtual-scroll-container').exists()).toBe(true)
    expect(wrapper.find('.virtual-scroll-content').exists()).toBe(true)
  })

  it('calculates total height correctly', () => {
    const items = Array.from({ length: 100 }, (_, i) => ({ id: i, name: `Item ${i}` }))
    const wrapper = mount(VirtualScroll, {
      props: {
        items,
        itemHeight: 60,
        containerHeight: 600
      }
    })

    const content = wrapper.find('.virtual-scroll-content')
    expect(content.attributes('style')).toContain('height: 6000px')
  })

  it('renders only visible items', async () => {
    const items = Array.from({ length: 100 }, (_, i) => ({ id: i, name: `Item ${i}` }))
    const wrapper = mount(VirtualScroll, {
      props: {
        items,
        itemHeight: 60,
        containerHeight: 600
      }
    })

    // With 600px height and 60px item height, should show ~10 items
    // With buffer of 5, should show ~20 items
    const visibleItems = wrapper.findAll('.virtual-scroll-item')
    expect(visibleItems.length).toBeGreaterThan(0)
    expect(visibleItems.length).toBeLessThan(30)
  })

  it('scrolls to specific index', async () => {
    const items = Array.from({ length: 100 }, (_, i) => ({ id: i, name: `Item ${i}` }))
    const wrapper = mount(VirtualScroll, {
      props: {
        items,
        itemHeight: 60,
        containerHeight: 600
      }
    })

    const scrollToIndex = (wrapper.vm as any).scrollToIndex
    scrollToIndex(50)

    await wrapper.vm.$nextTick()

    const container = wrapper.find('.virtual-scroll-container')
    expect(container.element.scrollTop).toBeGreaterThan(0)
  })

  it('shows loading indicator when loading', () => {
    const items = Array.from({ length: 100 }, (_, i) => ({ id: i, name: `Item ${i}` }))
    const wrapper = mount(VirtualScroll, {
      props: {
        items,
        itemHeight: 60,
        containerHeight: 600,
        loading: true
      }
    })

    expect(wrapper.find('.loading-indicator').exists()).toBe(true)
  })

  it('hides loading indicator when not loading', () => {
    const items = Array.from({ length: 100 }, (_, i) => ({ id: i, name: `Item ${i}` }))
    const wrapper = mount(VirtualScroll, {
      props: {
        items,
        itemHeight: 60,
        containerHeight: 600,
        loading: false
      }
    })

    expect(wrapper.find('.loading-indicator').exists()).toBe(false)
  })
})

describe('Enhanced Scan Store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('initializes with default state', () => {
    const { useEnhancedScanStore } = require('@/stores/enhancedScanStore')
    const store = useEnhancedScanStore()

    expect(store.scanState).toBe('idle')
    expect(store.isScanning).toBe(false)
    expect(store.isPaused).toBe(false)
    expect(store.filesScanned).toBe(0)
    expect(store.resultsFound).toBe(0)
    expect(store.results).toEqual([])
    expect(store.scanErrors).toEqual([])
  })

  it('computes derived state correctly', () => {
    const { useEnhancedScanStore } = require('@/stores/enhancedScanStore')
    const store = useEnhancedScanStore()

    store.scanState = 'running'
    expect(store.isScanning).toBe(true)
    expect(store.canPause).toBe(true)
    expect(store.canStop).toBe(true)
    expect(store.canStart).toBe(false)

    store.scanState = 'paused'
    expect(store.isPaused).toBe(true)
    expect(store.canResume).toBe(true)
    expect(store.canPause).toBe(false)
  })

  it('starts scan with configuration', async () => {
    const { useEnhancedScanStore } = require('@/stores/enhancedScanStore')
    const store = useEnhancedScanStore()

    const config = {
      scan_paths: ['/test/path'],
      exclude_paths: [],
      max_file_size: 100 * 1024 * 1024,
      sensitive_types: ['phonenumber', 'idcard'],
      thread_count: 4
    }

    await store.startScan(config)

    expect(store.scanState).toBe('running')
    expect(store.scanConfig).toEqual(config)
    expect(store.filesScanned).toBe(0)
    expect(store.resultsFound).toBe(0)
  })

  it('pauses scan', async () => {
    const { useEnhancedScanStore } = require('@/stores/enhancedScanStore')
    const store = useEnhancedScanStore()

    store.scanState = 'running'
    await store.pauseScan()

    expect(store.scanState).toBe('paused')
  })

  it('resumes scan', async () => {
    const { useEnhancedScanStore } = require('@/stores/enhancedScanStore')
    const store = useEnhancedScanStore()

    store.scanState = 'paused'
    await store.resumeScan()

    expect(store.scanState).toBe('running')
  })

  it('stops scan gracefully', async () => {
    const { useEnhancedScanStore } = require('@/stores/enhancedScanStore')
    const store = useEnhancedScanStore()

    store.scanState = 'running'
    await store.stopScan()

    expect(store.scanState).toBe('stopped')
  })

  it('updates progress', () => {
    const { useEnhancedScanStore } = require('@/stores/enhancedScanStore')
    const store = useEnhancedScanStore()

    const progressData = {
      current_file: '/test/file.xlsx',
      files_scanned: 10,
      results_found: 5,
      progress_percentage: 20,
      elapsed_seconds: 30,
      estimated_remaining_seconds: 120,
      scan_speed: 0.33
    }

    store.updateProgress(progressData)

    expect(store.currentFile).toBe('/test/file.xlsx')
    expect(store.filesScanned).toBe(10)
    expect(store.resultsFound).toBe(5)
    expect(store.progressPercentage).toBe(20)
  })

  it('adds scan results', () => {
    const { useEnhancedScanStore } = require('@/stores/enhancedScanStore')
    const store = useEnhancedScanStore()

    const result = {
      id: 'test-id',
      file_path: '/test/file.xlsx',
      sensitive_type: 'phonenumber',
      content: '13800138000',
      masked_content: '138****8000',
      row: 1,
      column: 1,
      found_at: new Date().toISOString()
    }

    store.addResult(result)

    expect(store.results.length).toBe(1)
    expect(store.results[0]).toEqual(result)
  })

  it('adds multiple scan results', () => {
    const { useEnhancedScanStore } = require('@/stores/enhancedScanStore')
    const store = useEnhancedScanStore()

    const results = [
      {
        id: 'test-id-1',
        file_path: '/test/file.xlsx',
        sensitive_type: 'phonenumber',
        content: '13800138000',
        masked_content: '138****8000',
        row: 1,
        column: 1,
        found_at: new Date().toISOString()
      },
      {
        id: 'test-id-2',
        file_path: '/test/file.xlsx',
        sensitive_type: 'idcard',
        content: '110101199003078011',
        masked_content: '1101****8011',
        row: 2,
        column: 1,
        found_at: new Date().toISOString()
      }
    ]

    store.addResults(results)

    expect(store.results.length).toBe(2)
  })

  it('handles scan errors', () => {
    const { useEnhancedScanStore } = require('@/stores/enhancedScanStore')
    const store = useEnhancedScanStore()

    const error = new Error('Test error')
    store.handleScanError('TEST_CATEGORY', error)

    expect(store.scanErrors.length).toBe(1)
    expect(store.scanErrors[0].category).toBe('TEST_CATEGORY')
    expect(store.scanErrors[0].message).toBe('Test error')
    expect(store.hasErrors).toBe(true)
  })

  it('clears scan errors', () => {
    const { useEnhancedScanStore } = require('@/stores/enhancedScanStore')
    const store = useEnhancedScanStore()

    const error = new Error('Test error')
    store.handleScanError('TEST_CATEGORY', error)

    expect(store.scanErrors.length).toBe(1)

    store.clearErrors()

    expect(store.scanErrors.length).toBe(0)
    expect(store.hasErrors).toBe(false)
  })

  it('updates and persists settings', () => {
    const { useEnhancedScanStore } = require('@/stores/enhancedScanStore')
    const store = useEnhancedScanStore()

    const newSettings = {
      max_file_size: 200 * 1024 * 1024,
      auto_mask_results: false,
      log_level: 'debug'
    }

    store.updateSettings(newSettings)

    expect(store.settings.max_file_size).toBe(200 * 1024 * 1024)
    expect(store.settings.auto_mask_results).toBe(false)
    expect(store.settings.log_level).toBe('debug')
  })
})

describe('Scan Store (Legacy)', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('initializes with default state', () => {
    const { useScanStore } = require('@/stores/scanStore')
    const store = useScanStore()

    expect(store.isScanning).toBe(false)
    expect(store.isPaused).toBe(false)
    expect(store.filesScanned).toBe(0)
    expect(store.resultsFound).toBe(0)
    expect(store.results).toEqual([])
  })

  it('starts scan with configuration', () => {
    const { useScanStore } = require('@/stores/scanStore')
    const store = useScanStore()

    const config = {
      scan_paths: ['/test/path'],
      exclude_paths: [],
      max_file_size: 100 * 1024 * 1024,
      sensitive_types: ['phonenumber', 'idcard']
    }

    store.startScan(config)

    expect(store.isScanning).toBe(true)
    expect(store.isPaused).toBe(false)
    expect(store.scanConfig).toEqual(config)
  })

  it('pauses scan', () => {
    const { useScanStore } = require('@/stores/scanStore')
    const store = useScanStore()

    store.pauseScan()

    expect(store.isPaused).toBe(true)
  })

  it('resumes scan', () => {
    const { useScanStore } = require('@/stores/scanStore')
    const store = useScanStore()

    store.resumeScan()

    expect(store.isPaused).toBe(false)
  })

  it('stops scan', () => {
    const { useScanStore } = require('@/stores/scanStore')
    const store = useScanStore()

    store.stopScan()

    expect(store.isScanning).toBe(false)
    expect(store.isPaused).toBe(false)
  })

  it('updates progress', () => {
    const { useScanStore } = require('@/stores/scanStore')
    const store = useScanStore()

    const progressData = {
      current_file: '/test/file.xlsx',
      files_scanned: 10,
      results_found: 5,
      progress_percentage: 20,
      elapsed_seconds: 30,
      estimated_remaining_seconds: 120,
      scan_speed: 0.33
    }

    store.updateProgress(progressData)

    expect(store.currentFile).toBe('/test/file.xlsx')
    expect(store.filesScanned).toBe(10)
    expect(store.resultsFound).toBe(5)
  })

  it('adds scan results', () => {
    const { useScanStore } = require('@/stores/scanStore')
    const store = useScanStore()

    const result = {
      id: 'test-id',
      file_path: '/test/file.xlsx',
      sensitive_type: 'phonenumber',
      content: '13800138000',
      masked_content: '138****8000',
      row: 1,
      column: 1,
      found_at: new Date().toISOString()
    }

    store.addResult(result)

    expect(store.results.length).toBe(1)
    expect(store.results[0]).toEqual(result)
  })

  it('clears scan results', () => {
    const { useScanStore } = require('@/stores/scanStore')
    const store = useScanStore()

    const result = {
      id: 'test-id',
      file_path: '/test/file.xlsx',
      sensitive_type: 'phonenumber',
      content: '13800138000',
      masked_content: '138****8000',
      row: 1,
      column: 1,
      found_at: new Date().toISOString()
    }

    store.addResult(result)
    expect(store.results.length).toBe(1)

    store.clearResults()
    expect(store.results.length).toBe(0)
  })

  it('computes scan statistics', () => {
    const { useScanStore } = require('@/stores/scanStore')
    const store = useScanStore()

    store.filesScanned = 10
    store.resultsFound = 5
    store.elapsedSeconds = 30
    store.estimatedRemaining = 120
    store.scanSpeed = 0.33

    expect(store.scanStats.filesScanned).toBe(10)
    expect(store.scanStats.resultsFound).toBe(5)
    expect(store.scanStats.elapsedSeconds).toBe(30)
    expect(store.scanStats.estimatedRemaining).toBe(120)
    expect(store.scanStats.scanSpeed).toBe('0.33')
  })
})
