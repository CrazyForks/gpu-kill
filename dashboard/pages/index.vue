<template>
  <div class="min-h-screen">
    <!-- Header -->
    <header class="bg-white dark:bg-gray-800 shadow-sm border-b border-gray-200 dark:border-gray-700">
      <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div class="flex justify-between items-center h-16">
          <div class="flex items-center">
            <div class="flex-shrink-0">
              <h1 class="text-2xl font-bold text-gray-900 dark:text-white">
                🚀 GPU Kill Dashboard
              </h1>
            </div>
          </div>
          <div class="flex items-center space-x-4">
            <div class="flex items-center space-x-2">
              <div class="w-2 h-2 bg-green-500 rounded-full animate-pulse"></div>
              <span class="text-sm text-gray-600 dark:text-gray-400">Live</span>
            </div>
            <button
              @click="toggleDarkMode"
              class="p-2 rounded-md text-gray-400 hover:text-gray-500 hover:bg-gray-100 dark:hover:bg-gray-700"
            >
              <SunIcon v-if="isDark" class="h-5 w-5" />
              <MoonIcon v-else class="h-5 w-5" />
            </button>
          </div>
        </div>
      </div>
    </header>

    <!-- Main Content -->
    <main class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
      <!-- Cluster Overview -->
      <div class="mb-8">
        <h2 class="text-xl font-semibold text-gray-900 dark:text-white mb-4">
          📊 Cluster Overview
        </h2>
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
          <div class="metric-card">
            <div class="flex items-center">
              <div class="flex-shrink-0">
                <ServerIcon class="h-8 w-8 text-blue-600 dark:text-blue-400" />
              </div>
              <div class="ml-4">
                <p class="text-sm font-medium text-gray-600 dark:text-gray-400">Total Nodes</p>
                <p class="text-2xl font-bold text-gray-900 dark:text-white">
                  {{ clusterSnapshot?.nodes?.length || 0 }}
                </p>
              </div>
            </div>
          </div>
          
          <div class="metric-card">
            <div class="flex items-center">
              <div class="flex-shrink-0">
                <CpuChipIcon class="h-8 w-8 text-green-600 dark:text-green-400" />
              </div>
              <div class="ml-4">
                <p class="text-sm font-medium text-gray-600 dark:text-gray-400">Total GPUs</p>
                <p class="text-2xl font-bold text-gray-900 dark:text-white">
                  {{ clusterSnapshot?.total_gpus || 0 }}
                </p>
              </div>
            </div>
          </div>
          
          <div class="metric-card">
            <div class="flex items-center">
              <div class="flex-shrink-0">
                <MemoryIcon class="h-8 w-8 text-purple-600 dark:text-purple-400" />
              </div>
              <div class="ml-4">
                <p class="text-sm font-medium text-gray-600 dark:text-gray-400">Total Memory</p>
                <p class="text-2xl font-bold text-gray-900 dark:text-white">
                  {{ formatMemory(clusterSnapshot?.total_memory_gb || 0) }}
                </p>
              </div>
            </div>
          </div>
          
          <div class="metric-card">
            <div class="flex items-center">
              <div class="flex-shrink-0">
                <ChartBarIcon class="h-8 w-8 text-orange-600 dark:text-orange-400" />
              </div>
              <div class="ml-4">
                <p class="text-sm font-medium text-gray-600 dark:text-gray-400">Avg Utilization</p>
                <p class="text-2xl font-bold text-gray-900 dark:text-white">
                  {{ Math.round(clusterSnapshot?.utilization_avg || 0) }}%
                </p>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Magic Moment - Contention Analysis -->
      <div class="mb-8" v-if="contentionAnalysis">
        <h2 class="text-xl font-semibold text-gray-900 dark:text-white mb-4">
          ⚡ Magic Moment - GPU Contention
        </h2>
        <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
          <!-- Blocked GPUs -->
          <div class="gpu-card">
            <h3 class="text-lg font-medium text-gray-900 dark:text-white mb-4">
              🚫 Blocked GPUs
            </h3>
            <div v-if="contentionAnalysis.blocked_gpus.length === 0" class="text-center py-8">
              <CheckCircleIcon class="h-12 w-12 text-green-500 mx-auto mb-2" />
              <p class="text-gray-600 dark:text-gray-400">No blocked GPUs! 🎉</p>
            </div>
            <div v-else class="space-y-3">
              <div
                v-for="gpu in contentionAnalysis.blocked_gpus"
                :key="`${gpu.node_id}-${gpu.gpu_index}`"
                class="border border-red-200 dark:border-red-800 rounded-lg p-3 bg-red-50 dark:bg-red-900/20"
              >
                <div class="flex justify-between items-start">
                  <div>
                    <p class="font-medium text-gray-900 dark:text-white">
                      {{ gpu.gpu_name }} ({{ gpu.node_id }})
                    </p>
                    <p class="text-sm text-gray-600 dark:text-gray-400">
                      {{ gpu.utilization_pct }}% utilized, {{ formatMemory(gpu.memory_used_mb / 1024) }} used
                    </p>
                  </div>
                  <span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200">
                    Blocked
                  </span>
                </div>
                <div class="mt-2">
                  <div class="utilization-bar">
                    <div
                      class="utilization-fill utilization-high"
                      :style="{ width: `${gpu.utilization_pct}%` }"
                    ></div>
                  </div>
                </div>
              </div>
            </div>
          </div>

          <!-- Top Users -->
          <div class="gpu-card">
            <h3 class="text-lg font-medium text-gray-900 dark:text-white mb-4">
              👥 Top Users
            </h3>
            <div v-if="contentionAnalysis.top_users.length === 0" class="text-center py-8">
              <UserGroupIcon class="h-12 w-12 text-gray-400 mx-auto mb-2" />
              <p class="text-gray-600 dark:text-gray-400">No active users</p>
            </div>
            <div v-else class="space-y-3">
              <div
                v-for="(user, index) in contentionAnalysis.top_users"
                :key="user.user"
                class="flex items-center justify-between p-3 bg-gray-50 dark:bg-gray-700 rounded-lg"
              >
                <div class="flex items-center">
                  <div class="flex-shrink-0 w-8 h-8 bg-blue-500 rounded-full flex items-center justify-center text-white text-sm font-medium">
                    {{ index + 1 }}
                  </div>
                  <div class="ml-3">
                    <p class="font-medium text-gray-900 dark:text-white">{{ user.user }}</p>
                    <p class="text-sm text-gray-600 dark:text-gray-400">
                      {{ user.gpu_count }} GPUs, {{ formatMemory(user.total_memory_mb / 1024) }}
                    </p>
                  </div>
                </div>
                <div class="text-right">
                  <p class="text-sm font-medium text-gray-900 dark:text-white">
                    {{ Math.round(user.avg_utilization) }}%
                  </p>
                  <p class="text-xs text-gray-600 dark:text-gray-400">avg util</p>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Node Details -->
      <div class="mb-8">
        <h2 class="text-xl font-semibold text-gray-900 dark:text-white mb-4">
          🖥️ Node Details
        </h2>
        <div class="grid grid-cols-1 lg:grid-cols-2 xl:grid-cols-3 gap-6">
          <div
            v-for="node in clusterSnapshot?.nodes"
            :key="node.node_id"
            class="gpu-card"
          >
            <div class="flex items-center justify-between mb-4">
              <h3 class="text-lg font-medium text-gray-900 dark:text-white">
                {{ node.hostname }}
              </h3>
              <span
                :class="{
                  'status-online': node.status === 'Online',
                  'status-offline': node.status === 'Offline',
                  'status-degraded': node.status === 'Degraded'
                }"
                class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium"
              >
                {{ node.status }}
              </span>
            </div>
            
            <div class="space-y-3">
              <div class="flex justify-between text-sm">
                <span class="text-gray-600 dark:text-gray-400">GPUs:</span>
                <span class="font-medium text-gray-900 dark:text-white">{{ node.gpus.length }}</span>
              </div>
              <div class="flex justify-between text-sm">
                <span class="text-gray-600 dark:text-gray-400">Processes:</span>
                <span class="font-medium text-gray-900 dark:text-white">{{ node.processes.length }}</span>
              </div>
              <div class="flex justify-between text-sm">
                <span class="text-gray-600 dark:text-gray-400">Last Seen:</span>
                <span class="font-medium text-gray-900 dark:text-white">{{ formatTime(node.timestamp) }}</span>
              </div>
            </div>

            <!-- GPU Details -->
            <div class="mt-4 space-y-2">
              <h4 class="text-sm font-medium text-gray-900 dark:text-white">GPUs:</h4>
              <div
                v-for="gpu in node.gpus"
                :key="gpu.gpu_index"
                class="bg-gray-50 dark:bg-gray-700 rounded-lg p-3"
              >
                <div class="flex justify-between items-start mb-2">
                  <div>
                    <p class="font-medium text-gray-900 dark:text-white">{{ gpu.name }}</p>
                    <p class="text-sm text-gray-600 dark:text-gray-400">
                      {{ formatMemory(gpu.mem_used_mb / 1024) }} / {{ formatMemory(gpu.mem_total_mb / 1024) }}
                    </p>
                  </div>
                  <span class="text-sm font-medium text-gray-900 dark:text-white">
                    {{ Math.round(gpu.util_pct) }}%
                  </span>
                </div>
                <div class="utilization-bar">
                  <div
                    :class="{
                      'utilization-low': gpu.util_pct < 50,
                      'utilization-medium': gpu.util_pct >= 50 && gpu.util_pct < 80,
                      'utilization-high': gpu.util_pct >= 80
                    }"
                    class="utilization-fill"
                    :style="{ width: `${gpu.util_pct}%` }"
                  ></div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </main>
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted } from 'vue'
import {
  ServerIcon,
  CpuChipIcon,
  MemoryIcon,
  ChartBarIcon,
  CheckCircleIcon,
  UserGroupIcon,
  SunIcon,
  MoonIcon
} from '@heroicons/vue/24/outline'

