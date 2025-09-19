<template>
  <div class="min-h-screen bg-black">
    <!-- Header -->
    <header class="bg-black border-b border-gray-500/20">
      <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div class="flex justify-between items-center h-16">
          <div class="flex items-center">
            <div class="flex-shrink-0">
              <h1 class="text-2xl font-bold text-white">
                🚀 GPU Kill
              </h1>
            </div>
          </div>
          <div class="flex items-center space-x-4">
            <div class="flex items-center space-x-2">
              <div class="w-2 h-2 bg-green-500 rounded-full animate-pulse"></div>
              <span class="text-sm text-gray-600 dark:text-gray-400">Live</span>
            </div>
          </div>
        </div>
      </div>
    </header>

    <!-- Main Content -->
    <main class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
      <!-- Cluster Overview -->
      <div class="mb-8">
        <h2 class="text-xl font-semibold text-white mb-4">
          📊 Cluster Overview
        </h2>
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
          <div class="bg-gray-500/10 rounded-xl border border-gray-500/20 p-4">
            <div class="flex items-center">
              <div class="flex-shrink-0">
                <span class="text-2xl">🖥️</span>
              </div>
              <div class="ml-4">
                <p class="text-sm font-medium text-gray-400">Total Nodes</p>
                <p class="text-2xl font-bold text-white">{{ clusterData?.nodes?.length || 0 }}</p>
              </div>
            </div>
          </div>
          
          <div class="bg-gray-500/10 rounded-xl border border-gray-500/20 p-4">
            <div class="flex items-center">
              <div class="flex-shrink-0">
                <span class="text-2xl">🎮</span>
              </div>
              <div class="ml-4">
                <p class="text-sm font-medium text-gray-400">Total GPUs</p>
                <p class="text-2xl font-bold text-white">{{ clusterData?.total_gpus || 0 }}</p>
              </div>
            </div>
          </div>
          
          <div class="bg-gray-500/10 rounded-xl border border-gray-500/20 p-4">
            <div class="flex items-center">
              <div class="flex-shrink-0">
                <span class="text-2xl">💾</span>
              </div>
              <div class="ml-4">
                <p class="text-sm font-medium text-gray-400">Total Memory</p>
                <p class="text-2xl font-bold text-white">{{ formatMemory(clusterData?.total_memory_gb || 0) }}</p>
              </div>
            </div>
          </div>
          
          <div class="bg-gray-500/10 rounded-xl border border-gray-500/20 p-4">
            <div class="flex items-center">
              <div class="flex-shrink-0">
                <span class="text-2xl">📈</span>
              </div>
              <div class="ml-4">
                <p class="text-sm font-medium text-gray-400">Avg Utilization</p>
                <p class="text-2xl font-bold text-white">{{ Math.round(clusterData?.utilization_avg || 0) }}%</p>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Magic Moment - Contention Analysis -->
      <div class="mb-8">
        <h2 class="text-xl font-semibold text-white mb-4">
          ⚡ Magic Moment - GPU Contention
        </h2>
        <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
          <!-- Blocked GPUs -->
          <div class="bg-gray-500/10 rounded-xl border border-gray-500/20 p-4">
            <h3 class="text-lg font-medium text-white mb-4">
              🚫 Blocked GPUs
            </h3>
            <div v-if="contentionData?.blocked_gpus?.length === 0" class="text-center py-8">
              <span class="text-4xl mb-2 block">✅</span>
              <p class="text-gray-600 dark:text-gray-400">No blocked GPUs! 🎉</p>
            </div>
            <div v-else class="space-y-3">
              <div 
                v-for="gpu in contentionData?.blocked_gpus" 
                :key="`${gpu.node_id}-${gpu.gpu_index}`"
                class="border border-red-500/10 rounded-lg p-3 bg-red-500/10"
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
                  <span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-red-500/10 text-red-500">
                    Blocked
                  </span>
                </div>
                <div class="mt-2">
                  <div class="w-full bg-gray-500/10 rounded-full h-2.5">
                    <div 
                      class="h-2.5 rounded-full bg-red-500 transition-all duration-300" 
                      :style="{ width: `${gpu.utilization_pct}%` }"
                    ></div>
                  </div>
                </div>
              </div>
            </div>
          </div>

          <!-- Top Users -->
          <div class="bg-gray-500/10 rounded-xl border border-gray-500/20 p-4">
            <h3 class="text-lg font-medium text-white mb-4">
              👥 Top Users
            </h3>
            <div v-if="contentionData?.top_users?.length === 0" class="text-center py-8">
              <span class="text-4xl mb-2 block">👥</span>
              <p class="text-gray-600 dark:text-gray-400">No active users</p>
            </div>
            <div v-else class="space-y-3">
              <div 
                v-for="(user, index) in contentionData?.top_users" 
                :key="user.user"
                class="flex items-center justify-between p-3 bg-gray-500/10 rounded-xl"
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

      <!-- Rogue Detection -->
      <div class="mb-8">
        <div class="flex justify-between items-center mb-4">
          <h2 class="text-xl font-semibold text-white">
            🕵️ Rogue Activity Detection
          </h2>
          <button 
            @click="refreshRogueData"
            class="bg-red-600 hover:bg-red-700 text-white px-4 py-2 rounded-lg font-medium transition-colors"
          >
            Scan for Threats
          </button>
        </div>
        
        <div v-if="rogueData" class="space-y-4">
          <!-- Risk Score -->
          <div class="bg-gray-500/10 rounded-xl border border-gray-500/20 p-4">
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
            <div class="w-full bg-gray-600 rounded-full h-2">
              <div 
                class="h-2 rounded-full transition-all duration-300"
                :class="{
                  'bg-green-500': rogueData.risk_score < 0.3,
                  'bg-yellow-500': rogueData.risk_score >= 0.3 && rogueData.risk_score < 0.7,
                  'bg-red-500': rogueData.risk_score >= 0.7
                }"
                :style="{ width: `${rogueData.risk_score * 100}%` }"
              ></div>
            </div>
          </div>

          <!-- Crypto Miners -->
          <div v-if="rogueData.crypto_miners?.length > 0" class="bg-red-500/10 border border-red-500/10 rounded-xl p-4">
            <h3 class="text-lg font-bold text-red-400 mb-3">🚨 Crypto Miners Detected</h3>
            <div v-for="(miner, index) in rogueData.crypto_miners" :key="index" class="mb-3 last:mb-0">
              <div class="bg-red-800/30 rounded-lg p-3">
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
          <div v-if="rogueData.suspicious_processes?.length > 0" class="bg-yellow-400/10 border border-yellow-400/10 rounded-xl p-4">
            <h3 class="text-lg font-bold text-yellow-400 mb-3">⚠️ Suspicious Processes</h3>
            <div v-for="(process, index) in rogueData.suspicious_processes" :key="index" class="mb-3 last:mb-0">
              <div class="bg-yellow-800/30 rounded-lg p-3">
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
                        'text-blue-400': process.risk_level === 'Medium',
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
          <div v-if="rogueData.resource_abusers?.length > 0" class="bg-orange-400/10 border border-orange-400/10 rounded-xl p-4">
            <h3 class="text-lg font-bold text-orange-400 mb-3">📊 Resource Abusers</h3>
            <div v-for="(abuser, index) in rogueData.resource_abusers" :key="index" class="mb-3 last:mb-0">
              <div class="bg-orange-800/30 rounded-lg p-3">
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
          <div v-if="!rogueData.crypto_miners?.length && !rogueData.suspicious_processes?.length && !rogueData.resource_abusers?.length" 
               class="bg-green-900/20 border border-green-500/30 rounded-lg p-4 text-center">
            <div class="text-green-400 text-lg font-bold mb-2">✅ All Clear!</div>
            <p class="text-gray-300">No suspicious activity detected in the last 24 hours.</p>
          </div>

          <!-- Recommendations -->
          <div v-if="rogueData.recommendations?.length > 0" class="bg-blue-300/10 border border-blue-300/10 rounded-xl p-4">
            <h3 class="text-lg font-bold text-blue-400 mb-3">📋 Recommendations</h3>
            <ul class="space-y-2">
              <li v-for="(recommendation, index) in rogueData.recommendations" :key="index" 
                  class="text-gray-300 flex items-start">
                <span class="mr-2">•</span>
                <span>{{ recommendation }}</span>
              </li>
            </ul>
          </div>
        </div>

        <div v-else class="text-center text-gray-400 py-8 bg-gray-500/10 rounded-xl border border-gray-500/20 p-4">
          <p>Click "Scan for Threats" to perform rogue activity detection</p>
        </div>
      </div>

      <!-- Guard Mode -->
      <div class="mb-8">
        <div class="flex justify-between items-center mb-4">
          <h2 class="text-xl font-semibold text-white">
            🛡️ Guard Mode - Policy Enforcement
          </h2>
          <div class="flex space-x-2">
            <button
              @click="refreshGuardData"
              class="bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 rounded-lg font-medium transition-colors"
            >
              Refresh Status
            </button>
            <button
              @click="toggleGuardMode"
              :class="[
                'px-4 py-2 rounded-lg font-medium transition-colors',
                guardStatus?.enabled 
                  ? 'bg-green-600 hover:bg-green-700 text-white' 
                  : 'bg-gray-600 hover:bg-gray-700 text-white'
              ]"
            >
              {{ guardStatus?.enabled ? 'Disable' : 'Enable' }} Guard Mode
            </button>
            <button
              @click="toggleDryRunMode"
              :class="[
                'px-4 py-2 rounded-lg font-medium transition-colors',
                guardStatus?.dry_run 
                  ? 'bg-yellow-600 hover:bg-yellow-700 text-white' 
                  : 'bg-orange-600 hover:bg-orange-700 text-white'
              ]"
            >
              {{ guardStatus?.dry_run ? '🧪 Dry Run' : '⚡ Enforcing' }}
            </button>
            <button
              @click="testPolicies"
              class="bg-purple-600 hover:bg-purple-700 text-white px-4 py-2 rounded-lg font-medium transition-colors"
            >
              🧪 Test Policies
            </button>
          </div>
        </div>

        <div v-if="guardStatus" class="space-y-4">
          <!-- Status Overview -->
          <div class="bg-gray-500/10 rounded-xl border border-gray-500/20 p-4">
            <div class="grid grid-cols-1 md:grid-cols-4 gap-4">
              <div class="text-center">
                <div class="text-2xl font-bold text-white mb-1">
                  {{ guardStatus.enabled ? '🟢' : '🔴' }}
                </div>
                <div class="text-sm text-gray-400">Status</div>
                <div class="text-xs text-gray-300">
                  {{ guardStatus.enabled ? 'Enabled' : 'Disabled' }}
                </div>
              </div>
              <div class="text-center">
                <div class="text-2xl font-bold text-white mb-1">
                  {{ guardStatus.dry_run ? '🧪' : '⚡' }}
                </div>
                <div class="text-sm text-gray-400">Mode</div>
                <div class="text-xs text-gray-300">
                  {{ guardStatus.dry_run ? 'Dry Run' : 'Enforcing' }}
                </div>
              </div>
              <div class="text-center">
                <div class="text-2xl font-bold text-white mb-1">
                  {{ guardStatus.total_violations }}
                </div>
                <div class="text-sm text-gray-400">Violations</div>
                <div class="text-xs text-gray-300">Total</div>
              </div>
              <div class="text-center">
                <div class="text-2xl font-bold text-white mb-1">
                  {{ guardStatus.user_policy_count }}
                </div>
                <div class="text-sm text-gray-400">Policies</div>
                <div class="text-xs text-gray-300">Active</div>
              </div>
            </div>
          </div>

          <!-- Enforcement Settings -->
          <div class="bg-gray-500/10 rounded-xl border border-gray-500/20 p-4">
            <h3 class="text-lg font-bold text-white mb-3">⚙️ Enforcement Settings</h3>
            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div class="flex items-center justify-between">
                <span class="text-gray-300">Soft Enforcement</span>
                <span :class="[
                  'px-2 py-1 rounded text-xs font-medium',
                  guardStatus.soft_enforcement 
                    ? 'bg-green-500/20 text-green-400' 
                    : 'bg-gray-500/20 text-gray-400'
                ]">
                  {{ guardStatus.soft_enforcement ? 'Enabled' : 'Disabled' }}
                </span>
              </div>
              <div class="flex items-center justify-between">
                <span class="text-gray-300">Hard Enforcement</span>
                <span :class="[
                  'px-2 py-1 rounded text-xs font-medium',
                  guardStatus.hard_enforcement 
                    ? 'bg-red-500/20 text-red-400' 
                    : 'bg-gray-500/20 text-gray-400'
                ]">
                  {{ guardStatus.hard_enforcement ? 'Enabled' : 'Disabled' }}
                </span>
              </div>
            </div>
          </div>

          <!-- Recent Violations -->
          <div v-if="guardStatus.recent_violations?.length > 0" class="bg-red-500/10 border border-red-500/10 rounded-xl p-4">
            <h3 class="text-lg font-bold text-red-400 mb-3">🚨 Recent Violations</h3>
            <div class="space-y-2">
              <div v-for="(violation, index) in guardStatus.recent_violations.slice(0, 5)" :key="index" 
                   class="bg-red-800/30 rounded-lg p-3">
                <div class="flex justify-between items-start mb-2">
                  <div>
                    <p class="font-medium text-white">{{ violation.user }} - {{ violation.violation_type }}</p>
                    <p class="text-sm text-gray-300">{{ violation.message }}</p>
                  </div>
                  <span :class="[
                    'text-sm font-medium px-2 py-1 rounded',
                    getViolationSeverityClass(violation.severity)
                  ]">
                    {{ violation.severity }}
                  </span>
                </div>
                <div class="text-xs text-gray-400">
                  {{ formatTime(violation.timestamp) }}
                </div>
              </div>
            </div>
          </div>

          <!-- Recent Warnings -->
          <div v-if="guardStatus.recent_warnings?.length > 0" class="bg-yellow-500/10 border border-yellow-500/10 rounded-xl p-4">
            <h3 class="text-lg font-bold text-yellow-400 mb-3">⚠️ Recent Warnings</h3>
            <div class="space-y-2">
              <div v-for="(warning, index) in guardStatus.recent_warnings.slice(0, 5)" :key="index" 
                   class="bg-yellow-800/30 rounded-lg p-3">
                <div class="flex justify-between items-start mb-2">
                  <div>
                    <p class="font-medium text-white">{{ warning.user }} - {{ warning.warning_type }}</p>
                    <p class="text-sm text-gray-300">{{ warning.message }}</p>
                  </div>
                  <span class="text-sm font-medium text-yellow-400">
                    Warning
                  </span>
                </div>
                <div class="text-xs text-gray-400">
                  {{ formatTime(warning.timestamp) }}
                </div>
              </div>
            </div>
          </div>

          <!-- All Clear -->
          <div v-if="!guardStatus.recent_violations?.length && !guardStatus.recent_warnings?.length"
               class="bg-green-500/10 border border-green-500/10 rounded-xl p-4 text-center">
            <div class="text-green-400 text-lg font-bold mb-2">✅ All Clear!</div>
            <p class="text-gray-300">No recent policy violations or warnings.</p>
          </div>

          <!-- Simulation Results -->
          <div v-if="simulationResults" class="bg-purple-500/10 border border-purple-500/10 rounded-xl p-4">
            <h3 class="text-lg font-bold text-purple-400 mb-3">🧪 Policy Test Results</h3>
            <div class="grid grid-cols-1 md:grid-cols-3 gap-4 mb-4">
              <div class="text-center">
                <div class="text-2xl font-bold text-red-400 mb-1">
                  {{ simulationResults.summary?.violation_count || 0 }}
                </div>
                <div class="text-sm text-gray-400">Violations</div>
              </div>
              <div class="text-center">
                <div class="text-2xl font-bold text-yellow-400 mb-1">
                  {{ simulationResults.summary?.warning_count || 0 }}
                </div>
                <div class="text-sm text-gray-400">Warnings</div>
              </div>
              <div class="text-center">
                <div class="text-2xl font-bold text-blue-400 mb-1">
                  {{ simulationResults.summary?.action_count || 0 }}
                </div>
                <div class="text-sm text-gray-400">Actions</div>
              </div>
            </div>
            
            <div v-if="simulationResults.simulation_result?.violations?.length > 0" class="mb-4">
              <h4 class="text-md font-bold text-red-400 mb-2">🚨 Simulated Violations:</h4>
              <div class="space-y-2">
                <div v-for="(violation, index) in simulationResults.simulation_result.violations.slice(0, 3)" :key="index" 
                     class="bg-red-800/30 rounded-lg p-2">
                  <div class="flex justify-between items-start">
                    <div>
                      <p class="font-medium text-white text-sm">{{ violation.user }} - {{ violation.violation_type }}</p>
                      <p class="text-xs text-gray-300">{{ violation.message }}</p>
                    </div>
                    <span :class="[
                      'text-xs font-medium px-2 py-1 rounded',
                      getViolationSeverityClass(violation.severity)
                    ]">
                      {{ violation.severity }}
                    </span>
                  </div>
                </div>
              </div>
            </div>

            <div v-if="simulationResults.simulation_result?.actions_taken?.length > 0" class="mb-4">
              <h4 class="text-md font-bold text-blue-400 mb-2">⚡ Simulated Actions:</h4>
              <div class="space-y-1">
                <div v-for="(action, index) in simulationResults.simulation_result.actions_taken.slice(0, 3)" :key="index" 
                     class="bg-blue-800/30 rounded-lg p-2">
                  <p class="text-sm text-white">{{ action.action_type }}: {{ action.message }}</p>
                </div>
              </div>
            </div>

            <div v-if="!simulationResults.simulation_result?.violations?.length && !simulationResults.simulation_result?.warnings?.length"
                 class="text-center">
              <div class="text-green-400 text-lg font-bold mb-2">✅ No Issues Found!</div>
              <p class="text-gray-300">All policies passed the simulation test.</p>
            </div>
          </div>

          <!-- Policy Management -->
          <div class="bg-gray-500/10 rounded-xl border border-gray-500/20 p-4">
            <h3 class="text-lg font-bold text-white mb-3">📋 Policy Management</h3>
            <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
              <div class="text-center">
                <div class="text-2xl font-bold text-blue-400 mb-1">
                  {{ guardStatus.user_policy_count }}
                </div>
                <div class="text-sm text-gray-400">User Policies</div>
              </div>
              <div class="text-center">
                <div class="text-2xl font-bold text-purple-400 mb-1">
                  {{ guardStatus.group_policy_count }}
                </div>
                <div class="text-sm text-gray-400">Group Policies</div>
              </div>
              <div class="text-center">
                <div class="text-2xl font-bold text-orange-400 mb-1">
                  {{ guardStatus.gpu_policy_count }}
                </div>
                <div class="text-sm text-gray-400">GPU Policies</div>
              </div>
            </div>
            <div class="mt-4 text-center">
              <button
                @click="showPolicyEditor = !showPolicyEditor"
                class="bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 rounded-lg font-medium transition-colors"
              >
                {{ showPolicyEditor ? 'Hide' : 'Show' }} Policy Editor
              </button>
            </div>
          </div>

          <!-- Policy Editor (Collapsible) -->
          <div v-if="showPolicyEditor" class="bg-gray-500/10 rounded-xl border border-gray-500/20 p-4">
            <h3 class="text-lg font-bold text-white mb-3">✏️ Policy Editor</h3>
            <div class="space-y-4">
              <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div>
                  <label class="block text-sm font-medium text-gray-300 mb-2">Username</label>
                  <input
                    v-model="newPolicy.username"
                    type="text"
                    placeholder="Enter username"
                    class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500"
                  />
                </div>
                <div>
                  <label class="block text-sm font-medium text-gray-300 mb-2">Memory Limit (GB)</label>
                  <input
                    v-model="newPolicy.memoryLimit"
                    type="number"
                    step="0.1"
                    placeholder="16.0"
                    class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500"
                  />
                </div>
                <div>
                  <label class="block text-sm font-medium text-gray-300 mb-2">Utilization Limit (%)</label>
                  <input
                    v-model="newPolicy.utilizationLimit"
                    type="number"
                    step="1"
                    placeholder="80"
                    class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500"
                  />
                </div>
                <div>
                  <label class="block text-sm font-medium text-gray-300 mb-2">Process Limit</label>
                  <input
                    v-model="newPolicy.processLimit"
                    type="number"
                    step="1"
                    placeholder="5"
                    class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500"
                  />
                </div>
              </div>
              <div class="flex space-x-2">
                <button
                  @click="addUserPolicy"
                  class="bg-green-600 hover:bg-green-700 text-white px-4 py-2 rounded-lg font-medium transition-colors"
                >
                  Add Policy
                </button>
                <button
                  @click="clearPolicyForm"
                  class="bg-gray-600 hover:bg-gray-700 text-white px-4 py-2 rounded-lg font-medium transition-colors"
                >
                  Clear
                </button>
              </div>
            </div>
          </div>
        </div>

        <div v-else class="text-center text-gray-400 py-8 bg-gray-500/10 rounded-xl border border-gray-500/20 p-4">
          <p>Click "Refresh Status" to load Guard Mode information</p>
        </div>
      </div>

      <!-- Node Details -->
      <div class="mb-8">
        <h2 class="text-xl font-semibold text-white mb-4">
          🖥️ Node Details
        </h2>
        <div v-if="clusterData?.nodes?.length === 0" class="text-center py-8">
          <p class="text-gray-400">No nodes found</p>
        </div>
        <div v-else class="grid grid-cols-1 lg:grid-cols-2 xl:grid-cols-3 gap-6">
          <div 
            v-for="node in clusterData?.nodes" 
            :key="node.node_id"
            class="bg-gray-500/10 rounded-xl border border-gray-500/20 p-4"
          >
            <div class="flex items-center justify-between mb-4">
              <h3 class="text-lg font-medium text-white">
                {{ node.hostname }}
              </h3>
              <span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-green-500/10 text-green-500">
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
                class="bg-gray-500/10 rounded-xl border border-gray-500/20 p-3"
              >
                <div class="flex justify-between items-start mb-2">
                  <div>
                    <p class="font-medium text-white">{{ gpu.name }}</p>
                    <p class="text-sm text-gray-600 dark:text-gray-400">
                      {{ formatMemory(gpu.mem_used_mb / 1024) }} / {{ formatMemory(gpu.mem_total_mb / 1024) }}
                    </p>
                  </div>
                  <span class="text-sm font-medium text-white">
                    {{ Math.round(gpu.util_pct) }}%
                  </span>
                </div>
                <div class="w-full bg-gray-500/10 rounded-full h-2.5">
                  <div 
                    class="h-2.5 rounded-full transition-all duration-300"
                    :class="{
                      'bg-green-500': gpu.util_pct < 50,
                      'bg-yellow-500': gpu.util_pct >= 50 && gpu.util_pct < 80,
                      'bg-red-500': gpu.util_pct >= 80
                    }"
                    :style="{ width: `${gpu.util_pct}%` }"
                  ></div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Refresh Button -->
      <div class="text-center">
        <button 
          @click="refreshData"
          class="bg-blue-300 text-black px-6 py-3 rounded-xl font-medium transition-colors"
        >
          Refresh Data
        </button>
      </div>
    </main>
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted } from 'vue'

