<template>
  <div class="virtual-scroll-container" ref="containerRef" @scroll="handleScroll">
    <div class="virtual-scroll-content" :style="{ height: `${totalHeight}px` }">
      <div
        v-for="item in visibleItems"
        :key="item.key"
        class="virtual-scroll-item"
        :style="{
          transform: `translateY(${item.offset}px)`,
          height: `${itemHeight}px`
        }"
      >
        <slot :item="item.data" :index="item.index"></slot>
      </div>
    </div>
    
    <!-- Loading indicator -->
    <div v-if="loading" class="loading-indicator">
      <el-icon class="is-loading"><Loading /></el-icon>
      <span>加载中...</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import { Loading } from '@element-plus/icons-vue'

interface Props {
  items: any[]
  itemHeight: number
  containerHeight: number
  bufferSize?: number
  loading?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  bufferSize: 5,
  loading: false
})

const containerRef = ref<HTMLElement>()
const scrollTop = ref(0)

// Computed properties
const totalHeight = computed(() => props.items.length * props.itemHeight)
const startIndex = computed(() => {
  const start = Math.floor(scrollTop.value / props.itemHeight)
  return Math.max(0, start - props.bufferSize)
})
const endIndex = computed(() => {
  const end = Math.ceil((scrollTop.value + props.containerHeight) / props.itemHeight)
  return Math.min(props.items.length, end + props.bufferSize)
})
const visibleItems = computed(() => {
  return props.items.slice(startIndex.value, endIndex.value).map((item, index) => ({
    key: `${startIndex.value + index}`,
    data: item,
    index: startIndex.value + index,
    offset: (startIndex.value + index) * props.itemHeight
  }))
})

// Event handlers
const handleScroll = () => {
  if (containerRef.value) {
    scrollTop.value = containerRef.value.scrollTop
  }
}

// Scroll to specific index
const scrollToIndex = (index: number) => {
  if (containerRef.value) {
    scrollTop.value = index * props.itemHeight
    containerRef.value.scrollTop = scrollTop.value
  }
}

// Scroll to top
const scrollToTop = () => {
  scrollToIndex(0)
}

// Scroll to bottom
const scrollToBottom = () => {
  scrollToIndex(props.items.length - 1)
}

// Expose methods
defineExpose({
  scrollToIndex,
  scrollToTop,
  scrollToBottom
})

// Watch for items changes
watch(() => props.items.length, () => {
  // Reset scroll position if items change significantly
  if (props.items.length === 0) {
    scrollTop.value = 0
  }
})

// Initialize
onMounted(() => {
  if (containerRef.value) {
    containerRef.value.style.height = `${props.containerHeight}px`
  }
})
</script>

<style scoped lang="css">
.virtual-scroll-container {
  position: relative;
  overflow-y: auto;
  overflow-x: hidden;
}

.virtual-scroll-content {
  position: relative;
  width: 100%;
}

.virtual-scroll-item {
  position: absolute;
  left: 0;
  right: 0;
  box-sizing: border-box;
}

.loading-indicator {
  position: sticky;
  bottom: 0;
  left: 0;
  right: 0;
  padding: 16px;
  background-color: rgba(255, 255, 255, 0.9);
  text-align: center;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  border-top: 1px solid #e4e7ed;
}

.loading-indicator .el-icon {
  font-size: 20px;
}

.loading-indicator span {
  color: #606266;
  font-size: 14px;
}
</style>
