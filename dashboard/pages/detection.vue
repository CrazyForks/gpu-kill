<template>
  <div class="min-h-screen bg-black overscroll-none flex">
    <!-- Left Sidebar -->
    <AppSidebar :current-tab="activeTab" @tab-change="setActiveTab" />

    <!-- Main Content Area -->
    <div class="flex-1 flex flex-col bg-gray-500/10 mr-2 my-2 rounded-xl border border-gray-500/10 h-screen overflow-hidden">
      <!-- Top Header -->
      <header class="border-b border-gray-500/10 flex-shrink-0">
        <div class="px-6 py-4">
          <div class="flex justify-between items-center">
            <div class="flex items-center space-x-2">
              <h2 class="text-base font-medium text-white">Detection</h2>
              <p class="text-sm text-gray-500">Threat detection and security monitoring</p>
            </div>
                <div class="flex items-center space-x-4">
                  <button @click="performScan" :disabled="isScanning" class="border border-gray-500/10 text-sm rounded-xl px-4 py-2 text-gray-400 bg-transparent hover:bg-gray-500/15 flex items-center space-x-2 disabled:opacity-50 disabled:cursor-not-allowed">
                    <MagnifyingGlassIcon :class="['w-4 h-4', isScanning ? 'animate-pulse' : '']" />
                    <span>{{ isScanning ? 'Scanning...' : 'Scan' }}</span>
                  </button>
									<span class="text-gray-500/20">|</span>
                  <button @click="refreshRogueData" :disabled="isRefreshing" class="border border-gray-500/10 text-sm rounded-xl px-4 py-2 text-white bg-gray-500/10 hover:bg-gray-500/15 flex items-center space-x-2 disabled:opacity-50 disabled:cursor-not-allowed">
                    <ArrowPathIcon :class="['w-4 h-4', isRefreshing ? 'animate-spin' : '']" />
                    <span>{{ isRefreshing ? 'Refreshing...' : 'Refresh' }}</span>
                  </button>
                </div>
          </div>
        </div>
      </header>

      <!-- Main Content -->
      <main class="flex-1 overflow-y-auto divide-y divide-gray-500/10">
        <!-- Rogue Detection -->
        <div class="">
          
          <div v-if="rogueData && !isScanning" class="divide-y divide-gray-500/10">
            <!-- Risk Score -->
            <div class="p-6">
              <div class="flex justify-between items-center mb-2">
                <h3 class="text-sm font-medium text-gray-300">Overall Risk Score</h3>
                <span 
                  class="text-2xl font-bold"
                  :class="{
                    'text-green-400': rogueData.risk_score < 0.3,
                    'text-yellow-400': rogueData.risk_score >= 0.3 && rogueData.risk_score < 0.7,
                    'text-red-400': rogueData.risk_score >= 0.7
                  }"
                >
                  {{ (rogueData.risk_score * 100).toFixed(1) }}%
                </span>
              </div>
              <div class="w-full bg-gray-500/10 rounded-full h-2">
                <div 
                  class="h-2 rounded-full transition-all duration-300"
                  :class="{
                    'bg-green-400': rogueData.risk_score < 0.3,
                    'bg-yellow-400': rogueData.risk_score >= 0.3 && rogueData.risk_score < 0.7,
                    'bg-red-400': rogueData.risk_score >= 0.7
                  }"
                  :style="{ width: `${rogueData.risk_score * 100}%` }"
                ></div>
              </div>
            </div>

            <!-- Crypto Miners -->
            <div v-if="rogueData.crypto_miners?.length > 0" class="p-6">
              <h3 class="text-xs font-medium text-gray-500 uppercase mb-3">Crypto Miners Detected</h3>
              <div v-for="(miner, index) in rogueData.crypto_miners" :key="index" class="mb-3 last:mb-0">
                <div class="bg-gray-500/5 border border-gray-500/10 rounded-xl p-4">
                  <div class="flex justify-between items-start mb-2">
                    <div>
                      <p class="font-medium text-white">PID {{ miner.process.pid }}: {{ miner.process.proc_name }}</p>
                      <p class="text-sm text-gray-300">User: {{ miner.process.user }}</p>
                    </div>
                    <span class="text-sm font-medium text-red-400">
                      Confidence: {{ (miner.confidence * 100).toFixed(1) }}%
                    </span>
                  </div>
                  <div class="text-sm text-gray-300">
                    <p class="font-medium mb-1">Indicators:</p>
                    <ul class="list-disc list-inside space-y-1">
                      <li v-for="indicator in miner.mining_indicators" :key="indicator">
                        {{ indicator }}
                      </li>
                    </ul>
                  </div>
                </div>
              </div>
            </div>

            <!-- Suspicious Processes -->
            <div v-if="rogueData.suspicious_processes?.length > 0" class="p-6">
              <h3 class="text-xs font-medium text-gray-500 uppercase mb-3">Suspicious Processes</h3>
              <div v-for="(process, index) in rogueData.suspicious_processes" :key="index" class="mb-3 last:mb-0">
                <div class="bg-gray-500/5 border border-gray-500/10 rounded-xl p-4">
                  <div class="flex justify-between items-start mb-2">
                    <div>
                      <p class="font-medium text-white">PID {{ process.process.pid }}: {{ process.process.proc_name }}</p>
                      <p class="text-sm text-gray-300">User: {{ process.process.user }}</p>
                    </div>
                    <div class="text-right">
                      <span 
                        class="text-sm font-medium"
                        :class="{
                          'text-red-400': process.risk_level === 'Critical',
                          'text-yellow-400': process.risk_level === 'High',
                          'text-blue-300': process.risk_level === 'Medium',
                          'text-green-400': process.risk_level === 'Low'
                        }"
                      >
                        {{ process.risk_level }}
                      </span>
                      <p class="text-xs text-gray-400">Confidence: {{ (process.confidence * 100).toFixed(1) }}%</p>
                    </div>
                  </div>
                  <div class="text-sm text-gray-300">
                    <p class="font-medium mb-1">Reasons:</p>
                    <ul class="list-disc list-inside space-y-1">
                      <li v-for="reason in process.reasons" :key="reason">
                        {{ reason }}
                      </li>
                    </ul>
                  </div>
                </div>
              </div>
            </div>

            <!-- Resource Abusers -->
            <div v-if="rogueData.resource_abusers?.length > 0" class="p-6">
              <h3 class="text-xs font-medium text-gray-500 uppercase mb-3">Resource Abusers</h3>
              <div v-for="(abuser, index) in rogueData.resource_abusers" :key="index" class="mb-3 last:mb-0">
                <div class="bg-gray-500/5 border border-gray-500/10 rounded-xl p-4">
                  <div class="flex justify-between items-start mb-2">
                    <div>
                      <p class="font-medium text-white">PID {{ abuser.process.pid }}: {{ abuser.process.proc_name }}</p>
                      <p class="text-sm text-gray-300">User: {{ abuser.process.user }}</p>
                    </div>
                    <div class="text-right">
                      <span class="text-sm font-medium text-orange-400">
                        Severity: {{ abuser.severity.toFixed(2) }}
                      </span>
                      <p class="text-xs text-gray-400">{{ abuser.duration_hours.toFixed(1) }}h duration</p>
                    </div>
                  </div>
                  <div class="text-sm text-gray-300">
                    <p class="font-medium">Abuse Type: {{ getAbuseTypeName(abuser.abuse_type) }}</p>
                  </div>
                </div>
              </div>
            </div>

            <!-- All Clear -->
            <div v-if="rogueData && !rogueData.crypto_miners?.length && !rogueData.suspicious_processes?.length && !rogueData.resource_abusers?.length" 
                class="bg-green-400/5 border max-w-sm mx-auto border-green-400/10 rounded-xl p-4 text-center">
								<div class="text-center py-8"">
									<div class="flex items-center rounded-full bg-green-400/10 justify-center mx-auto mb-4">
										<CheckCircleIcon class="w-6 h-6 text-green-400" />
									</div>
									<div class="flex items-center text-lg text-white justify-center space-x-2 mb-2">All Clear!</div>
									<p class="text-gray-400">No suspicious activity detected in the last 24 hours.</p>
							</div>
            </div>

            <!-- Recommendations -->
            <div v-if="rogueData.recommendations?.length > 0" class="p-6">
              <h3 class="text-xs font-medium text-blue-300 uppercase mb-3">Recommendations</h3>
              <ul class="space-y-2">
                <li v-for="(recommendation, index) in rogueData.recommendations" :key="index" 
                    class="text-gray-300 flex items-start">
                  <span class="mr-2">•</span>
                  <span>{{ recommendation }}</span>
                </li>
              </ul>
            </div>
          </div>

					<!-- Empty State -->
          <div v-if="!rogueData || isScanning" class="text-center text-gray-500 text-sm py-8 my-6 max-w-sm mx-auto bg-gray-500/5 rounded-xl border border-gray-500/10 p-4">
            <div class="text-center py-8">
							<div class="w-12 h-12 bg-gray-500/10 rounded-full flex items-center justify-center mx-auto mb-4">
								<MagnifyingGlassIcon class="w-6 h-6 text-gray-500" />
							</div>
							<div class="flex items-center text-lg text-white justify-center space-x-2 mb-2">
								Scan for threats
							</div>
							<p class="text-gray-400">Click "Scan" to perform rogue activity detection.</p>
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
  ArrowPathIcon,
  MagnifyingGlassIcon,
  CheckCircleIcon
} from '@heroicons/vue/24/solid'

