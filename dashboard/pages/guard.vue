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
              <h2 class="text-base font-medium text-white">Guard</h2>
              <p class="text-sm text-gray-500">Policy enforcement and monitoring</p>
            </div>
            
            <div class="flex items-center space-x-4">
              <button
                v-if="guardActiveTab === 'policies'"
                @click="testPolicies"
                class="flex items-center space-x-2 bg-transparent text-sm hover:bg-gray-500/15 border border-gray-500/10 text-gray-400 px-4 py-2 rounded-xl transition-all duration-200"
              >
                <BeakerIcon class="w-4 h-4" />
                <span>Test</span>
              </button>
              <button
                @click="toggleGuardMode"
                :class="[
                  'flex items-center space-x-2 px-4 py-2 rounded-xl font-medium transition-colors text-sm',
                  guardStatus?.enabled 
                    ? 'bg-green-400 hover:bg-green-500 text-black' 
                    : 'bg-blue-300 hover:bg-blue-400 text-black'
                ]"
              >
                <ShieldCheckIcon v-if="guardStatus?.enabled" class="w-4 h-4" />
                <ShieldExclamationIcon v-else class="w-4 h-4" />
                <span>{{ guardStatus?.enabled ? 'Disable' : 'Enable' }}</span>
              </button>
              <button
                @click="toggleDryRunMode"
                :class="[
                  'flex items-center space-x-2 px-4 py-2 rounded-xl font-medium transition-colors text-sm',
                  guardStatus?.dry_run 
                    ? 'bg-yellow-400 hover:bg-yellow-500 text-black' 
                    : 'bg-yellow-400 hover:bg-yellow-500 text-black'
                ]"
              >
                <BeakerIcon v-if="guardStatus?.dry_run" class="w-4 h-4" />
                <BoltIcon v-else class="w-4 h-4" />
                <span>{{ guardStatus?.dry_run ? 'Dry Run' : 'Enforcing' }}</span>
              </button>
              <span class="text-gray-500/20">|</span>
              <button @click="refreshGuardData" :disabled="isRefreshing" class="border border-gray-500/10 text-sm rounded-xl px-4 py-2 text-white bg-gray-500/10 hover:bg-gray-500/15 flex items-center space-x-2 disabled:opacity-50 disabled:cursor-not-allowed">
                <ArrowPathIcon :class="['w-4 h-4', isRefreshing ? 'animate-spin' : '']" />
                <span>{{ isRefreshing ? 'Refreshing...' : 'Refresh' }}</span>
              </button>
            </div>
          </div>
        </div>
      </header>

      <!-- Main Content -->
      <main class="flex-1 overflow-y-auto">
        <!-- Guard Page Tabs -->
        <div class="px-6 border-b border-gray-500/10">
          <div class="flex items-center space-x-8">
            <button
              @click="guardActiveTab = 'overview'"
              :class="[
                'relative text-sm transition-colors py-3 px-1',
                guardActiveTab === 'overview'
                  ? 'text-blue-300'
                  : 'text-gray-400 hover:text-white'
              ]"
            >
              Overview
              <div 
                v-if="guardActiveTab === 'overview'"
                class="absolute bottom-0 left-0 right-0 h-0.5 bg-blue-300"
              ></div>
            </button>
            <button
              @click="guardActiveTab = 'policies'"
              :class="[
                'relative text-sm transition-colors py-3 px-1',
                guardActiveTab === 'policies'
                  ? 'text-blue-300'
                  : 'text-gray-400 hover:text-white'
              ]"
            >
              Policy
              <div 
                v-if="guardActiveTab === 'policies'"
                class="absolute bottom-0 left-0 right-0 h-0.5 bg-blue-300"
              ></div>
            </button>
          </div>
        </div>

        <!-- Overview Tab -->
        <div v-if="guardActiveTab === 'overview'" class="divide-y divide-gray-500/10">
          <div v-if="guardStatus" class="divide-y divide-gray-500/10">
            <!-- Status Overview -->
            <div class="p-6">
              <div class="grid grid-cols-1 md:grid-cols-4 gap-4">
                <div class="flex items-center space-x-3">
                  <div class="w-10 h-10 rounded-full flex items-center justify-center" :class="guardStatus.enabled ? 'bg-green-400/5' : 'bg-red-500/5'">
                    <CheckCircleIcon v-if="guardStatus.enabled" class="w-5 h-5 text-green-400" />
                    <XCircleIcon v-else class="w-5 h-5 text-red-400" />
                  </div>
                  <div>
                    <div class="text-sm text-gray-400">Status</div>
                    <div class="text-xs text-gray-300">
                      {{ guardStatus.enabled ? 'Enabled' : 'Disabled' }}
                    </div>
                  </div>
                </div>
                <div class="flex items-center space-x-3">
                  <div class="w-10 h-10 rounded-full flex items-center justify-center" :class="guardStatus.dry_run ? 'bg-yellow-400/5' : 'bg-orange-400/5'">
                    <BeakerIcon v-if="guardStatus.dry_run" class="w-5 h-5 text-yellow-400" />
                    <BoltIcon v-else class="w-5 h-5 text-orange-400" />
                  </div>
                  <div>
                    <div class="text-sm text-gray-400">Mode</div>
                    <div class="text-xs text-gray-300">
                      {{ guardStatus.dry_run ? 'Dry Run' : 'Enforcing' }}
                    </div>
                  </div>
                </div>
                <div class="flex items-center space-x-3">
                  <div class="w-10 h-10 rounded-full flex items-center justify-center bg-red-500/5">
                    <ExclamationTriangleIcon class="w-5 h-5 text-red-500" />
                  </div>
                  <div>
                    <div class="text-sm text-gray-400">Violations</div>
                    <div class="text-xs text-gray-300">{{ guardStatus.total_violations }} Total</div>
                  </div>
                </div>
                <div class="flex items-center space-x-3">
                  <div class="w-10 h-10 rounded-full flex items-center justify-center bg-blue-300/5">
                    <DocumentTextIcon class="w-5 h-5 text-blue-300" />
                  </div>
                  <div>
                    <div class="text-sm text-gray-400">Policies</div>
                    <div class="text-xs text-gray-300">{{ guardStatus.user_policy_count }} Active</div>
                  </div>
                </div>
              </div>
            </div>

            <!-- Enforcement Settings -->
            <div class="p-6">
              <h3 class="text-xs font-medium text-gray-500 uppercase mb-3">Enforcement</h3>
              <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div class="flex items-center justify-between border border-gray-500/10 rounded-xl p-3 bg-gray-500/5">
                  <div class="flex items-center space-x-3">
                    <span class="text-gray-300 text-sm">Soft</span>
                    <span class="text-xs text-gray-500">Warnings only</span>
                  </div>
                  <button
                    @click="toggleSoftEnforcement"
                    :class="[
                      'relative inline-flex h-6 w-11 items-center rounded-full transition-colors focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 focus:ring-offset-gray-900',
                      guardStatus.soft_enforcement ? 'bg-green-400' : 'bg-gray-500/20'
                    ]"
                  >
                    <span
                      :class="[
                        'inline-block h-4 w-4 transform rounded-full bg-white transition-transform',
                        guardStatus.soft_enforcement ? 'translate-x-6' : 'translate-x-1'
                      ]"
                    />
                  </button>
                </div>
                <div class="flex items-center justify-between border border-gray-500/10 rounded-xl p-3 bg-gray-500/5">
                  <div class="flex items-center space-x-3">
                    <span class="text-gray-300 text-sm">Hard</span>
                    <span class="text-xs text-gray-500">Terminate processes</span>
                  </div>
                  <button
                    @click="toggleHardEnforcement"
                    :class="[
                      'relative inline-flex h-6 w-11 items-center rounded-full transition-colors focus:outline-none focus:ring-2 focus:ring-red-500 focus:ring-offset-2 focus:ring-offset-gray-900',
                      guardStatus.hard_enforcement ? 'bg-red-400' : 'bg-gray-500/20'
                    ]"
                  >
                    <span
                      :class="[
                        'inline-block h-4 w-4 transform rounded-full bg-white transition-transform',
                        guardStatus.hard_enforcement ? 'translate-x-6' : 'translate-x-1'
                      ]"
                    />
                  </button>
                </div>
              </div>
              <div class="mt-3 text-xs text-gray-500">
                <p>• <strong>Soft:</strong> Send warnings and notifications when policies are violated</p>
                <p>• <strong>Hard:</strong> Terminate processes that violate policies (use with caution)</p>
              </div>
            </div>

            <!-- Recent Violations -->
            <div v-if="guardStatus.recent_violations?.length > 0" class="p-6">
              <h3 class="text-xs font-medium text-gray-500 uppercase mb-3">Recent Violations</h3>
              <div class="space-y-2">
                <div v-for="(violation, index) in guardStatus.recent_violations.slice(0, 5)" :key="index" 
                    class="bg-gray-500/5  border border-gray-500/10 rounded-xl p-3">
                  <div class="flex justify-between items-start mb-2">
                    <div>
                      <p class="font-medium text-white">{{ violation.user }} - {{ violation.violation_type }}</p>
                      <p class="text-sm text-gray-300">{{ violation.message }}</p>
                    </div>
                    <span :class="[
                      'text-xs font-medium px-2 py-1 rounded',
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
            <div v-if="guardStatus.recent_warnings?.length > 0" class="p-6">
              <h3 class="text-xs font-medium text-gray-500 uppercase mb-3">Recent Warnings</h3>
              <div class="space-y-2">
                <div v-for="(warning, index) in guardStatus.recent_warnings.slice(0, 5)" :key="index" 
                    class="bg-gray-500/5  border border-gray-500/10 rounded-xl p-3">
                  <div class="flex justify-between items-start mb-2">
                    <div>
                      <p class="font-medium text-white">{{ warning.user }} - {{ warning.warning_type }}</p>
                      <p class="text-sm text-gray-300">{{ warning.message }}</p>
                    </div>
                    <span class="text-xs font-medium text-yellow-400">
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
                class="bg-green-500/5 border border-green-500/10 rounded-xl p-4 text-center">
              <div class="text-green-400 text-lg font-bold mb-2">✅ All Clear!</div>
              <p class="text-gray-300">No recent policy violations or warnings.</p>
            </div>



          </div>

           <!-- Empty State -->
           <div v-else class="text-center text-gray-500 text-sm py-8 my-6 max-w-sm mx-auto bg-gray-500/5 rounded-xl border border-gray-500/10 p-4">
             <div class="text-center py-8">
               <div class="w-12 h-12 bg-gray-500/10 rounded-full flex items-center justify-center mx-auto mb-4">
                 <ShieldCheckIcon class="w-6 h-6 text-gray-500" />
               </div>
               <div class="flex items-center text-lg text-white justify-center space-x-2 mb-2">
                 Guard Mode
               </div>
               <p class="text-gray-400">Click "Enable" to activate policy enforcement.</p>
             </div>
           </div>
        </div>

        <!-- Policies Tab -->
        <div v-if="guardActiveTab === 'policies'">
          <div v-if="guardStatus" class="">

            <!-- Policy Statistics -->
            <div class="grid grid-cols-1 md:grid-cols-3 gap-6 p-6 border-b border-gray-500/10">
              <!-- User Policies Card -->
              <div class="relative bg-gradient-to-br from-blue-500/10 via-blue-400/5 to-transparent border border-blue-400/20 rounded-2xl p-6 overflow-hidden group hover:border-blue-400/30 transition-all duration-300">
                <div class="absolute inset-0 bg-gradient-to-br from-blue-500/5 to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-300"></div>
                <div class="relative z-10">
                  <div class="flex items-center justify-between mb-4">
                    <div>
                      <h3 class="text-lg font-semibold text-white mb-1">User</h3>
                      <p class="text-sm text-blue-300/70">Individual user limits</p>
                    </div>
                    <div class="h-12 w-12 bg-blue-400/20 rounded-xl flex items-center justify-center backdrop-blur-sm group-hover:bg-blue-400/30 transition-colors duration-300">
                      <UserIcon class="h-6 w-6 text-blue-400" />
                    </div>
                  </div>
                  <div class="flex items-baseline space-x-2 mb-3">
                    <div class="text-4xl font-bold text-white">{{ guardStatus.user_policy_count || 0 }}</div>
                    <div class="text-sm text-blue-300/70">active</div>
                  </div>
                  <div class="flex items-center text-sm text-blue-300/70">
                    <div class="w-2 h-2 bg-blue-400 rounded-full mr-2 animate-pulse"></div>
                    <span>Enforcing memory & process limits</span>
                  </div>
                </div>
              </div>
              
              <!-- Group Policies Card -->
              <div class="relative bg-gradient-to-br from-purple-500/10 via-purple-400/5 to-transparent border border-purple-400/20 rounded-2xl p-6 overflow-hidden group hover:border-purple-400/30 transition-all duration-300">
                <div class="absolute inset-0 bg-gradient-to-br from-purple-500/5 to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-300"></div>
                <div class="relative z-10">
                  <div class="flex items-center justify-between mb-4">
                    <div>
                      <h3 class="text-lg font-semibold text-white mb-1">Group</h3>
                      <p class="text-sm text-purple-300/70">Team-based restrictions</p>
                    </div>
                    <div class="h-12 w-12 bg-purple-400/20 rounded-xl flex items-center justify-center backdrop-blur-sm group-hover:bg-purple-400/30 transition-colors duration-300">
                      <UserGroupIcon class="h-6 w-6 text-purple-400" />
                    </div>
                  </div>
                  <div class="flex items-baseline space-x-2 mb-3">
                    <div class="text-4xl font-bold text-white">{{ guardStatus.group_policy_count || 0 }}</div>
                    <div class="text-sm text-purple-300/70">active</div>
                  </div>
                  <div class="flex items-center text-sm text-purple-300/70">
                    <div class="w-2 h-2 bg-purple-400 rounded-full mr-2 animate-pulse"></div>
                    <span>Managing team resource allocation</span>
                  </div>
                </div>
              </div>
              
              <!-- GPU Policies Card -->
              <div class="relative bg-gradient-to-br from-orange-500/10 via-orange-400/5 to-transparent border border-orange-400/20 rounded-2xl p-6 overflow-hidden group hover:border-orange-400/30 transition-all duration-300">
                <div class="absolute inset-0 bg-gradient-to-br from-orange-500/5 to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-300"></div>
                <div class="relative z-10">
                  <div class="flex items-center justify-between mb-4">
                    <div>
                      <h3 class="text-lg font-semibold text-white mb-1">GPU</h3>
                      <p class="text-sm text-orange-300/70">Hardware-specific rules</p>
                    </div>
                    <div class="h-12 w-12 bg-orange-400/20 rounded-xl flex items-center justify-center backdrop-blur-sm group-hover:bg-orange-400/30 transition-colors duration-300">
                      <CpuChipIcon class="h-6 w-6 text-orange-400" />
                    </div>
                  </div>
                  <div class="flex items-baseline space-x-2 mb-3">
                    <div class="text-4xl font-bold text-white">{{ guardStatus.gpu_policy_count || 0 }}</div>
                    <div class="text-sm text-orange-300/70">active</div>
                  </div>
                  <div class="flex items-center text-sm text-orange-300/70">
                    <div class="w-2 h-2 bg-orange-400 rounded-full mr-2 animate-pulse"></div>
                    <span>Controlling GPU access & usage</span>
                  </div>
                </div>
              </div>
            </div>

            <!-- Simulation Results -->
            <div v-if="simulationResults" class="p-6 border-b border-gray-500/10">
              <h3 class="text-xs font-medium text-gray-500 uppercase mb-3">Test Results</h3>
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
                  <div class="text-2xl font-bold text-blue-300 mb-1">
                    {{ simulationResults.summary?.action_count || 0 }}
                  </div>
                  <div class="text-sm text-gray-400">Actions</div>
                </div>
              </div>
              
              <div v-if="simulationResults.simulation_result?.violations?.length > 0" class="mb-4">
                <h4 class="text-md font-bold text-red-400 mb-2">Simulated Violations:</h4>
                <div class="space-y-2">
                  <div v-for="(violation, index) in simulationResults.simulation_result.violations.slice(0, 3)" :key="index" 
                      class="bg-gray-500/5  border border-gray-500/10 rounded-xl p-2">
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
                <h4 class="text-md font-bold text-blue-300 mb-2">Simulated Actions:</h4>
                <div class="space-y-1">
                  <div v-for="(action, index) in simulationResults.simulation_result.actions_taken.slice(0, 3)" :key="index" 
                      class="bg-blue-300/5 rounded-lg p-2">
                    <p class="text-sm text-white">{{ action.action_type }}: {{ action.message }}</p>
                  </div>
                </div>
              </div>

              <div v-if="!simulationResults.simulation_result?.violations?.length && !simulationResults.simulation_result?.warnings?.length"
                  class="text-center mt-5">
                <div class="text-green-400 text-lg font-bold mb-2">No Issues Found!</div>
                <p class="text-gray-400">All policies passed the simulation test.</p>
              </div>
            </div>

            <!-- Policy Type Tabs -->
            <div class="px-6 border-b border-gray-500/10">
              <div class="flex items-center space-x-8">
                <button
                  @click="policyTypeTab = 'user'"
                  :class="[
                    'relative text-sm transition-colors py-3 px-1',
                    policyTypeTab === 'user'
                      ? 'text-blue-300'
                      : 'text-gray-400 hover:text-white'
                  ]"
                >
                  User
                  <div 
                    v-if="policyTypeTab === 'user'"
                    class="absolute bottom-0 left-0 right-0 h-0.5 bg-blue-300"
                  ></div>
                </button>
                <button
                  @click="policyTypeTab = 'group'"
                  :class="[
                    'relative text-sm transition-colors py-3 px-1',
                    policyTypeTab === 'group'
                      ? 'text-purple-300'
                      : 'text-gray-400 hover:text-white'
                  ]"
                >
                  Group
                  <div 
                    v-if="policyTypeTab === 'group'"
                    class="absolute bottom-0 left-0 right-0 h-0.5 bg-purple-300"
                  ></div>
                </button>
                <button
                  @click="policyTypeTab = 'gpu'"
                  :class="[
                    'relative text-sm transition-colors py-3 px-1',
                    policyTypeTab === 'gpu'
                      ? 'text-orange-300'
                      : 'text-gray-400 hover:text-white'
                  ]"
                >
                  GPU
                  <div 
                    v-if="policyTypeTab === 'gpu'"
                    class="absolute bottom-0 left-0 right-0 h-0.5 bg-orange-300"
                  ></div>
                </button>
              </div>
            </div>

            <!-- User Policies Tab Content -->
            <div v-if="policyTypeTab === 'user'">
              <div class="flex justify-between items-center px-6 py-3">
                <h4 class="text-xs uppercase text-gray-500 font-medium">User Policies</h4>
                <div class="flex items-center space-x-3">
                  <button
                    @click="openUserPolicyEditor"
                    class="bg-gray-500/10 hover:bg-gray-500/15 border border-gray-500/10 text-white px-3 py-2 rounded-xl transition-colors text-sm flex items-center space-x-2"
                  >
                    <PlusIcon class="w-4 h-4" />
                    <span>Create</span>
                  </button>
                </div>
              </div>
              
              <!-- Empty State for User Policies -->
              <div v-if="!guardStatus.user_policies || Object.keys(guardStatus.user_policies).length === 0" 
                   class="text-center text-gray-500 text-sm py-12 mb-6 mx-auto max-w-sm bg-gray-500/5 rounded-xl border border-gray-500/10">
                <div class="w-12 h-12 bg-gray-500/10 rounded-full flex items-center justify-center mx-auto mb-4">
                  <PlusIcon class="w-6 h-6 text-gray-500" />
                </div>
                <div class="text-lg text-white mb-2">No User Policies</div>
                <p class="text-gray-400">Create your first user policy to get started.</p>
              </div>
              
              <!-- User Policies Table -->
              <div v-else class="overflow-hidden border-t border-gray-500/10">
                <div class="overflow-x-auto">
                  <table class="min-w-full divide-y divide-gray-500/10">
                    <thead class="bg-gray-500/5">
                      <tr>
                        <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-gray-400 uppercase tracking-wider">
                          User
                        </th>
                        <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-gray-400 uppercase tracking-wider">
                          Memory Limit
                        </th>
                        <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-gray-400 uppercase tracking-wider">
                          Utilization
                        </th>
                        <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-gray-400 uppercase tracking-wider">
                          Process Limit
                        </th>
                        <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-gray-400 uppercase tracking-wider">
                          Duration
                        </th>
                        <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-gray-400 uppercase tracking-wider">
                          Status
                        </th>
                        <th scope="col" class="relative px-6 py-3">
                          <span class="sr-only">Actions</span>
                        </th>
                      </tr>
                    </thead>
                    <tbody class="bg-transparent divide-y divide-gray-500/10">
                      <tr v-for="(policy, username) in guardStatus.user_policies" :key="username" 
                          class="hover:bg-gray-500/5 transition-colors">
                        <td class="px-6 py-4 whitespace-nowrap">
                          <div class="flex items-center">
                            <div class="flex-shrink-0 h-8 w-8">
                              <div class="h-8 w-8 rounded-full bg-blue-300/10 flex items-center justify-center">
                                <span class="text-sm font-medium text-blue-300">
                                  {{ username.charAt(0).toUpperCase() }}
                                </span>
                              </div>
                            </div>
                            <div class="ml-4">
                              <div class="text-sm font-medium text-white">{{ username }}</div>
                              <div class="text-sm text-gray-400">User Policy</div>
                            </div>
                          </div>
                        </td>
                        <td class="px-6 py-4 whitespace-nowrap">
                          <div class="text-sm text-white">{{ policy.memory_limit_gb }}GB</div>
                        </td>
                        <td class="px-6 py-4 whitespace-nowrap">
                          <div class="text-sm text-white">{{ policy.utilization_limit_pct }}%</div>
                        </td>
                        <td class="px-6 py-4 whitespace-nowrap">
                          <div class="text-sm text-white">{{ policy.max_concurrent_processes }}</div>
                        </td>
                        <td class="px-6 py-4 whitespace-nowrap">
                          <div class="text-sm text-white">{{ policy.duration_limit_hours }}h</div>
                        </td>
                        <td class="px-6 py-4 whitespace-nowrap">
                          <span class="inline-flex px-2 py-1 text-xs font-medium rounded-full bg-blue-300/10 text-blue-300">
                            Active
                          </span>
                        </td>
                        <td class="px-6 py-4 whitespace-nowrap text-right text-sm font-medium">
                          <button
                            @click="deleteUserPolicy(username)"
                            class="text-gray-500 hover:text-red-400 transition-colors p-1 rounded-lg hover:bg-red-400/10"
                            title="Delete Policy"
                          >
                            <TrashIcon class="w-4 h-4" />
                          </button>
                        </td>
                      </tr>
                    </tbody>
                  </table>
                </div>
              </div>
            </div>

            <!-- Group Policies Tab Content -->
            <div v-if="policyTypeTab === 'group'">
              <div class="flex justify-between items-center px-6 py-3">
                <h4 class="text-xs uppercase text-gray-500 font-medium">Group Policies</h4>
                <div class="flex items-center space-x-3">
                  <button
                    @click="openGroupPolicyEditor"
                    class="bg-gray-500/10 hover:bg-gray-500/15 border border-gray-500/10 text-white px-3 py-2 rounded-xl transition-colors text-sm flex items-center space-x-2"
                  >
                    <PlusIcon class="w-4 h-4" />
                    <span>Create</span>
                  </button>
                </div>
              </div>
              
              <!-- Empty State for Group Policies -->
              <div v-if="!guardStatus.group_policies || Object.keys(guardStatus.group_policies).length === 0" 
                   class="text-center text-gray-500 text-sm py-12 mb-6 mx-auto max-w-sm bg-gray-500/5 rounded-xl border border-gray-500/10">
                <div class="w-12 h-12 bg-gray-500/10 rounded-full flex items-center justify-center mx-auto mb-4">
                  <PlusIcon class="w-6 h-6 text-gray-500" />
                </div>
                <div class="text-lg text-white mb-2">No Group Policies</div>
                <p class="text-gray-400">Create your first group policy to get started.</p>
              </div>
              
              <!-- Group Policies Table -->
              <div v-else class="overflow-hidden border-t border-gray-500/10">
                <div class="overflow-x-auto">
                  <table class="min-w-full divide-y divide-gray-500/10">
                    <thead class="bg-gray-500/5">
                      <tr>
                        <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-gray-400 uppercase tracking-wider">
                          Group
                        </th>
                        <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-gray-400 uppercase tracking-wider">
                          Total Memory
                        </th>
                        <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-gray-400 uppercase tracking-wider">
                          Utilization
                        </th>
                        <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-gray-400 uppercase tracking-wider">
                          Process Limit
                        </th>
                        <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-gray-400 uppercase tracking-wider">
                          Members
                        </th>
                        <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-gray-400 uppercase tracking-wider">
                          Status
                        </th>
                        <th scope="col" class="relative px-6 py-3">
                          <span class="sr-only">Actions</span>
                        </th>
                      </tr>
                    </thead>
                    <tbody class="bg-transparent divide-y divide-gray-500/10">
                      <tr v-for="(policy, groupName) in guardStatus.group_policies" :key="groupName" 
                          class="hover:bg-gray-500/5 transition-colors">
                        <td class="px-6 py-4 whitespace-nowrap">
                          <div class="flex items-center">
                            <div class="flex-shrink-0 h-8 w-8">
                              <div class="h-8 w-8 rounded-full bg-purple-400/10 flex items-center justify-center">
                                <span class="text-sm font-medium text-purple-400">
                                  {{ groupName.charAt(0).toUpperCase() }}
                                </span>
                              </div>
                            </div>
                            <div class="ml-4">
                              <div class="text-sm font-medium text-white">{{ groupName }}</div>
                              <div class="text-sm text-gray-400">Group Policy</div>
                            </div>
                          </div>
                        </td>
                        <td class="px-6 py-4 whitespace-nowrap">
                          <div class="text-sm text-white">{{ policy.total_memory_limit_gb }}GB</div>
                        </td>
                        <td class="px-6 py-4 whitespace-nowrap">
                          <div class="text-sm text-white">{{ policy.total_utilization_limit_pct }}%</div>
                        </td>
                        <td class="px-6 py-4 whitespace-nowrap">
                          <div class="text-sm text-white">{{ policy.max_concurrent_processes }}</div>
                        </td>
                        <td class="px-6 py-4 whitespace-nowrap">
                          <div v-if="policy.members && policy.members.length > 0" class="text-sm text-white">
                            <div v-for="(member, index) in policy.members.slice(0, 2)" :key="index" class="inline-block">
                              <span class="bg-purple-400/10 text-purple-300 px-2 py-1 rounded text-xs mr-1">{{ member }}</span>
                            </div>
                            <span v-if="policy.members.length > 2" class="text-xs text-gray-400">+{{ policy.members.length - 2 }} more</span>
                          </div>
                          <div v-else class="text-sm text-gray-400">0</div>
                        </td>
                        <td class="px-6 py-4 whitespace-nowrap">
                          <span class="inline-flex px-2 py-1 text-xs font-medium rounded-full bg-purple-400/10 text-purple-400">
                            Active
                          </span>
                        </td>
                        <td class="px-6 py-4 whitespace-nowrap text-right text-sm font-medium">
                          <button
                            @click="deleteGroupPolicy(groupName)"
                            class="text-gray-500 hover:text-red-400 transition-colors p-1 rounded-lg hover:bg-red-400/10"
                            title="Delete Policy"
                          >
                            <TrashIcon class="w-4 h-4" />
                          </button>
                        </td>
                      </tr>
                    </tbody>
                  </table>
                </div>
              </div>
            </div>

            <!-- GPU Policies Tab Content -->
            <div v-if="policyTypeTab === 'gpu'">
              <div class="flex justify-between items-center px-6 py-3">
                <h4 class="text-xs uppercase text-gray-500 font-medium">GPU Policies</h4>
                <div class="flex items-center space-x-3">
                  <button
                    @click="openGpuPolicyEditor"
                    class="bg-gray-500/10 hover:bg-gray-500/15 border border-gray-500/10 text-white px-3 py-2 rounded-xl transition-colors text-sm flex items-center space-x-2"
                  >
                    <PlusIcon class="w-4 h-4" />
                    <span>Create</span>
                  </button>
                </div>
              </div>
              
              <!-- Empty State for GPU Policies -->
              <div v-if="!guardStatus.gpu_policies || Object.keys(guardStatus.gpu_policies).length === 0" 
                   class="text-center text-gray-500 text-sm py-12 mb-6 mx-auto max-w-sm bg-gray-500/5 rounded-xl border border-gray-500/10">
                <div class="w-12 h-12 bg-gray-500/10 rounded-full flex items-center justify-center mx-auto mb-4">
                  <PlusIcon class="w-6 h-6 text-gray-500" />
                </div>
                <div class="text-lg text-white mb-2">No GPU Policies</div>
                <p class="text-gray-400">Create your first GPU policy to get started.</p>
              </div>
              
              <!-- GPU Policies Table -->
              <div v-else class="overflow-hidden border-t border-gray-500/10">
                <div class="overflow-x-auto">
                  <table class="min-w-full divide-y divide-gray-500/10">
                    <thead class="bg-gray-500/5">
                      <tr>
                        <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-gray-400 uppercase tracking-wider">
                          GPU
                        </th>
                        <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-gray-400 uppercase tracking-wider">
                          Max Memory
                        </th>
                        <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-gray-400 uppercase tracking-wider">
                          Max Utilization
                        </th>
                        <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-gray-400 uppercase tracking-wider">
                          Reserved Memory
                        </th>
                        <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-gray-400 uppercase tracking-wider">
                          Allowed Users
                        </th>
                        <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-gray-400 uppercase tracking-wider">
                          Status
                        </th>
                        <th scope="col" class="relative px-6 py-3">
                          <span class="sr-only">Actions</span>
                        </th>
                      </tr>
                    </thead>
                    <tbody class="bg-transparent divide-y divide-gray-500/10">
                      <tr v-for="(policy, gpuIndex) in guardStatus.gpu_policies" :key="gpuIndex" 
                          class="hover:bg-gray-500/5 transition-colors">
                        <td class="px-6 py-4 whitespace-nowrap">
                          <div class="flex items-center">
                            <div class="flex-shrink-0 h-8 w-8">
                              <div class="h-8 w-8 rounded-full bg-orange-400/10 flex items-center justify-center">
                                <span class="text-sm font-medium text-orange-400">
                                  {{ gpuIndex }}
                                </span>
                              </div>
                            </div>
                            <div class="ml-4">
                              <div class="text-sm font-medium text-white">GPU {{ gpuIndex }}</div>
                              <div class="text-sm text-gray-400">GPU Policy</div>
                            </div>
                          </div>
                        </td>
                        <td class="px-6 py-4 whitespace-nowrap">
                          <div class="text-sm text-white">{{ policy.max_memory_gb }}GB</div>
                        </td>
                        <td class="px-6 py-4 whitespace-nowrap">
                          <div class="text-sm text-white">{{ policy.max_utilization_pct }}%</div>
                        </td>
                        <td class="px-6 py-4 whitespace-nowrap">
                          <div class="text-sm text-white">{{ policy.reserved_memory_gb }}GB</div>
                        </td>
                        <td class="px-6 py-4 whitespace-nowrap">
                          <div v-if="policy.allowed_users && policy.allowed_users.length > 0" class="text-sm text-white">
                            <div v-for="(user, index) in policy.allowed_users.slice(0, 2)" :key="index" class="inline-block">
                              <span class="bg-orange-400/10 text-orange-300 px-2 py-1 rounded text-xs mr-1">{{ user }}</span>
                            </div>
                            <span v-if="policy.allowed_users.length > 2" class="text-xs text-gray-400">+{{ policy.allowed_users.length - 2 }} more</span>
                          </div>
                          <div v-else class="text-sm text-gray-400">All</div>
                        </td>
                        <td class="px-6 py-4 whitespace-nowrap">
                          <span class="inline-flex px-2 py-1 text-xs font-medium rounded-full bg-orange-400/10 text-orange-400">
                            Active
                          </span>
                        </td>
                        <td class="px-6 py-4 whitespace-nowrap text-right text-sm font-medium">
                          <button
                            @click="deleteGpuPolicy(gpuIndex)"
                            class="text-gray-500 hover:text-red-400 transition-colors p-1 rounded-lg hover:bg-red-400/10"
                            title="Delete Policy"
                          >
                            <TrashIcon class="w-4 h-4" />
                          </button>
                        </td>
                      </tr>
                    </tbody>
                  </table>
                </div>
              </div>
            </div>
          </div>


          <!-- Empty State for No Guard Status -->
          <div v-else class="text-center text-gray-500 text-sm py-8 my-6 max-w-sm mx-auto bg-gray-500/5 rounded-xl border border-gray-500/10 p-4">
            <div class="text-center py-8">
              <div class="w-12 h-12 bg-gray-500/10 rounded-full flex items-center justify-center mx-auto mb-4">
                <ShieldCheckIcon class="w-6 h-6 text-gray-500" />
              </div>
              <div class="flex items-center text-lg text-white justify-center space-x-2 mb-2">
                Guard Mode
              </div>
              <p class="text-gray-400">Click "Enable" to activate policy enforcement.</p>
            </div>
          </div>
        </div>
      </main>
    </div>
  </div>

  <!-- Policy Creation Modal -->
  <div v-if="showPolicyEditor" class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
    <div class="bg-[#0b0b10] rounded-xl border border-gray-500/10 p-6 w-full max-w-md mx-4">
      <div class="flex justify-between items-center mb-6">
        <h3 class="text-lg font-medium text-white">
          {{ policyEditorType === 'user' ? 'Create User Policy' : 
             policyEditorType === 'group' ? 'Create Group Policy' : 
             'Create GPU Policy' }}
        </h3>
        <button
          @click="showPolicyEditor = false"
          :disabled="isCreatingPolicy"
          class="text-gray-400 hover:text-white transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
        >
          <XMarkIcon class="w-5 h-5" />
        </button>
      </div>
      
      <div class="space-y-4">
        <!-- User Policy Form -->
        <div v-if="policyEditorType === 'user'" class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-white mb-2">Username</label>
            <input
              v-model="newPolicy.username"
              type="text"
              placeholder="Enter username"
              class="w-full px-3 py-2 text-sm bg-gray-500/5 border border-gray-500/10 rounded-xl text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-gray-500/10"
            />
          </div>
          
          <div>
            <label class="block text-sm font-medium text-white mb-2">Memory Limit (GB)</label>
            <input
              v-model="newPolicy.memoryLimit"
              type="number"
              step="0.1"
              placeholder="16.0"
              class="w-full px-3 py-2 text-sm bg-gray-500/5 border border-gray-500/10 rounded-xl text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-gray-500/10"
            />
          </div>
          
          <div>
            <label class="block text-sm font-medium text-white mb-2">Utilization Limit (%)</label>
            <input
              v-model="newPolicy.utilizationLimit"
              type="number"
              step="1"
              placeholder="80"
              class="w-full px-3 py-2 text-sm bg-gray-500/5 border border-gray-500/10 rounded-xl text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-gray-500/10"
            />
          </div>
          
          <div>
            <label class="block text-sm font-medium text-white mb-2">Process Limit</label>
            <input
              v-model="newPolicy.processLimit"
              type="number"
              step="1"
              placeholder="5"
              class="w-full px-3 py-2 text-sm bg-gray-500/5 border border-gray-500/10 rounded-xl text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-gray-500/10"
            />
          </div>
        </div>

        <!-- Group Policy Form -->
        <div v-if="policyEditorType === 'group'" class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-white mb-2">Group Name</label>
            <input
              v-model="newGroupPolicy.groupName"
              type="text"
              placeholder="Enter group name"
              class="w-full px-3 py-2 text-sm bg-gray-500/5 border border-gray-500/10 rounded-xl text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-gray-500/10"
            />
          </div>
          
          <div>
            <label class="block text-sm font-medium text-white mb-2">Total Memory Limit (GB)</label>
            <input
              v-model="newGroupPolicy.totalMemoryLimit"
              type="number"
              step="0.1"
              placeholder="32.0"
              class="w-full px-3 py-2 text-sm bg-gray-500/5 border border-gray-500/10 rounded-xl text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-gray-500/10"
            />
          </div>
          
          <div>
            <label class="block text-sm font-medium text-white mb-2">Total Utilization Limit (%)</label>
            <input
              v-model="newGroupPolicy.totalUtilizationLimit"
              type="number"
              step="1"
              placeholder="80"
              class="w-full px-3 py-2 text-sm bg-gray-500/5 border border-gray-500/10 rounded-xl text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-gray-500/10"
            />
          </div>
          
          <div>
            <label class="block text-sm font-medium text-white mb-2">Max Concurrent Processes</label>
            <input
              v-model="newGroupPolicy.maxConcurrentProcesses"
              type="number"
              step="1"
              placeholder="10"
              class="w-full px-3 py-2 text-sm bg-gray-500/5 border border-gray-500/10 rounded-xl text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-gray-500/10"
            />
          </div>
          
          <div>
            <label class="block text-sm font-medium text-white mb-2">Group Members (comma-separated)</label>
            <input
              v-model="newGroupPolicy.membersInput"
              type="text"
              placeholder="user1, user2, user3"
              class="w-full px-3 py-2 text-sm bg-gray-500/5 border border-gray-500/10 rounded-xl text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-gray-500/10"
            />
            <p class="text-xs text-gray-400 mt-1">Enter usernames separated by commas</p>
          </div>
        </div>

        <!-- GPU Policy Form -->
        <div v-if="policyEditorType === 'gpu'" class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-white mb-2">GPU Index</label>
            <input
              v-model="newGpuPolicy.gpuIndex"
              type="number"
              step="1"
              placeholder="0"
              class="w-full px-3 py-2 text-sm bg-gray-500/5 border border-gray-500/10 rounded-xl text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-gray-500/10"
            />
          </div>
          
          <div>
            <label class="block text-sm font-medium text-white mb-2">Max Memory (GB)</label>
            <input
              v-model="newGpuPolicy.maxMemory"
              type="number"
              step="0.1"
              placeholder="24.0"
              class="w-full px-3 py-2 text-sm bg-gray-500/5 border border-gray-500/10 rounded-xl text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-gray-500/10"
            />
          </div>
          
          <div>
            <label class="block text-sm font-medium text-white mb-2">Max Utilization (%)</label>
            <input
              v-model="newGpuPolicy.maxUtilization"
              type="number"
              step="1"
              placeholder="90"
              class="w-full px-3 py-2 text-sm bg-gray-500/5 border border-gray-500/10 rounded-xl text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-gray-500/10"
            />
          </div>
          
          <div>
            <label class="block text-sm font-medium text-white mb-2">Reserved Memory (GB)</label>
            <input
              v-model="newGpuPolicy.reservedMemory"
              type="number"
              step="0.1"
              placeholder="2.0"
              class="w-full px-3 py-2 text-sm bg-gray-500/5 border border-gray-500/10 rounded-xl text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-gray-500/10"
            />
          </div>
          
          <div>
            <label class="block text-sm font-medium text-white mb-2">Allowed Users (comma-separated)</label>
            <input
              v-model="newGpuPolicy.allowedUsersInput"
              type="text"
              placeholder="user1, user2, user3"
              class="w-full px-3 py-2 text-sm bg-gray-500/5 border border-gray-500/10 rounded-xl text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-gray-500/10"
            />
            <p class="text-xs text-gray-400 mt-1">Enter usernames separated by commas (leave empty to allow all users)</p>
          </div>
        </div>
        
        <div class="flex space-x-3 pt-4">
            <button
              @click="clearPolicyForm"
              :disabled="isCreatingPolicy"
              class="flex-1 bg-gray-500/5 text-sm hover:bg-gray-500/10 border border-gray-500/10 text-white px-3 py-2 rounded-xl font-medium transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            >
              Clear
            </button>
            <button
              @click="createPolicy"
              :disabled="isCreatingPolicy"
              class="flex-1 bg-blue-300 text-sm hover:bg-blue-400 text-black px-3 py-2 rounded-xl font-medium transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center space-x-2"
            >
              <ArrowPathIcon v-if="isCreatingPolicy" class="w-4 h-4 animate-spin" />
              <span>{{ isCreatingPolicy ? 'Creating...' : 'Create' }}</span>
            </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted } from 'vue'
