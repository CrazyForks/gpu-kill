<template>
  <div class="min-h-screen bg-black flex">
    <!-- Left Sidebar -->
    <AppSidebar :current-tab="activeTab" @tab-change="setActiveTab" />

      <!-- Main Content Area -->
      <div class="flex-1 flex flex-col bg-gray-500/10 mr-2 my-2 rounded-xl border border-gray-500/10 h-screen overflow-hidden">
        <!-- Top Header -->
        <header class="border-b border-gray-500/10 flex-shrink-0">
          <div class="px-6 py-4">
            <div class="flex justify-between items-center">
              <div class="flex items-center space-x-2">
                <h2 class="text-base font-medium text-white">Overview</h2>
                <p class="text-sm text-gray-500">Real-time GPU cluster monitoring</p>
              </div>
              <div class="flex items-center space-x-4">
                <button @click="refreshData" :disabled="isRefreshing" class="border border-gray-500/10 text-sm rounded-xl px-4 py-2 text-white bg-gray-500/10 hover:bg-gray-500/15 flex items-center space-x-2 disabled:opacity-50 disabled:cursor-not-allowed">
                  <ArrowPathIcon :class="['w-4 h-4', isRefreshing ? 'animate-spin' : '']" />
                  <span>{{ isRefreshing ? 'Refreshing...' : 'Refresh' }}</span>
                </button>
              </div>
            </div>
          </div>
        </header>

        <!-- Main Content -->
        <main class="flex-1 overflow-y-auto divide-y divide-gray-500/10">
          <!-- Cluster Overview -->
          <div class="p-6">
            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
              <div class="bg-gray-500/5 rounded-xl border border-gray-500/10 p-6 transition-all duration-200">
                <div class="flex items-center justify-between">
                  <div>
                    <p class="text-sm font-medium text-gray-400 mb-1">Total Nodes</p>
                    <p class="text-3xl font-bold text-white">{{ clusterData?.nodes?.length || 0 }}</p>
                    <p class="text-xs text-gray-500 mt-1">Active servers</p>
                  </div>
                  <div class="w-12 h-12 bg-blue-300/10 rounded-xl flex items-center justify-center">
                    <ServerIcon class="w-6 h-6 text-blue-300" />
                  </div>
                </div>
              </div>
              
              <div class="bg-gray-500/5 rounded-xl border border-gray-500/10 p-6 transition-all duration-200">
                <div class="flex items-center justify-between">
                  <div>
                    <p class="text-sm font-medium text-gray-400 mb-1">Total GPUs</p>
                    <p class="text-3xl font-bold text-white">{{ clusterData?.total_gpus || 0 }}</p>
                    <p class="text-xs text-gray-500 mt-1">Available devices</p>
                  </div>
                  <div class="w-12 h-12 bg-gpu-400/10 rounded-xl flex items-center justify-center">
                    <CpuChipIcon class="w-6 h-6 text-gpu-400" />
                  </div>
                </div>
              </div>
              
              <div class="bg-gray-500/5 rounded-xl border border-gray-500/10 p-6 transition-all duration-200">
                <div class="flex items-center justify-between">
                  <div>
                    <p class="text-sm font-medium text-gray-400 mb-1">Total Memory</p>
                    <p class="text-3xl font-bold text-white">{{ formatMemory(clusterData?.total_memory_gb || 0) }}</p>
                    <p class="text-xs text-gray-500 mt-1">VRAM capacity</p>
                  </div>
                  <div class="w-12 h-12 bg-green-400/10 rounded-xl flex items-center justify-center">
                    <CircleStackIcon class="w-6 h-6 text-green-400" />
                  </div>
                </div>
              </div>
              
              <div class="bg-gray-500/5 rounded-xl border border-gray-500/10 p-6 transition-all duration-200">
                <div class="flex items-center justify-between">
                  <div>
                    <p class="text-sm font-medium text-gray-400 mb-1">Avg Utilization</p>
                    <p class="text-3xl font-bold text-white">{{ Math.round(clusterData?.utilization_avg || 0) }}%</p>
                    <p class="text-xs text-gray-500 mt-1">GPU usage</p>
                  </div>
                  <div class="w-12 h-12 bg-orange-400/10 rounded-xl flex items-center justify-center">
                    <ChartBarIcon class="w-6 h-6 text-orange-400" />
                  </div>
                </div>
              </div>
            </div>
          </div>

          <!-- Node Details -->
          <div class="p-6">
            <div class="flex items-center space-x-3 mb-4">
              <h2 class="text-xs font-medium text-gray-500 uppercase">Node Details</h2>
            </div>
            <!-- empty state -->
            <div v-if="clusterData?.nodes?.length === 0" class="text-center py-8">
              <p class="text-gray-400">No nodes found</p>
            </div>
            <div v-else class="grid grid-cols-1 lg:grid-cols-2 xl:grid-cols-3 gap-6">
              <div 
                v-for="node in clusterData?.nodes" 
                :key="node.node_id"
                class="bg-gray-500/5 rounded-xl border border-gray-500/10 p-4"
              >
                <div class="flex items-center justify-between mb-4">
                  <h3 class="text-lg font-medium text-white">
                    {{ node.hostname }}
                  </h3>
                  <span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-green-400/10 text-green-400">
                    {{ node.status }}
                  </span>
                </div>
                
                <div class="space-y-3">
                  <div class="flex justify-between text-sm">
                    <span class="text-gray-400">GPUs:</span>
                    <span class="font-medium text-white">{{ node.gpus?.length || 0 }}</span>
                  </div>
                  <div class="flex justify-between text-sm">
                    <span class="text-gray-400">Processes:</span>
                    <span class="font-medium text-white">{{ node.processes?.length || 0 }}</span>
                  </div>
                  <div class="flex justify-between text-sm">
                    <span class="text-gray-400">Last Seen:</span>
                    <span class="font-medium text-white">{{ formatTime(node.timestamp) }}</span>
                  </div>
                </div>

                <div v-if="node.gpus?.length > 0" class="mt-4 space-y-2">
                  <h4 class="text-sm font-medium text-white">GPUs:</h4>
                  <div 
                    v-for="gpu in node.gpus" 
                    :key="gpu.gpu_index"
                    class="rounded-xl border border-gray-500/10 p-3 bg-gray-500/5"
                  >
                    <div class="flex justify-between items-start mb-2">
                      <div>
                        <p class="font-medium text-white">{{ gpu.name }}</p>
                        <p class="text-sm text-gray-500">
                          {{ formatMemory(gpu.mem_used_mb / 1024) }} / {{ formatMemory(gpu.mem_total_mb / 1024) }}
                        </p>
                      </div>
                      <span class="text-sm font-medium text-white">
                        {{ Math.round(gpu.util_pct) }}%
                      </span>
                    </div>
                    <div class="w-full bg-gray-500/10 rounded-full h-2">
                      <div 
                        class="h-2 rounded-full transition-all duration-300"
                        :class="{
                          'bg-green-400': gpu.util_pct < 50,
                          'bg-yellow-400': gpu.util_pct >= 50 && gpu.util_pct < 80,
                          'bg-red-400': gpu.util_pct >= 80
                        }"
                        :style="{ width: `${gpu.util_pct}%` }"
                      ></div>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>

          <!-- Magic Moment - Contention Analysis -->
          <div class="p-6">
            <div class="flex items-center space-x-3 mb-6">
              <h2 class="text-xs font-medium text-gray-500 uppercase">GPU Contention</h2>
            </div>
            <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
              <!-- Blocked GPUs -->
              <div class="rounded-xl border border-gray-500/10 p-6 bg-gray-500/5">
                <div class="flex items-center space-x-3 mb-4">
                  <div class="w-7 h-7 bg-red-500/10 rounded-lg flex items-center justify-center">
                    <ExclamationTriangleIcon class="w-4 h-4 text-red-400" />
                  </div>
                  <h3 class="text-lg font-semibold text-white">Blocked GPUs</h3>
                </div>
                <!-- empty state -->
                <div v-if="contentionData?.blocked_gpus?.length === 0" class="text-center py-8">
                  <div class="w-16 h-16 bg-gray-500/10 rounded-full flex items-center justify-center mx-auto mb-4">
                    <CheckCircleIcon class="w-8 h-8 text-gray-500" />
                  </div>
                  <p class="text-gray-400">No blocked GPUs! All systems running smoothly.</p>
                </div>
                <div v-else class="space-y-3">
                  <div 
                    v-for="gpu in contentionData?.blocked_gpus" 
                    :key="`${gpu.node_id}-${gpu.gpu_index}`"
                    class="rounded-xl border border-gray-500/10 p-3 bg-gray-500/5"
                  >
                    <div class="flex justify-between items-start">
                      <div>
                        <p class="font-medium text-white">
                          {{ gpu.gpu_name }} ({{ gpu.node_id }})
                        </p>
                        <p class="text-sm text-gray-400">
                          {{ gpu.utilization_pct }}% utilized, {{ formatMemory(gpu.memory_used_mb / 1024) }} used
                        </p>
                      </div>
                      <span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-red-400/10 text-red-400">
                        Blocked
                      </span>
                    </div>
                    <div class="mt-2">
                      <div class="w-full bg-gray-500/10 rounded-full h-2">
                        <div 
                          class="h-2 rounded-full bg-red-400 transition-all duration-300" 
                          :style="{ width: `${gpu.utilization_pct}%` }"
                        ></div>
                      </div>
                    </div>
                  </div>
                </div>
              </div>

              <!-- Top Users -->
              <div class="rounded-xl border border-gray-500/10 p-6 bg-gray-500/5">
                <div class="flex items-center space-x-3 mb-4">
                  <div class="w-7 h-7 bg-blue-400/10 rounded-lg flex items-center justify-center">
                    <UsersIcon class="w-4 h-4 text-blue-400" />
                  </div>
                  <h3 class="text-lg font-semibold text-white">Top Users</h3>
                </div>
                <!-- empty state -->
                <div v-if="contentionData?.top_users?.length === 0" class="text-center py-8">
                  <div class="w-16 h-16 bg-gray-500/10 rounded-full flex items-center justify-center mx-auto mb-4">
                    <UsersIcon class="w-8 h-8 text-gray-500" />
                  </div>
                  <p class="text-gray-400">No active users detected</p>
                </div>
                <div v-else class="space-y-3">
                  <div 
                    v-for="(user, index) in contentionData?.top_users" 
                    :key="user.user"
                    class="flex items-center justify-between p-3 border border-gray-500/10 bg-gray-500/5 rounded-xl"
                  >
                    <div class="flex items-center">
                      <div class="flex-shrink-0 w-8 h-8 bg-blue-300/10 rounded-full flex items-center justify-center text-white text-sm font-medium">
                        {{ index + 1 }}
                      </div>
                      <div class="ml-3">
                        <p class="font-medium text-white">{{ user.user }}</p>
                        <p class="text-sm text-gray-400">
                          {{ user.gpu_count }} GPUs, {{ formatMemory(user.total_memory_mb / 1024) }}
                        </p>
                      </div>
                    </div>
                    <div class="text-right">
                      <p class="text-sm font-medium text-white">
                        {{ Math.round(user.avg_utilization) }}%
                      </p>
                      <p class="text-xs text-gray-400">avg util</p>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </main>
      </div>
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted } from 'vue'
import {
  CpuChipIcon,
  ServerIcon,
  CircleStackIcon,
  ChartBarIcon,
  ArrowPathIcon,
  BoltIcon,
  ExclamationTriangleIcon,
  CheckCircleIcon,
  UsersIcon
} from '@heroicons/vue/24/solid'

