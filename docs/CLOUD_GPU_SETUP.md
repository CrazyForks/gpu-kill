# Cloud GPU Setup Guide

This guide shows how to set up GPU testing using cloud services.

## Quick Start

### Option 1: AWS EC2 with GPU

1. **Launch GPU Instance:**
   ```bash
   # Using AWS CLI
   aws ec2 run-instances \
     --image-id ami-0c02fb55956c7d316 \
     --instance-type g4dn.xlarge \
     --key-name your-key \
     --security-group-ids sg-xxxxxxxxx \
     --subnet-id subnet-xxxxxxxxx
   ```

2. **Connect and Setup:**
   ```bash
   ssh -i your-key.pem ubuntu@your-instance-ip
   curl -sSL https://raw.githubusercontent.com/treadiehq/gpu-kill/main/scripts/setup-gpu-runner.sh | bash
   ```

### Option 2: Google Cloud with GPU

1. **Create GPU Instance:**
   ```bash
   gcloud compute instances create gpu-test-runner \
     --zone=us-central1-a \
     --machine-type=n1-standard-4 \
     --accelerator=type=nvidia-tesla-t4,count=1 \
     --image-family=ubuntu-2004-lts \
     --image-project=ubuntu-os-cloud \
     --maintenance-policy=TERMINATE \
     --restart-on-failure
   ```

2. **Setup:**
   ```bash
   gcloud compute ssh gpu-test-runner --zone=us-central1-a
   curl -sSL https://raw.githubusercontent.com/treadiehq/gpu-kill/main/scripts/setup-gpu-runner.sh | bash
   ```

### Option 3: Azure with GPU

1. **Create VM:**
   ```bash
   az vm create \
     --resource-group myResourceGroup \
     --name gpu-test-vm \
     --image UbuntuLTS \
     --size Standard_NC6s_v3 \
     --admin-username azureuser \
     --generate-ssh-keys
   ```

2. **Setup:**
   ```bash
   ssh azureuser@your-vm-ip
   curl -sSL https://raw.githubusercontent.com/treadiehq/gpu-kill/main/scripts/setup-gpu-runner.sh | bash
   ```

## Cost-Effective Options

### Spot Instances
- **AWS Spot**: Up to 90% savings
- **GCP Preemptible**: Up to 80% savings
- **Azure Spot**: Up to 90% savings

### Example Spot Instance Setup (AWS):
```bash
aws ec2 request-spot-instances \
  --spot-price "0.50" \
  --instance-count 1 \
  --type "one-time" \
  --launch-specification '{
    "ImageId": "ami-0c02fb55956c7d316",
    "InstanceType": "g4dn.xlarge",
    "KeyName": "your-key",
    "SecurityGroupIds": ["sg-xxxxxxxxx"]
  }'
```

## Docker-Based Testing

### NVIDIA Docker Setup
```bash
# Install NVIDIA Docker
distribution=$(. /etc/os-release;echo $ID$VERSION_ID)
curl -s -L https://nvidia.github.io/nvidia-docker/gpgkey | sudo apt-key add -
curl -s -L https://nvidia.github.io/nvidia-docker/$distribution/nvidia-docker.list | sudo tee /etc/apt/sources.list.d/nvidia-docker.list

sudo apt-get update && sudo apt-get install -y nvidia-docker2
sudo systemctl restart docker

# Test GPU access
docker run --rm --gpus all nvidia/cuda:11.0-base nvidia-smi
```

### GPU Kill Docker Testing
```bash
# Build GPU Kill with GPU support
docker build -t gpukill:gpu .

# Run tests with GPU access
docker run --rm --gpus all gpukill:gpu cargo test --test gpu_hardware_tests
```

## GitHub Actions Integration

### Enable GPU Tests
Once you have a self-hosted runner set up:

1. **Remove the `if: false` condition** in `.github/workflows/ci.yml`:
   ```yaml
   gpu-hardware-tests:
     name: GPU Hardware Tests
     runs-on: [self-hosted, gpu]
     # if: false  # Remove this line
   ```

2. **Add runner labels** when setting up:
   ```bash
   ./config.sh --labels "gpu,nvidia,linux" --name "nvidia-gpu-runner"
   ```

### Conditional GPU Testing
The CI will automatically:
- ✅ **Run GPU tests** when GPU hardware is available
- ✅ **Skip gracefully** when no GPU hardware is found
- ✅ **Work on any runner** (hosted or self-hosted)

## Cost Optimization

### Scheduled Testing
Set up runners to only run during business hours:
```yaml
on:
  schedule:
    - cron: '0 9 * * 1-5'  # 9 AM, Monday-Friday
```

### Auto-shutdown
Add auto-shutdown to cloud instances:
```bash
# AWS
aws ec2 create-tags --resources i-1234567890abcdef0 --tags Key=shutdown,Value=yes

# GCP
gcloud compute instances add-metadata gpu-test-runner \
  --metadata shutdown-script='sudo shutdown -h +60'
```

## Monitoring and Alerts

### Set up monitoring for:
- GPU utilization during tests
- Test success/failure rates
- Runner availability
- Cost tracking

### Example monitoring script:
```bash
#!/bin/bash
# Monitor GPU test results
curl -H "Authorization: token $GITHUB_TOKEN" \
  "https://api.github.com/repos/treadiehq/gpu-kill/actions/runs" | \
  jq '.workflow_runs[] | select(.name=="GPU Hardware Tests") | {status, conclusion, created_at}'
```

## Troubleshooting

### Common Issues:

1. **GPU not detected:**
   ```bash
   # Check NVIDIA
   nvidia-smi
   
   # Check AMD
   rocm-smi --showid
   
   # Check Intel
   intel_gpu_top
   ```

2. **Permission issues:**
   ```bash
   # Add user to docker group
   sudo usermod -aG docker $USER
   
   # Check GPU permissions
   ls -la /dev/nvidia*
   ```

3. **Driver issues:**
   ```bash
   # Update NVIDIA drivers
   sudo apt-get install nvidia-driver-470
   
   # Update AMD drivers
   sudo apt-get install rocm-dkms
   ```

## Next Steps

1. **Choose your cloud provider** (AWS, GCP, Azure)
2. **Set up a GPU instance** using the scripts above
3. **Configure the GitHub Actions runner** with GPU labels
4. **Enable GPU tests** in the CI workflow
5. **Monitor and optimize** costs and performance

The GPU tests will now run automatically whenever GPU hardware is available! 🚀