import {
  ArrowPathIcon,
  BeakerIcon,
  BoltIcon,
  CheckCircleIcon,
  XCircleIcon,
  ExclamationTriangleIcon,
  DocumentTextIcon,
  ShieldExclamationIcon,
  ShieldCheckIcon,
  XMarkIcon,
  PlusIcon,
  TrashIcon,
  CpuChipIcon,
  UserGroupIcon,
  UserIcon,
} from '@heroicons/vue/24/solid'

// Page metadata
useHead({
  title: 'Guard',
})

const config = useRuntimeConfig()
const guardStatus = ref(null)
const simulationResults = ref(null)
const showPolicyEditor = ref(false)
const policyEditorType = ref('user') // 'user', 'group', 'gpu'
const activeTab = ref('guard')
const guardActiveTab = ref('overview') // New tab system for guard page
const policyTypeTab = ref('user') // New tab system for policy types
const isRefreshing = ref(false)
const isCreatingPolicy = ref(false)
const newPolicy = ref({
  username: '',
  memoryLimit: 16.0,
  utilizationLimit: 80,
  processLimit: 5
})

const newGroupPolicy = ref({
  groupName: '',
  totalMemoryLimit: 32.0,
  totalUtilizationLimit: 80,
  maxConcurrentProcesses: 10,
  members: [],
  membersInput: ''
})