// Page metadata
useHead({
  title: 'Overview',
})

const config = useRuntimeConfig()
const clusterData = ref(null)
const contentionData = ref(null)
const isDark = ref(false)
const activeTab = ref('overview')
const isRefreshing = ref(false)
let refreshInterval = null

const formatMemory = (gb) => {
  if (gb >= 1024) {
    return `${(gb / 1024).toFixed(1)} TB`
  }
  return `${gb.toFixed(1)} GB`
}

const formatTime = (timestamp) => {
  return new Date(timestamp).toLocaleTimeString()
}

const toggleDarkMode = () => {
  isDark.value = !isDark.value
  if (process.client) {
    document.documentElement.classList.toggle('dark')
  }
}

const fetchClusterData = async () => {
  try {
    const response = await fetch(`${config.public.apiBase}/api/cluster/snapshot`)
    if (response.ok) {
      clusterData.value = await response.json()
      console.log('Cluster data:', clusterData.value)
      return
    }
  } catch (error) {
    console.log('API not available, using sample data:', error)
  }
  
  // Fallback to sample data if API is not available
  clusterData.value = {
    nodes: [
      {
        node_id: "node-001",
        hostname: "gpu-server-01",
        status: "online",
        timestamp: new Date().toISOString(),
        gpus: [
          {
            gpu_index: 0,
            name: "NVIDIA RTX 4090",
            mem_used_mb: 8192,
            mem_total_mb: 24576,
            util_pct: 75
          },
          {
            gpu_index: 1,
            name: "NVIDIA RTX 4090",
            mem_used_mb: 12288,
            mem_total_mb: 24576,
            util_pct: 45
          }
        ],
        processes: [
          { pid: 12345, user: "alice", proc_name: "training_job.py", gpu_index: 0 },
          { pid: 67890, user: "bob", proc_name: "inference.py", gpu_index: 1 }
        ]
      },
      {
        node_id: "node-002",
        hostname: "gpu-server-02",
        status: "online",
        timestamp: new Date().toISOString(),
        gpus: [
          {
            gpu_index: 0,
            name: "NVIDIA A100",
            mem_used_mb: 32768,
            mem_total_mb: 40960,
            util_pct: 85
          },
          {
            gpu_index: 1,
            name: "NVIDIA A100",
            mem_used_mb: 20480,
            mem_total_mb: 40960,
            util_pct: 60
          },
          {
            gpu_index: 2,
            name: "NVIDIA A100",
            mem_used_mb: 10240,
            mem_total_mb: 40960,
            util_pct: 30
          }
        ],
        processes: [
          { pid: 11111, user: "charlie", proc_name: "model_training.py", gpu_index: 0 },
          { pid: 22222, user: "diana", proc_name: "data_processing.py", gpu_index: 1 },
          { pid: 33333, user: "eve", proc_name: "experiment.py", gpu_index: 2 }
        ]
      },
      {
        node_id: "node-003",
        hostname: "gpu-server-03",
        status: "online",
        timestamp: new Date().toISOString(),
        gpus: [
          {
            gpu_index: 0,
            name: "NVIDIA RTX 3090",
            mem_used_mb: 18432,
            mem_total_mb: 24576,
            util_pct: 90
          }
        ],
        processes: [
          { pid: 44444, user: "frank", proc_name: "deep_learning.py", gpu_index: 0 }
        ]
      }
    ],
    total_gpus: 6,
    total_memory_gb: 200.0,
    utilization_avg: 64
  }
  console.log('Using sample cluster data:', clusterData.value)
}