// Page metadata
useHead({
  title: 'Rogue Detection',
})

const config = useRuntimeConfig()
const rogueData = ref(null)
const activeTab = ref('detection')
const isRefreshing = ref(false)
const isScanning = ref(false)

const fetchRogueData = async () => {
  try {
    // For testing purposes, use sample data
    const testData = {
      timestamp: new Date().toISOString(),
      suspicious_processes: [
        {
          process: {
            gpu_index: 0,
            pid: 12345,
            user: "hacker",
            proc_name: "suspicious_miner",
            used_mem_mb: 2048,
            start_time: "2025-09-20T01:00:00Z",
            container: null
          },
          reasons: [
            "High GPU utilization with low CPU usage",
            "Process name contains mining keywords",
            "Unusual memory allocation patterns"
          ],
          confidence: 0.85,
          risk_level: "High"
        }
      ],
      crypto_miners: [
        {
          process: {
            gpu_index: 1,
            pid: 67890,
            user: "miner",
            proc_name: "xmrig",
            used_mem_mb: 1024,
            start_time: "2025-09-20T00:30:00Z",
            container: null
          },
          mining_indicators: [
            "Known cryptocurrency mining software",
            "Extremely high GPU utilization",
            "Long-running process with consistent resource usage"
          ],
          confidence: 0.92,
          estimated_hashrate: 150.5
        }
      ],
      resource_abusers: [
        {
          process: {
            gpu_index: 2,
            pid: 11111,
            user: "abuser",
            proc_name: "gpu_hog",
            used_mem_mb: 8192,
            start_time: "2025-09-19T20:00:00Z",
            container: null
          },
          abuse_type: "MemoryHog",
          severity: 0.9,
          duration_hours: 8.5
        }
      ],
      data_exfiltrators: [],
      risk_score: 0.78,
      recommendations: [
        "Immediate action required: Terminate crypto mining processes",
        "Review user 'miner' and 'hacker' accounts for unauthorized access",
        "Investigate process 'gpu_hog' for potential resource abuse",
        "Consider implementing GPU usage quotas per user"
      ]
    }
    
    rogueData.value = testData
    console.log('Rogue data (test):', rogueData.value)
    
    // Uncomment below to use real API
    // const response = await fetch(`${config.public.apiBase}/api/cluster/rogue`)
    // if (response.ok) {
    //   const data = await response.json()
    //   rogueData.value = data
    //   console.log('Rogue data:', data)
    // }
  } catch (error) {
    console.error('Error fetching rogue data:', error)
  }
}