const config = useRuntimeConfig()
const clusterData = ref(null)
const contentionData = ref(null)
const rogueData = ref(null)
const guardStatus = ref(null)
const simulationResults = ref(null)
const isDark = ref(false)
const showPolicyEditor = ref(false)
const newPolicy = ref({
  username: '',
  memoryLimit: 16.0,
  utilizationLimit: 80,
  processLimit: 5
})
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
    }
  } catch (error) {
    console.error('Error fetching cluster data:', error)
  }
}

const fetchContentionData = async () => {
  try {
    const response = await fetch(`${config.public.apiBase}/api/cluster/contention`)
    if (response.ok) {
      contentionData.value = await response.json()
      console.log('Contention data:', contentionData.value)
    }
  } catch (error) {
    console.error('Error fetching contention data:', error)
  }
}

const fetchRogueData = async () => {
  try {
    const response = await fetch(`${config.public.apiBase}/api/cluster/rogue`)
    if (response.ok) {
      rogueData.value = await response.json()
      console.log('Rogue data:', rogueData.value)
    }
  } catch (error) {
    console.error('Error fetching rogue data:', error)
  }
}

const refreshRogueData = () => {
  fetchRogueData()
}

const getAbuseTypeName = (abuseType) => {
  const typeMap = {
    'MemoryHog': 'Memory Hog',
    'LongRunning': 'Long Running',
    'ExcessiveUtilization': 'Excessive Utilization',
    'UnauthorizedAccess': 'Unauthorized Access'
  }
  return typeMap[abuseType] || abuseType
}