const fetchContentionData = async () => {
  try {
    const response = await fetch(`${config.public.apiBase}/api/cluster/contention`)
    if (response.ok) {
      contentionData.value = await response.json()
      console.log('Contention data:', contentionData.value)
      return
    }
  } catch (error) {
    console.log('API not available, using sample contention data:', error)
  }
  
  // Fallback to sample data if API is not available
  contentionData.value = {
    blocked_gpus: [
      {
        node_id: "node-003",
        gpu_index: 0,
        gpu_name: "NVIDIA RTX 3090",
        utilization_pct: 90,
        memory_used_mb: 18432,
        blocking_processes: [
          { user: "frank", proc_name: "deep_learning.py", pid: 44444 }
        ]
      }
    ],
    top_users: [
      {
        user: "charlie",
        gpu_count: 1,
        total_memory_mb: 32768,
        avg_utilization: 85,
        processes: ["model_training.py"]
      },
      {
        user: "frank",
        gpu_count: 1,
        total_memory_mb: 18432,
        avg_utilization: 90,
        processes: ["deep_learning.py"]
      },
      {
        user: "diana",
        gpu_count: 1,
        total_memory_mb: 20480,
        avg_utilization: 60,
        processes: ["data_processing.py"]
      },
      {
        user: "alice",
        gpu_count: 1,
        total_memory_mb: 8192,
        avg_utilization: 75,
        processes: ["training_job.py"]
      },
      {
        user: "eve",
        gpu_count: 1,
        total_memory_mb: 10240,
        avg_utilization: 30,
        processes: ["experiment.py"]
      }
    ]
  }
  console.log('Using sample contention data:', contentionData.value)
}