const newGpuPolicy = ref({
  gpuIndex: 0,
  maxMemory: 24.0,
  maxUtilization: 90,
  reservedMemory: 2.0,
  allowedUsers: [],
  allowedUsersInput: ''
})

const fetchGuardData = async () => {
  try {
    // Try to fetch from real API first
    const response = await fetch(`${config.public.apiBase}/api/guard/status`)
    if (response.ok) {
      const data = await response.json()
      guardStatus.value = data
      console.log('Guard status (API):', data)
      return
    }
  } catch (error) {
    console.log('API not available, using sample data:', error)
  }
  
  // Fallback to sample data if API is not available
  const testData = {
    enabled: false,
    dry_run: true,
    soft_enforcement: true,
    hard_enforcement: false,
    global: {
      enabled: false,
      dry_run: true,
      max_memory_gb: 32.0,
      max_utilization_pct: 80,
      max_duration_hours: 24.0,
      max_concurrent_processes: 10
    },
    enforcement: {
      soft_enforcement: true,
      hard_enforcement: false,
      grace_period_seconds: 300,
      max_warnings: 3
    },
    user_policies: {},
    group_policies: {},
    gpu_policies: {},
    time_policies: {},
    user_policy_count: 0,
    group_policy_count: 0,
    gpu_policy_count: 0,
    recent_violations: [
      {
        id: "violation_1",
        timestamp: new Date(Date.now() - 2 * 60 * 60 * 1000).toISOString(), // 2 hours ago
        user: "user1",
        violation_type: "MemoryLimitExceeded",
        severity: "Medium",
        details: "Process 'gpu_hog' exceeded 8GB memory limit",
        gpu_id: 0,
        process_pid: 12345
      }
    ],
    recent_warnings: [
      {
        id: "warning_1",
        timestamp: new Date(Date.now() - 30 * 60 * 1000).toISOString(), // 30 minutes ago
        user: "admin",
        warning_type: "HighUtilization",
        severity: "Low",
        details: "GPU utilization at 85% for 2 hours",
        gpu_id: 1,
        process_pid: 67890
      }
    ]
  }
  
  // Load saved policies from localStorage
  const savedPolicies = loadPoliciesFromStorage()
  if (savedPolicies.user_policies && Object.keys(savedPolicies.user_policies).length > 0) {
    testData.user_policies = savedPolicies.user_policies
    testData.user_policy_count = savedPolicies.user_policy_count
  }
  if (savedPolicies.group_policies && Object.keys(savedPolicies.group_policies).length > 0) {
    testData.group_policies = savedPolicies.group_policies
    testData.group_policy_count = savedPolicies.group_policy_count
  }
  if (savedPolicies.gpu_policies && Object.keys(savedPolicies.gpu_policies).length > 0) {
    testData.gpu_policies = savedPolicies.gpu_policies
    testData.gpu_policy_count = savedPolicies.gpu_policy_count
  }
  
  // Load enforcement settings from localStorage
  if (savedPolicies.soft_enforcement !== undefined) {
    testData.soft_enforcement = savedPolicies.soft_enforcement
  }
  if (savedPolicies.hard_enforcement !== undefined) {
    testData.hard_enforcement = savedPolicies.hard_enforcement
  }
  if (savedPolicies.enforcement) {
    testData.enforcement = { ...testData.enforcement, ...savedPolicies.enforcement }
  }
  
  guardStatus.value = testData
  console.log('Guard status (sample with saved policies):', guardStatus.value)
}