const fetchGuardData = async () => {
  try {
    const response = await fetch(`${config.public.apiBase}/api/guard/status`)
    if (response.ok) {
      guardStatus.value = await response.json()
      console.log('Guard status:', guardStatus.value)
    }
  } catch (error) {
    console.error('Error fetching guard status:', error)
  }
}

const refreshGuardData = () => {
  fetchGuardData()
}

const toggleGuardMode = async () => {
  try {
    const newStatus = !guardStatus.value.enabled
    const response = await fetch(`${config.public.apiBase}/api/guard/config`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        ...guardStatus.value,
        global: {
          ...guardStatus.value.global,
          enabled: newStatus
        }
      })
    })
    
    if (response.ok) {
      await fetchGuardData()
    }
  } catch (error) {
    console.error('Error toggling guard mode:', error)
  }
}

const addUserPolicy = async () => {
  if (!newPolicy.value.username) {
    alert('Please enter a username')
    return
  }

  try {
    const response = await fetch(`${config.public.apiBase}/api/guard/policies`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        user_policies: {
          [newPolicy.value.username]: {
            username: newPolicy.value.username,
            memory_limit_gb: parseFloat(newPolicy.value.memoryLimit),
            utilization_limit_pct: parseInt(newPolicy.value.utilizationLimit),
            duration_limit_hours: 12.0,
            max_concurrent_processes: parseInt(newPolicy.value.processLimit),
            priority: 5,
            allowed_gpus: [],
            blocked_gpus: [],
            time_overrides: []
          }
        }
      })
    })
    
    if (response.ok) {
      clearPolicyForm()
      await fetchGuardData()
    }
  } catch (error) {
    console.error('Error adding user policy:', error)
  }
}

