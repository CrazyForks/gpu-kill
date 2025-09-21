use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};

use crate::audit::{AuditRecord, AuditManager};
use crate::nvml_api::GpuProc;

/// Rogue detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RogueDetectionResult {
    pub timestamp: DateTime<Utc>,
    pub suspicious_processes: Vec<SuspiciousProcess>,
    pub crypto_miners: Vec<CryptoMiner>,
    pub resource_abusers: Vec<ResourceAbuser>,
    pub data_exfiltrators: Vec<DataExfiltrator>,
    pub risk_score: f32,
    pub recommendations: Vec<String>,
}

/// Suspicious process detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuspiciousProcess {
    pub process: GpuProc,
    pub reasons: Vec<String>,
    pub confidence: f32,
    pub risk_level: RiskLevel,
}

/// Crypto miner detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoMiner {
    pub process: GpuProc,
    pub mining_indicators: Vec<String>,
    pub confidence: f32,
    pub estimated_hashrate: Option<f32>,
}

/// Resource abuse detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAbuser {
    pub process: GpuProc,
    pub abuse_type: AbuseType,
    pub severity: f32,
    pub duration_hours: f32,
}

/// Data exfiltration detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataExfiltrator {
    pub process: GpuProc,
    pub exfil_indicators: Vec<String>,
    pub confidence: f32,
    pub data_volume_mb: Option<f32>,
}

/// Risk levels for suspicious activity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Types of resource abuse
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AbuseType {
    MemoryHog,
    LongRunning,
    ExcessiveUtilization,
    UnauthorizedAccess,
}

/// Rogue detection heuristics and rules
pub struct RogueDetector {
    audit_manager: AuditManager,
    detection_rules: DetectionRules,
}

/// Configurable detection rules
#[derive(Debug, Clone)]
pub struct DetectionRules {
    pub crypto_miner_patterns: Vec<String>,
    pub suspicious_process_names: Vec<String>,
    pub max_memory_usage_gb: f32,
    pub max_utilization_pct: f32,
    pub max_duration_hours: f32,
    pub min_confidence_threshold: f32,
}

impl Default for DetectionRules {
    fn default() -> Self {
        Self {
            crypto_miner_patterns: vec![
                "cuda".to_string(),
                "opencl".to_string(),
                "miner".to_string(),
                "hash".to_string(),
                "cryptonight".to_string(),
                "ethash".to_string(),
                "equihash".to_string(),
            ],
            suspicious_process_names: vec![
                "xmrig".to_string(),
                "ccminer".to_string(),
                "cgminer".to_string(),
                "bfgminer".to_string(),
                "sgminer".to_string(),
                "ethminer".to_string(),
                "t-rex".to_string(),
                "lolminer".to_string(),
                "nbminer".to_string(),
                "gminer".to_string(),
            ],
            max_memory_usage_gb: 20.0,
            max_utilization_pct: 95.0,
            max_duration_hours: 24.0,
            min_confidence_threshold: 0.7,
        }
    }
}

impl RogueDetector {
    /// Create a new rogue detector
    pub fn new(audit_manager: AuditManager) -> Self {
        Self {
            audit_manager,
            detection_rules: DetectionRules::default(),
        }
    }

    /// Create a new rogue detector with configuration
    pub fn with_config(audit_manager: AuditManager, config_manager: &crate::rogue_config::RogueConfigManager) -> Self {
        Self {
            audit_manager,
            detection_rules: config_manager.to_detection_rules(),
        }
    }

    /// Create a new rogue detector with custom rules
    #[allow(dead_code)]
    pub fn with_rules(audit_manager: AuditManager, rules: DetectionRules) -> Self {
        Self {
            audit_manager,
            detection_rules: rules,
        }
    }

