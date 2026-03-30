import { ref, computed } from 'vue'
import { useScanStore } from '../stores/scanStore'
import { ElMessage } from 'element-plus'

const scanStore = useScanStore()

const scanForm = ref({
  scan_paths: [],
  exclude_paths: [],
  max_file_size: 100 * 1024 * 1024,
  sensitive_types: ['phonenumber', 'idcard', 'name', 'address']
})

const newPath = ref('')
const newExcludePath = ref('')

const canStartScan = computed(() => scanForm.value.scan_paths.length > 0)

const addPath = () => {
  if (newPath.value.trim()) {