# GPU Kill Dashboard

A modern, responsive dashboard for monitoring GPU clusters built with Nuxt.js and Tailwind CSS.

## Features

- **Real-time Cluster Monitoring**: Live updates via WebSocket
- **Magic Moment**: Instant visibility into GPU contention and blocked resources
- **Rogue Detection**: Security monitoring with threat detection and risk scoring
- **Guard Mode Management**: Policy enforcement with user, group, and GPU policies
- **Auto-refresh**: Automatic data updates with manual refresh controls
- **Data Persistence**: Policy data saved locally across page refreshes
- **Interactive Controls**: Toggle switches for enforcement modes
- **Policy Management**: Complete CRUD operations for User, Group, and GPU policies
- **Policy Testing**: Built-in policy simulation and testing interface

## Quick Start

1. **Start the GPU Kill Coordinator Server**:
   ```bash
   cd /path/to/gpu-kill
   ./target/release/gpukill --server --server-port 8080
   ```

2. **Start the Dashboard**:
   ```bash
   cd dashboard
   npm install  # First time only
   npm run dev
   ```

3. **Open your browser**:
   - Dashboard: http://localhost:3000
   - API: http://localhost:8080

## Dashboard Views

### Overview Page
- **Cluster Statistics**: Total nodes, GPUs, memory, and average utilization
- **Real-time Metrics**: Live indicators with auto-refresh
- **Magic Moment**: GPU contention analysis with blocked resources
- **Top Users**: Ranked list of users by GPU memory consumption
- **Node Details**: Individual node status and health information

### Detection Page
- **Threat Detection**: Real-time security monitoring
- **Risk Scoring**: Confidence-based threat assessment
- **Crypto Miner Detection**: Identifies mining software and patterns
- **Suspicious Processes**: Flags unusual process behavior
- **Resource Abuse Monitoring**: Detects excessive memory usage
- **Interactive Scanning**: Manual scan controls with loading states

### Guard Page
- **Policy Management**: User, Group, and GPU policy configuration
- **Enforcement Controls**: Soft/hard enforcement toggle switches
- **Policy Statistics**: Modern gradient cards showing policy counts
- **Visual Tables**: Clean display of all policies with action buttons
- **Modal Forms**: Intuitive policy creation with validation
- **Policy Testing**: Built-in simulation and testing interface
- **Data Persistence**: Policy data saved locally across refreshes

## Configuration

The dashboard automatically connects to the GPU Kill coordinator API. You can configure the API endpoint:

```bash
# Set custom API base URL
export API_BASE=http://your-server:8080
npm run dev
```

## Development

```bash
# Install dependencies
npm install

# Start development server
npm run dev

# Build for production
npm run build

# Preview production build
npm run preview
```

## API Integration

The dashboard connects to the GPU Kill coordinator API endpoints:

- `GET /api/cluster/snapshot` - Cluster overview data
- `GET /api/cluster/contention` - Magic Moment analysis
- `GET /api/cluster/rogue` - Rogue detection results
- `GET /api/guard/config` - Guard Mode configuration
- `GET /api/guard/status` - Guard Mode status
- `POST /api/guard/toggle-dry-run` - Toggle dry-run mode
- `POST /api/guard/test-policies` - Test policy enforcement
- `WS /ws` - WebSocket for real-time updates