    /// Analyze audit records for suspicious activity
    pub async fn detect_rogue_activity(&self, hours: u32) -> Result<RogueDetectionResult> {
        info!("Starting rogue activity detection for last {} hours", hours);
        
        let audit_records = self.audit_manager.query_records(hours, None, None).await?;
        debug!("Analyzing {} audit records", audit_records.len());

        let mut suspicious_processes = Vec::new();
        let mut crypto_miners = Vec::new();
        let mut resource_abusers = Vec::new();
        let mut data_exfiltrators = Vec::new();

        // Group audit records by PID for analysis
        let process_groups = self.group_records_by_pid(&audit_records);

        for (_pid, records) in process_groups {
            // Detect crypto miners
            if let Some(miner) = self.detect_crypto_miner(&records) {
                crypto_miners.push(miner);
            }

            // Detect suspicious processes
            if let Some(suspicious) = self.detect_suspicious_process(&records) {
                suspicious_processes.push(suspicious);
            }

            // Detect resource abusers
            if let Some(abuser) = self.detect_resource_abuser(&records) {
                resource_abusers.push(abuser);
            }

            // Detect data exfiltrators
            if let Some(exfiltrator) = self.detect_data_exfiltrator(&records) {
                data_exfiltrators.push(exfiltrator);
            }
        }

        // Calculate overall risk score
        let risk_score = self.calculate_risk_score(
            &suspicious_processes,
            &crypto_miners,
            &resource_abusers,
            &data_exfiltrators,
        );

        // Generate recommendations
        let recommendations = self.generate_recommendations(
            &suspicious_processes,
            &crypto_miners,
            &resource_abusers,
            &data_exfiltrators,
        );

        let result = RogueDetectionResult {
            timestamp: Utc::now(),
            suspicious_processes,
            crypto_miners,
            resource_abusers,
            data_exfiltrators,
            risk_score,
            recommendations,
        };

        info!("Rogue detection completed. Risk score: {:.2}", risk_score);
        Ok(result)
    }

    /// Group audit records by PID for analysis
    fn group_records_by_pid(&self, records: &[AuditRecord]) -> HashMap<u32, Vec<AuditRecord>> {
        let mut groups = HashMap::new();
        
        for record in records {
            if let Some(pid) = record.pid {
                groups.entry(pid).or_insert_with(Vec::new).push(record.clone());
            }
        }
        
        groups
    }

    /// Detect crypto mining activity
    fn detect_crypto_miner(&self, records: &[AuditRecord]) -> Option<CryptoMiner> {
        if records.is_empty() {
            return None;
        }

        let record = &records[0]; // Use first record as representative
        let mut indicators = Vec::new();
        let mut confidence = 0.0;

        // Check process name patterns
        if let Some(process_name) = &record.process_name {
            let process_name_lower = process_name.to_lowercase();
            for pattern in &self.detection_rules.crypto_miner_patterns {
                if process_name_lower.contains(pattern) {
                    indicators.push(format!("Process name contains '{}'", pattern));
                    confidence += 0.3;
                }
            }

            // Check for known miner names
            for miner_name in &self.detection_rules.suspicious_process_names {
                if process_name_lower.contains(miner_name) {
                    indicators.push(format!("Known miner process: {}", miner_name));
                    confidence += 0.5;
                }
            }
        }

        // Check for high GPU utilization
        if let Some(avg_util) = self.calculate_average_utilization(records) {
            if avg_util > 90.0 {
                indicators.push(format!("High GPU utilization: {:.1}%", avg_util));
                confidence += 0.2;
            }
        }

        // Check for sustained high memory usage
        if let Some(avg_memory) = self.calculate_average_memory_usage(records) {
            if avg_memory > 8.0 {
                indicators.push(format!("High memory usage: {:.1} GB", avg_memory));
                confidence += 0.1;
            }
        }

        // Check for long-running processes
        if let Some(duration) = self.calculate_process_duration(records) {
            if duration > 2.0 {
                indicators.push(format!("Long-running process: {:.1} hours", duration));
                confidence += 0.1;
            }
        }

        if confidence >= self.detection_rules.min_confidence_threshold {
            // Create a GpuProc from the AuditRecord for compatibility
            let process = GpuProc {
                gpu_index: record.gpu_index,
                pid: record.pid.unwrap_or(0),
                user: record.user.clone().unwrap_or_else(|| "unknown".to_string()),
                proc_name: record.process_name.clone().unwrap_or_else(|| "unknown".to_string()),
                used_mem_mb: record.memory_used_mb,
                start_time: "unknown".to_string(),
                container: record.container.clone(),
            };

            Some(CryptoMiner {
                process,
                mining_indicators: indicators,
                confidence,
                estimated_hashrate: self.estimate_hashrate(records),
            })
        } else {
            None
        }
    }