const toggleGuardMode = async () => {
  try {
    const newStatus = !guardStatus.value.enabled
    
    // Try to use real API first
    try {
      const response = await fetch(`${config.public.apiBase}/api/guard/config`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          global: {
            enabled: newStatus,
            dry_run: guardStatus.value.dry_run
          }
        })
      })
      
      if (response.ok) {
        await fetchGuardData()
        return
      }
    } catch (apiError) {
      console.log('API not available, using local toggle:', apiError)
    }
    
    // Fallback to local toggle if API is not available
    if (guardStatus.value) {
      guardStatus.value.enabled = newStatus
      guardStatus.value.global.enabled = newStatus
      console.log('Guard mode toggled locally:', newStatus ? 'Enabled' : 'Disabled')
    }
  } catch (error) {
    console.error('Error toggling guard mode:', error)
  }
}

const toggleDryRunMode = async () => {
  try {
    if (guardStatus.value) {
      // Toggle the dry run status
      guardStatus.value.dry_run = !guardStatus.value.dry_run
      guardStatus.value.global.dry_run = guardStatus.value.dry_run
      console.log('Dry-run mode toggled:', guardStatus.value.dry_run ? 'Dry Run' : 'Enforcing')
    }
    
    // Uncomment below to use real API
    // const response = await fetch(`${config.public.apiBase}/api/guard/toggle-dry-run`, {
    //   method: 'POST',
    //   headers: {
    //     'Content-Type': 'application/json',
    //   }
    // })
    // 
    // if (response.ok) {
    //   const result = await response.json()
    //   console.log('Dry-run toggle result:', result)
    //   await fetchGuardData()
    // }
  } catch (error) {
    console.error('Error toggling dry-run mode:', error)
  }
}