// Reactive data
const clusterSnapshot = ref(null)
const contentionAnalysis = ref(null)
const isDark = ref(false)
const ws = ref(null)

// Configuration
const config = useRuntimeConfig()

// Utility functions
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
  document.documentElement.classList.toggle('dark')
}

// WebSocket connection
const connectWebSocket = () => {
  const wsUrl = config.public.apiBase.replace('http', 'ws') + '/ws'
  ws.value = new WebSocket(wsUrl)
  
  ws.value.onopen = () => {
    console.log('WebSocket connected')
  }
  
  ws.value.onmessage = (event) => {
    try {
      const data = JSON.parse(event.data)
      clusterSnapshot.value = data
      console.log('Received cluster snapshot:', data)
    } catch (error) {
      console.error('Error parsing WebSocket message:', error)
    }
  }
  
  ws.value.onclose = () => {
    console.log('WebSocket disconnected, reconnecting...')
    setTimeout(connectWebSocket, 3000)
  }
  
  ws.value.onerror = (error) => {
    console.error('WebSocket error:', error)
  }
}

// API calls
const fetchClusterSnapshot = async () => {
  try {
    const response = await fetch(`${config.public.apiBase}/api/cluster/snapshot`)
    if (response.ok) {
      clusterSnapshot.value = await response.json()
    }
  } catch (error) {
    console.error('Error fetching cluster snapshot:', error)
  }
}

const fetchContentionAnalysis = async () => {
  try {
    const response = await fetch(`${config.public.apiBase}/api/cluster/contention`)
    if (response.ok) {
      contentionAnalysis.value = await response.json()
    }
  } catch (error) {
    console.error('Error fetching contention analysis:', error)
  }
}

// Lifecycle
onMounted(() => {
  // Check for dark mode preference
  isDark.value = document.documentElement.classList.contains('dark')
  
  // Initial data fetch
  fetchClusterSnapshot()
  fetchContentionAnalysis()
  
  // Connect WebSocket
  connectWebSocket()
  
  // Set up periodic refresh
  const interval = setInterval(() => {
    fetchContentionAnalysis()
  }, 10000) // Refresh every 10 seconds
  
  onUnmounted(() => {
    clearInterval(interval)
    if (ws.value) {
      ws.value.close()
    }
  })
})
</script>