    /// Detect suspicious processes
    fn detect_suspicious_process(&self, records: &[AuditRecord]) -> Option<SuspiciousProcess> {
        if records.is_empty() {
            return None;
        }

        let record = &records[0];
        let mut reasons = Vec::new();
        let mut confidence = 0.0;

        // Check for unusual process names
        if let Some(process_name) = &record.process_name {
            if self.is_unusual_process_name(process_name) {
                reasons.push("Unusual process name pattern".to_string());
                confidence += 0.3;
            }
        }

        // Check for high resource usage
        if let Some(avg_util) = self.calculate_average_utilization(records) {
            if avg_util > self.detection_rules.max_utilization_pct {
                reasons.push(format!("Excessive GPU utilization: {:.1}%", avg_util));
                confidence += 0.4;
            }
        }

        if let Some(avg_memory) = self.calculate_average_memory_usage(records) {
            if avg_memory > self.detection_rules.max_memory_usage_gb {
                reasons.push(format!("Excessive memory usage: {:.1} GB", avg_memory));
                confidence += 0.3;
            }
        }

        // Check for unusual user
        if let Some(user) = &record.user {
            if self.is_unusual_user(user) {
                reasons.push(format!("Unusual user: {}", user));
                confidence += 0.2;
            }
        }

        if confidence >= self.detection_rules.min_confidence_threshold {
            // Create a GpuProc from the AuditRecord for compatibility
            let process = GpuProc {
                gpu_index: record.gpu_index,
                pid: record.pid.unwrap_or(0),
                user: record.user.clone().unwrap_or_else(|| "unknown".to_string()),
                proc_name: record.process_name.clone().unwrap_or_else(|| "unknown".to_string()),
                used_mem_mb: record.memory_used_mb,
                start_time: "unknown".to_string(),
                container: record.container.clone(),
            };

            Some(SuspiciousProcess {
                process,
                reasons,
                confidence,
                risk_level: self.determine_risk_level(confidence),
            })
        } else {
            None
        }
    }

    /// Detect resource abuse
    fn detect_resource_abuser(&self, records: &[AuditRecord]) -> Option<ResourceAbuser> {
        if records.is_empty() {
            return None;
        }

        let record = &records[0];
        let mut abuse_type = AbuseType::MemoryHog;
        let mut severity = 0.0;

        // Check for memory abuse
        if let Some(avg_memory) = self.calculate_average_memory_usage(records) {
            if avg_memory > self.detection_rules.max_memory_usage_gb {
                abuse_type = AbuseType::MemoryHog;
                severity = (avg_memory / self.detection_rules.max_memory_usage_gb).min(2.0);
            }
        }

        // Check for excessive utilization
        if let Some(avg_util) = self.calculate_average_utilization(records) {
            if avg_util > self.detection_rules.max_utilization_pct {
                abuse_type = AbuseType::ExcessiveUtilization;
                severity = (avg_util / self.detection_rules.max_utilization_pct).min(2.0);
            }
        }

        // Check for long-running processes
        if let Some(duration) = self.calculate_process_duration(records) {
            if duration > self.detection_rules.max_duration_hours {
                abuse_type = AbuseType::LongRunning;
                severity = (duration / self.detection_rules.max_duration_hours).min(2.0);
            }
        }

        if severity > 1.0 {
            // Create a GpuProc from the AuditRecord for compatibility
            let process = GpuProc {
                gpu_index: record.gpu_index,
                pid: record.pid.unwrap_or(0),
                user: record.user.clone().unwrap_or_else(|| "unknown".to_string()),
                proc_name: record.process_name.clone().unwrap_or_else(|| "unknown".to_string()),
                used_mem_mb: record.memory_used_mb,
                start_time: "unknown".to_string(),
                container: record.container.clone(),
            };

            Some(ResourceAbuser {
                process,
                abuse_type,
                severity,
                duration_hours: self.calculate_process_duration(records).unwrap_or(0.0),
            })
        } else {
            None
        }
    }

    /// Detect data exfiltration (placeholder - would need network monitoring)
    fn detect_data_exfiltrator(&self, _records: &[AuditRecord]) -> Option<DataExfiltrator> {
        // This would require network monitoring data
        // For now, we'll implement basic heuristics
        None
    }

    /// Calculate average GPU utilization for a process
    fn calculate_average_utilization(&self, records: &[AuditRecord]) -> Option<f32> {
        if records.is_empty() {
            return None;
        }

        let total_util: f32 = records.iter()
            .map(|r| r.utilization_pct)
            .sum();
        
        Some(total_util / records.len() as f32)
    }

    /// Calculate average memory usage for a process
    fn calculate_average_memory_usage(&self, records: &[AuditRecord]) -> Option<f32> {
        if records.is_empty() {
            return None;
        }

        let total_memory: f32 = records.iter()
            .map(|r| r.memory_used_mb as f32 / 1024.0)
            .sum();
        
        Some(total_memory / records.len() as f32)
    }