const toggleSoftEnforcement = async () => {
  try {
    if (guardStatus.value) {
      // Toggle soft enforcement
      guardStatus.value.soft_enforcement = !guardStatus.value.soft_enforcement
      guardStatus.value.enforcement.soft_enforcement = guardStatus.value.soft_enforcement
      console.log('Soft enforcement toggled:', guardStatus.value.soft_enforcement ? 'Enabled' : 'Disabled')
      
      // Save to localStorage
      savePoliciesToStorage()
    }
    
    // Uncomment below to use real API
    // const response = await fetch(`${config.public.apiBase}/api/guard/config`, {
    //   method: 'POST',
    //   headers: {
    //     'Content-Type': 'application/json',
    //   },
    //   body: JSON.stringify({
    //     enforcement: {
    //       soft_enforcement: guardStatus.value.soft_enforcement,
    //       hard_enforcement: guardStatus.value.hard_enforcement
    //     }
    //   })
    // })
    // 
    // if (response.ok) {
    //   const data = await response.json()
    //   guardStatus.value = data
    // }
  } catch (error) {
    console.error('Error toggling soft enforcement:', error)
  }
}

const toggleHardEnforcement = async () => {
  try {
    if (guardStatus.value) {
      // Toggle hard enforcement
      guardStatus.value.hard_enforcement = !guardStatus.value.hard_enforcement
      guardStatus.value.enforcement.hard_enforcement = guardStatus.value.hard_enforcement
      console.log('Hard enforcement toggled:', guardStatus.value.hard_enforcement ? 'Enabled' : 'Disabled')
      
      // Save to localStorage
      savePoliciesToStorage()
    }
    
    // Uncomment below to use real API
    // const response = await fetch(`${config.public.apiBase}/api/guard/config`, {
    //   method: 'POST',
    //   headers: {
    //     'Content-Type': 'application/json',
    //   },
    //   body: JSON.stringify({
    //     enforcement: {
    //       soft_enforcement: guardStatus.value.soft_enforcement,
    //       hard_enforcement: guardStatus.value.hard_enforcement
    //     }
    //   })
    // })
    // 
    // if (response.ok) {
    //   const data = await response.json()
    //   guardStatus.value = data
    // }
  } catch (error) {
    console.error('Error toggling hard enforcement:', error)
  }
}