const refreshRogueData = async () => {
  if (isRefreshing.value) return
  
  isRefreshing.value = true
  try {
    await fetchRogueData()
    
    // Ensure minimum loading time of 1 second
    await new Promise(resolve => setTimeout(resolve, 1000))
  } finally {
    isRefreshing.value = false
  }
}

const performScan = async () => {
  if (isScanning.value) return
  
  isScanning.value = true
  try {
    // Perform the scan
    await fetchRogueData()
    
    // Ensure minimum scanning time of 2 seconds for better UX
    await new Promise(resolve => setTimeout(resolve, 2000))
  } finally {
    isScanning.value = false
  }
}

const setActiveTab = (tab) => {
  activeTab.value = tab
}

const getAbuseTypeName = (abuseType) => {
  const typeMap = {
    'memory': 'Memory Abuse',
    'utilization': 'Utilization Abuse',
    'process_count': 'Process Count Abuse',
    'duration': 'Duration Abuse'
  }
  return typeMap[abuseType] || abuseType
}

let refreshInterval

onMounted(() => {
  console.log('Detection page mounted')
  
  // Auto-refresh every 30 seconds (only if data exists)
  refreshInterval = setInterval(() => {
    if (rogueData.value) {
      fetchRogueData()
    }
  }, 30000)
})

onUnmounted(() => {
  if (refreshInterval) {
    clearInterval(refreshInterval)
  }
})
</script>