    /// Calculate process duration in hours
    fn calculate_process_duration(&self, records: &[AuditRecord]) -> Option<f32> {
        if records.len() < 2 {
            return None;
        }

        let timestamps: Vec<DateTime<Utc>> = records.iter()
            .map(|r| r.timestamp)
            .collect();

        if timestamps.len() < 2 {
            return None;
        }

        let min_time = timestamps.iter().min()?;
        let max_time = timestamps.iter().max()?;
        let duration = (*max_time - *min_time).num_seconds() as f32 / 3600.0;
        
        Some(duration)
    }

    /// Estimate hashrate for crypto mining (placeholder)
    fn estimate_hashrate(&self, _records: &[AuditRecord]) -> Option<f32> {
        // This would require more sophisticated analysis
        None
    }

    /// Check if process name is unusual
    fn is_unusual_process_name(&self, name: &str) -> bool {
        let name_lower = name.to_lowercase();
        
        // Check for random-looking names
        if name.len() > 20 && name.chars().filter(|c| c.is_ascii_digit()).count() > 5 {
            return true;
        }

        // Check for suspicious patterns
        let suspicious_patterns = ["temp", "tmp", "random", "test", "unknown"];
        for pattern in suspicious_patterns {
            if name_lower.contains(pattern) {
                return true;
            }
        }

        false
    }

    /// Check if user is unusual
    fn is_unusual_user(&self, user: &str) -> bool {
        let unusual_users = ["root", "admin", "system", "daemon", "nobody"];
        unusual_users.contains(&user.to_lowercase().as_str())
    }

    /// Determine risk level based on confidence
    fn determine_risk_level(&self, confidence: f32) -> RiskLevel {
        match confidence {
            c if c >= 0.9 => RiskLevel::Critical,
            c if c >= 0.7 => RiskLevel::High,
            c if c >= 0.5 => RiskLevel::Medium,
            _ => RiskLevel::Low,
        }
    }

    /// Calculate overall risk score
    fn calculate_risk_score(
        &self,
        suspicious: &[SuspiciousProcess],
        miners: &[CryptoMiner],
        abusers: &[ResourceAbuser],
        _exfiltrators: &[DataExfiltrator],
    ) -> f32 {
        let mut score = 0.0;

        // Weight different types of threats
        for process in suspicious {
            score += match process.risk_level {
                RiskLevel::Critical => 1.0,
                RiskLevel::High => 0.7,
                RiskLevel::Medium => 0.4,
                RiskLevel::Low => 0.1,
            };
        }

        for miner in miners {
            score += miner.confidence * 0.8; // Crypto miners are high priority
        }

        for abuser in abusers {
            score += abuser.severity * 0.3; // Resource abuse is medium priority
        }

        // Normalize to 0-1 scale
        (score / 10.0).min(1.0)
    }

    /// Generate recommendations based on detected threats
    fn generate_recommendations(
        &self,
        suspicious: &[SuspiciousProcess],
        miners: &[CryptoMiner],
        abusers: &[ResourceAbuser],
        _exfiltrators: &[DataExfiltrator],
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        if !miners.is_empty() {
            recommendations.push("🚨 CRITICAL: Crypto miners detected! Consider immediate termination.".to_string());
            recommendations.push("Review system security and check for unauthorized access.".to_string());
        }

        if !suspicious.is_empty() {
            recommendations.push("⚠️ Suspicious processes detected. Review and investigate.".to_string());
        }

        if !abusers.is_empty() {
            recommendations.push("📊 Resource abuse detected. Consider implementing usage limits.".to_string());
        }

        if recommendations.is_empty() {
            recommendations.push("✅ No suspicious activity detected.".to_string());
        }

        recommendations
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::AuditManager;
    use chrono::Utc;

    #[test]
    fn test_detection_rules_default() {
        let rules = DetectionRules::default();
        assert!(!rules.crypto_miner_patterns.is_empty());
        assert!(!rules.suspicious_process_names.is_empty());
        assert!(rules.max_memory_usage_gb > 0.0);
    }

    #[test]
    fn test_rogue_detector_creation() {
        let audit_manager = AuditManager::new().unwrap();
        let detector = RogueDetector::new(audit_manager);
        assert_eq!(detector.detection_rules.min_confidence_threshold, 0.7);
    }
}