const testPolicies = async () => {
  try {
    // For testing purposes, create sample simulation results
    const testResults = {
      timestamp: new Date().toISOString(),
      total_policies_checked: 3,
      violations_found: 1,
      warnings_found: 2,
      actions_simulated: [
        {
          action_type: "KillProcess",
          target_pid: 12345,
          reason: "Memory limit exceeded",
          severity: "Medium",
          user: "user1"
        }
      ],
      warnings: [
        {
          warning_type: "HighUtilization",
          target_pid: 67890,
          reason: "GPU utilization at 85% for 2 hours",
          severity: "Low",
          user: "admin"
        },
        {
          warning_type: "LongRunning",
          target_pid: 11111,
          reason: "Process running for 8.5 hours",
          severity: "Medium",
          user: "abuser"
        }
      ]
    }
    
    simulationResults.value = testResults
    console.log('Policy test results (test):', testResults)
    
    // Uncomment below to use real API
    // const response = await fetch(`${config.public.apiBase}/api/guard/test-policies`, {
    //   method: 'POST',
    //   headers: {
    //     'Content-Type': 'application/json',
    //   }
    // })
    // 
    // if (response.ok) {
    //   const result = await response.json()
    //   simulationResults.value = result
    //   console.log('Policy test results:', result)
    // }
  } catch (error) {
    console.error('Error testing policies:', error)
  }
}

