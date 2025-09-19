# GPU Kill Dashboard

A modern, responsive dashboard for monitoring GPU clusters built with Nuxt.js and Tailwind CSS.

## Features

- **📊 Real-time Cluster Monitoring**: Live updates via WebSocket
- **⚡ Magic Moment**: Instant visibility into GPU contention and blocked resources
- **🎨 Beautiful UI**: Modern design with dark mode support
- **📱 Responsive**: Works on desktop, tablet, and mobile
- **🔄 Auto-refresh**: Automatic data updates every 10 seconds

## Quick Start

1. **Start the GPU Kill Coordinator Server**:
   ```bash
   cd /path/to/gpu-kill
   ./target/release/gpukill --server --server-port 8080
   ```

2. **Start the Dashboard**:
   ```bash
   cd dashboard
   npm run dev
   ```

3. **Open your browser**:
   - Dashboard: http://localhost:3000
   - API: http://localhost:8080

## Dashboard Views

### Cluster Overview
- Total nodes, GPUs, memory, and average utilization
- Real-time metrics with live indicators

### Magic Moment - GPU Contention
- **Blocked GPUs**: Shows GPUs with high utilization or memory usage
- **Top Users**: Ranked list of users by GPU memory consumption
- **Recommendations**: Automated suggestions for resource optimization

### Node Details
- Individual node status and health
- Per-GPU utilization bars and memory usage
- Process counts and last seen timestamps

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

## Architecture

- **Frontend**: Nuxt.js 3 with Vue 3 Composition API
- **Styling**: Tailwind CSS with custom components
- **Icons**: Heroicons for consistent iconography
- **Real-time**: WebSocket connection for live updates
- **Charts**: Chart.js for data visualization (ready for future enhancements)

## API Integration

The dashboard connects to the GPU Kill coordinator API endpoints:

- `GET /api/cluster/snapshot` - Cluster overview data
- `GET /api/cluster/contention` - Magic Moment analysis
- `WS /ws` - WebSocket for real-time updates

## Customization

### Adding New Metrics
1. Update the coordinator API to include new data
2. Add new metric cards in `pages/index.vue`
3. Style with Tailwind CSS classes

### Dark Mode
The dashboard includes automatic dark mode detection and manual toggle. Users can switch between light and dark themes.

### Responsive Design
The dashboard is fully responsive with breakpoints:
- Mobile: Single column layout
- Tablet: 2-column grid
- Desktop: 3-4 column grid

## Future Enhancements

- **Charts**: Historical utilization graphs
- **Alerts**: Real-time notifications for issues
- **User Management**: Role-based access control
- **Export**: Data export functionality
- **Mobile App**: PWA support for mobile devices