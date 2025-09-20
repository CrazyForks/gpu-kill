<template>
  <aside class="w-64 h-screen flex flex-col sticky top-0 overflow-y-auto">
    <!-- Sidebar Header -->
    <div class="p-4 py-3 flex items-center justify-between">
      <div class="flex items-center space-x-1">
        <img src="/assets/img/logo.png" alt="GPU Kill Logo" class="w-6 h-6" />
        <div>
          <h1 class="text-sm font-semibold text-white">GPU Kill</h1>
        </div>
      </div>
      <div class="flex items-center space-x-2 px-2 py-1.5">
        <div class="w-2 h-2 bg-green-400 rounded-full animate-pulse-slow"></div>
        <span class="text-xs text-gray-400 font-medium">Connected</span>
      </div>
    </div>

    <!-- Navigation -->
    <nav class="flex-1 p-4 space-y-2 overflow-y-auto">
      <a 
        href="/" 
        @click="setActiveTab('overview')"
        :class="[
          'flex items-center space-x-3 text-sm px-3 py-2 border border-transparent rounded-xl transition-colors',
          activeTab === 'overview' 
            ? 'bg-gray-500/10 border !border-gray-500/10 text-white' 
            : 'text-white hover:bg-gray-500/10 hover:border-gray-500/10'
        ]"
      >
        <ChartBarIcon :class="[
          'w-4 h-4',
          activeTab === 'overview' ? 'text-gray-500' : 'text-gray-500'
        ]" />
        <span>Overview</span>
      </a>
      <a 
        href="/detection" 
        @click="setActiveTab('detection')"
        :class="[
          'flex items-center space-x-3 text-sm px-3 py-2 border border-transparent rounded-xl transition-colors',
          activeTab === 'detection' 
            ? 'bg-gray-500/10 border !border-gray-500/10 text-white' 
            : 'text-white hover:bg-gray-500/10 hover:border-gray-500/10'
        ]"
      >
        <ShieldExclamationIcon :class="[
          'w-4 h-4',
          activeTab === 'detection' ? 'text-gray-500' : 'text-gray-500'
        ]" />
        <span>Detection</span>
      </a>
      <a 
        href="/guard" 
        @click="setActiveTab('guard')"
        :class="[
          'flex items-center space-x-3 text-sm px-3 py-2 border border-transparent rounded-xl transition-colors',
          activeTab === 'guard' 
            ? 'bg-gray-500/10 border border-gray-500/10 text-white' 
            : 'text-white hover:bg-gray-500/10 hover:border-gray-500/10'
        ]"
      >
        <ShieldCheckIcon :class="[
          'w-4 h-4',
          activeTab === 'guard' ? 'text-gray-500' : 'text-gray-500'
        ]" />
        <span>Guard</span>
      </a>
    </nav>
  </aside>
</template>

<script setup>
import { ref, onMounted, watch } from 'vue'
import {
  ChartBarIcon,
  ShieldExclamationIcon,
  ShieldCheckIcon
} from '@heroicons/vue/24/solid'

// Props
const props = defineProps({
  currentTab: {
    type: String,
    default: 'overview'
  }
})

// Emits
const emit = defineEmits(['tab-change'])

// Reactive data
const activeTab = ref(props.currentTab)

// Methods
const setActiveTab = (tab) => {
  activeTab.value = tab
  emit('tab-change', tab)
}

// Watch for prop changes
watch(() => props.currentTab, (newTab) => {
  activeTab.value = newTab
})

// Set initial active tab based on current route
onMounted(() => {
  const route = useRoute()
  if (route.path === '/') {
    activeTab.value = 'overview'
  } else if (route.path === '/detection') {
    activeTab.value = 'detection'
  } else if (route.path === '/guard') {
    activeTab.value = 'guard'
  }
})
</script>