const addUserPolicy = async () => {
  if (!newPolicy.value.username) {
    alert('Please enter a username')
    return
  }

  if (isCreatingPolicy.value) return

  isCreatingPolicy.value = true

  try {
    // Try to use real API first
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
        showPolicyEditor.value = false
        await fetchGuardData()
        console.log('Policy created successfully via API')
        return
      }
    } catch (apiError) {
      console.log('API not available, using local simulation:', apiError)
    }

    // Fallback to local simulation if API is not available
    if (guardStatus.value) {
      // Simulate adding a policy locally
      guardStatus.value.user_policy_count += 1
      guardStatus.value.user_policies = guardStatus.value.user_policies || {}
      guardStatus.value.user_policies[newPolicy.value.username] = {
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
      
      console.log('Policy created locally:', {
        username: newPolicy.value.username,
        user_policy_count: guardStatus.value.user_policy_count,
        user_policies: guardStatus.value.user_policies
      })
      
      // Save to localStorage
      savePoliciesToStorage()
      
      clearPolicyForm()
      showPolicyEditor.value = false
      console.log('Policy created locally (simulation)')
    }
  } catch (error) {
    console.error('Error adding policy:', error)
    alert('Failed to create policy. Please try again.')
  } finally {
    isCreatingPolicy.value = false
  }
}

const createPolicy = async () => {
  if (policyEditorType.value === 'user') {
    await addUserPolicy()
  } else if (policyEditorType.value === 'group') {
    await addGroupPolicy()
  } else if (policyEditorType.value === 'gpu') {
    await addGpuPolicy()
  }
}

const addGroupPolicy = async () => {
  if (!newGroupPolicy.value.groupName) {
    alert('Please enter a group name')
    return
  }

  if (isCreatingPolicy.value) return

  isCreatingPolicy.value = true

  try {
    // Simulate adding a group policy locally
    if (guardStatus.value) {
      guardStatus.value.group_policy_count += 1
      guardStatus.value.group_policies = guardStatus.value.group_policies || {}
      // Parse members from comma-separated input
      const members = newGroupPolicy.value.membersInput
        ? newGroupPolicy.value.membersInput.split(',').map(m => m.trim()).filter(m => m)
        : []

      guardStatus.value.group_policies[newGroupPolicy.value.groupName] = {
        group_name: newGroupPolicy.value.groupName,
        total_memory_limit_gb: parseFloat(newGroupPolicy.value.totalMemoryLimit),
        total_utilization_limit_pct: parseInt(newGroupPolicy.value.totalUtilizationLimit),
        max_concurrent_processes: parseInt(newGroupPolicy.value.maxConcurrentProcesses),
        priority: 5,
        allowed_gpus: [],
        blocked_gpus: [],
        members: members
      }
      
      // Save to localStorage
      savePoliciesToStorage()
      
      clearPolicyForm()
      showPolicyEditor.value = false
      console.log('Group policy created locally:', newGroupPolicy.value.groupName)
    }
  } catch (error) {
    console.error('Error adding group policy:', error)
    alert('Failed to create group policy. Please try again.')
  } finally {
    isCreatingPolicy.value = false
  }
}