const clearPolicyForm = () => {
  newPolicy.value = {
    username: '',
    memoryLimit: 16.0,
    utilizationLimit: 80,
    processLimit: 5
  }
}

const getViolationSeverityClass = (severity) => {
  const severityMap = {
    'Low': 'bg-blue-500/20 text-blue-400',
    'Medium': 'bg-yellow-500/20 text-yellow-400',
    'High': 'bg-orange-500/20 text-orange-400',
    'Critical': 'bg-red-500/20 text-red-400'
  }
  return severityMap[severity] || 'bg-gray-500/20 text-gray-400'
}

const toggleDryRunMode = async () => {
  try {
    const response = await fetch(`${config.public.apiBase}/api/guard/toggle-dry-run`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      }
    })
    
    if (response.ok) {
      const result = await response.json()
      console.log('Dry-run toggle result:', result)
      await fetchGuardData()
    }
  } catch (error) {
    console.error('Error toggling dry-run mode:', error)
  }
}

const testPolicies = async () => {
  try {
    const response = await fetch(`${config.public.apiBase}/api/guard/test-policies`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      }
    })
    
    if (response.ok) {
      const result = await response.json()
      simulationResults.value = result
      console.log('Policy test results:', result)
    }
  } catch (error) {
    console.error('Error testing policies:', error)
  }
}

const refreshData = () => {
  fetchClusterData()
  fetchContentionData()
}

onMounted(() => {
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