const setActiveTab = (tab) => {
  activeTab.value = tab
}

const refreshData = async () => {
  if (isRefreshing.value) return
  
  isRefreshing.value = true
  try {
    // Add minimum loading time so users can see the loading state
    const [clusterResult, contentionResult] = await Promise.all([
      fetchClusterData(),
      fetchContentionData()
    ])
    
    // Ensure minimum loading time of 1 second
    await new Promise(resolve => setTimeout(resolve, 1000))
  } finally {
    isRefreshing.value = false
  }
}

onMounted(() => {
  // // Fix overscroll background issue
  // document.documentElement.style.background = '#000000'
  // document.documentElement.style.backgroundColor = '#000000'
  // document.body.style.background = '#000000'
  // document.body.style.backgroundColor = '#000000'
  
  // // Disable overscroll behavior
  // document.documentElement.style.overscrollBehavior = 'none'
  // document.body.style.overscrollBehavior = 'none'
  
  // // Apply to all elements
  // const allElements = document.querySelectorAll('*')
  // allElements.forEach(el => {
  //   el.style.overscrollBehavior = 'none'
  // })
  
  console.log('Dashboard mounted, fetching data...')
  refreshData()
  
  // Auto-refresh every 10 seconds
  refreshInterval = setInterval(refreshData, 10000)
})

onUnmounted(() => {
  if (refreshInterval) {
    clearInterval(refreshInterval)
  }
})
</script>