const addGpuPolicy = async () => {
  if (isCreatingPolicy.value) return

  isCreatingPolicy.value = true

  try {
    // Simulate adding a GPU policy locally
    if (guardStatus.value) {
      guardStatus.value.gpu_policy_count += 1
      guardStatus.value.gpu_policies = guardStatus.value.gpu_policies || {}
      // Parse allowed users from comma-separated input
      const allowedUsers = newGpuPolicy.value.allowedUsersInput
        ? newGpuPolicy.value.allowedUsersInput.split(',').map(u => u.trim()).filter(u => u)
        : []

      guardStatus.value.gpu_policies[newGpuPolicy.value.gpuIndex] = {
        gpu_index: parseInt(newGpuPolicy.value.gpuIndex),
        max_memory_gb: parseFloat(newGpuPolicy.value.maxMemory),
        max_utilization_pct: parseInt(newGpuPolicy.value.maxUtilization),
        reserved_memory_gb: parseFloat(newGpuPolicy.value.reservedMemory),
        allowed_users: allowedUsers,
        blocked_users: [],
        priority: 5
      }
      
      // Save to localStorage
      savePoliciesToStorage()
      
      clearPolicyForm()
      showPolicyEditor.value = false
      console.log('GPU policy created locally:', newGpuPolicy.value.gpuIndex)
    }
  } catch (error) {
    console.error('Error adding GPU policy:', error)
    alert('Failed to create GPU policy. Please try again.')
  } finally {
    isCreatingPolicy.value = false
  }
}

const clearPolicyForm = () => {
  if (policyEditorType.value === 'user') {
    newPolicy.value = {
      username: '',
      memoryLimit: 16.0,
      utilizationLimit: 80,
      processLimit: 5
    }
  } else if (policyEditorType.value === 'group') {
    newGroupPolicy.value = {
      groupName: '',
      totalMemoryLimit: 32.0,
      totalUtilizationLimit: 80,
      maxConcurrentProcesses: 10,
      members: [],
      membersInput: ''
    }
  } else if (policyEditorType.value === 'gpu') {
    newGpuPolicy.value = {
      gpuIndex: 0,
      maxMemory: 24.0,
      maxUtilization: 90,
      reservedMemory: 2.0,
      allowedUsers: [],
      allowedUsersInput: ''
    }
  }
}

const deleteUserPolicy = (username) => {
  if (confirm(`Are you sure you want to delete the policy for ${username}?`)) {
    if (guardStatus.value && guardStatus.value.user_policies) {
      delete guardStatus.value.user_policies[username]
      guardStatus.value.user_policy_count = Math.max(0, guardStatus.value.user_policy_count - 1)
      
      // Save to localStorage
      savePoliciesToStorage()
      
      console.log(`Policy deleted for user: ${username}`)
    }
  }
}

const openUserPolicyEditor = () => {
  policyEditorType.value = 'user'
  showPolicyEditor.value = true
}

const openGroupPolicyEditor = () => {
  policyEditorType.value = 'group'
  showPolicyEditor.value = true
}

const openGpuPolicyEditor = () => {
  policyEditorType.value = 'gpu'
  showPolicyEditor.value = true
}

const deleteGroupPolicy = (groupName) => {
  if (confirm(`Are you sure you want to delete the policy for group ${groupName}?`)) {
    if (guardStatus.value && guardStatus.value.group_policies) {
      delete guardStatus.value.group_policies[groupName]
      guardStatus.value.group_policy_count = Math.max(0, guardStatus.value.group_policy_count - 1)
      
      // Save to localStorage
      savePoliciesToStorage()
      
      console.log(`Policy deleted for group: ${groupName}`)
    }
  }
}

const deleteGpuPolicy = (gpuIndex) => {
  if (confirm(`Are you sure you want to delete the policy for GPU ${gpuIndex}?`)) {
    if (guardStatus.value && guardStatus.value.gpu_policies) {
      delete guardStatus.value.gpu_policies[gpuIndex]
      guardStatus.value.gpu_policy_count = Math.max(0, guardStatus.value.gpu_policy_count - 1)
      
      // Save to localStorage
      savePoliciesToStorage()
      
      console.log(`Policy deleted for GPU: ${gpuIndex}`)
    }
  }
}

const setActiveTab = (tab) => {
  activeTab.value = tab
}

// localStorage functions for policy persistence
const savePoliciesToStorage = () => {
  if (guardStatus.value) {
    const policiesToSave = {
      user_policies: guardStatus.value.user_policies || {},
      user_policy_count: guardStatus.value.user_policy_count || 0,
      group_policies: guardStatus.value.group_policies || {},
      group_policy_count: guardStatus.value.group_policy_count || 0,
      gpu_policies: guardStatus.value.gpu_policies || {},
      gpu_policy_count: guardStatus.value.gpu_policy_count || 0,
      soft_enforcement: guardStatus.value.soft_enforcement,
      hard_enforcement: guardStatus.value.hard_enforcement,
      enforcement: guardStatus.value.enforcement || {}
    }
    localStorage.setItem('gpu-kill-policies', JSON.stringify(policiesToSave))
    console.log('All policies and enforcement settings saved to localStorage:', policiesToSave)
  }
}

const loadPoliciesFromStorage = () => {
  try {
    const saved = localStorage.getItem('gpu-kill-policies')
    if (saved) {
      const parsed = JSON.parse(saved)
      console.log('All policies loaded from localStorage:', parsed)
      return parsed
    }
  } catch (error) {
    console.error('Error loading policies from localStorage:', error)
  }
  return { 
    user_policies: {}, 
    user_policy_count: 0,
    group_policies: {},
    group_policy_count: 0,
    gpu_policies: {},
    gpu_policy_count: 0,
    soft_enforcement: true,
    hard_enforcement: false,
    enforcement: {
      soft_enforcement: true,
      hard_enforcement: false,
      grace_period_seconds: 300,
      max_warnings: 3
    }
  }
}

const getViolationSeverityClass = (severity) => {
  switch (severity) {
    case 'Critical':
      return 'bg-red-500/5 text-red-400'
    case 'High':
      return 'bg-orange-400/5 text-orange-400'
    case 'Medium':
      return 'bg-yellow-400/5 text-yellow-400'
    case 'Low':
      return 'bg-blue-300/5 text-blue-300'
    default:
      return 'bg-gray-500/5 text-gray-400'
  }
}

const formatTime = (timestamp) => {
  return new Date(timestamp).toLocaleString()
}

const refreshGuardData = async () => {
  if (isRefreshing.value) return
  
  isRefreshing.value = true
  try {
    await fetchGuardData()
    
    // Ensure minimum loading time of 1 second
    await new Promise(resolve => setTimeout(resolve, 1000))
  } finally {
    isRefreshing.value = false
  }
}

let refreshInterval

onMounted(() => {
  console.log('Guard page mounted, fetching data...')
  fetchGuardData()
  
  // Auto-refresh every 30 seconds
  refreshInterval = setInterval(fetchGuardData, 30000)
})

onUnmounted(() => {
  if (refreshInterval) {
    clearInterval(refreshInterval)
  }
})
</script>
