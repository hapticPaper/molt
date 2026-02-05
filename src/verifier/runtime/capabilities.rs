//! Validator capabilities and environment checking.
//!
//! This module manages what languages a validator can execute and provides
//! environment setup validation.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Get the validator's data directory
fn validator_data_dir() -> PathBuf {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());

    PathBuf::from(home).join(".hardclaw")
}

/// Configuration for validator's Python environment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythonSandboxConfig {
    pub venv_path: PathBuf,
    pub python_bin: PathBuf,
    pub pip_bin: PathBuf,
    pub installed_packages: Vec<String>,
}

/// Configuration for validator's AI models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIModelConfig {
    pub ollama_models_dir: PathBuf,
    pub downloaded_models: Vec<String>,
}

/// Languages supported by the verification system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LanguageSupport {
    Python,
    JavaScript,
    TypeScript,
    Wasm,
}

impl LanguageSupport {
    /// Get all supported languages
    pub fn all() -> Vec<Self> {
        vec![Self::Python, Self::JavaScript, Self::TypeScript, Self::Wasm]
    }

    /// Get the language name
    pub fn name(&self) -> &'static str {
        match self {
            Self::Python => "python",
            Self::JavaScript => "javascript",
            Self::TypeScript => "typescript",
            Self::Wasm => "wasm",
        }
    }

    /// Get the display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Python => "Python",
            Self::JavaScript => "JavaScript",
            Self::TypeScript => "TypeScript",
            Self::Wasm => "WebAssembly",
        }
    }
}

/// Environment check results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentCheck {
    /// Language being checked
    pub language: LanguageSupport,
    /// Whether the language runtime is available
    pub available: bool,
    /// Version information if available
    pub version: Option<String>,
    /// Any warnings or issues
    pub warnings: Vec<String>,
    /// Setup instructions if not available
    pub setup_instructions: Option<String>,
}

/// AI model check for safety review
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIModelCheck {
    /// Whether AI review is available
    pub available: bool,
    /// Configured models
    pub models: Vec<String>,
    /// Warnings
    pub warnings: Vec<String>,
    /// Setup instructions
    pub setup_instructions: Option<String>,
}

impl AIModelCheck {
    /// Setup and verify AI model environment (install Ollama + download models)
    pub fn check() -> Self {
        let mut warnings = Vec::new();
        let mut setup_instructions = None;

        let data_dir = validator_data_dir();
        let config_path = data_dir.join("ai-models-config.json");

        eprintln!("🤖 Setting up AI model environment...");

        // Step 1: Install Ollama if not present
        let ollama_check = Command::new("ollama").arg("--version").output();

        let ollama_ok = matches!(ollama_check, Ok(output) if output.status.success());

        if !ollama_ok {
            eprintln!("📦 Installing Ollama...");

            #[cfg(target_os = "macos")]
            let install_result = Command::new("sh")
                .arg("-c")
                .arg("brew install ollama && brew services start ollama")
                .status();

            #[cfg(target_os = "linux")]
            let install_result = Command::new("sh")
                .arg("-c")
                .arg("curl -fsSL https://ollama.com/install.sh | sh && sudo systemctl enable ollama && sudo systemctl start ollama")
                .status();

            #[cfg(target_os = "windows")]
            let install_result = Command::new("winget")
                .args(&["install", "Ollama.Ollama"])
                .status();

            #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
            let install_result: Result<std::process::ExitStatus, std::io::Error> =
                Err(std::io::Error::new(
                    std::io::ErrorKind::Unsupported,
                    "Auto-install not supported on this OS",
                ));

            match install_result {
                Ok(status) if status.success() => {
                    eprintln!("✅ Ollama installed and started");
                    std::thread::sleep(std::time::Duration::from_secs(3)); // Wait for service
                }
                _ => {
                    setup_instructions = Some("Install Ollama manually:\n  https://ollama.com/download\n  Then run: ollama pull llama3.2".to_string());

                    return Self {
                        available: false,
                        models: Vec::new(),
                        warnings: vec!["Ollama installation failed".to_string()],
                        setup_instructions,
                    };
                }
            }
        }

        // Step 2: Ensure Ollama service is running
        #[cfg(target_os = "linux")]
        {
            Command::new("sudo")
                .args(&["systemctl", "start", "ollama"])
                .status()
                .ok();
        }

        #[cfg(target_os = "macos")]
        {
            Command::new("brew")
                .args(&["services", "start", "ollama"])
                .status()
                .ok();
        }

        std::thread::sleep(std::time::Duration::from_secs(1));

        // Step 3: Check available models
        let models_output = Command::new("ollama").arg("list").output();

        let mut models = match models_output {
            Ok(output) if output.status.success() => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                stdout
                    .lines()
                    .skip(1) // Skip header
                    .filter_map(|line| line.split_whitespace().next())
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
            }
            _ => Vec::new(),
        };

        // Step 4: Download required models
        let required_models = vec!["llama3.2", "codellama"];

        for model in &required_models {
            if !models.iter().any(|m| m.starts_with(model)) {
                eprintln!(
                    "📥 Downloading {} (this may take several minutes)...",
                    model
                );

                let pull_result = Command::new("ollama").args(&["pull", model]).status();

                match pull_result {
                    Ok(status) if status.success() => {
                        eprintln!("  ✓ {}", model);
                        models.push(model.to_string());
                    }
                    _ => {
                        warnings.push(format!("Failed to download {}", model));
                    }
                }
            }
        }

        // Step 5: Save config
        fs::create_dir_all(&data_dir).ok();

        let ollama_models_dir = std::env::var("OLLAMA_MODELS")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                #[cfg(target_os = "macos")]
                {
                    PathBuf::from("/usr/local/share/ollama/models")
                }
                #[cfg(target_os = "linux")]
                {
                    PathBuf::from("/usr/share/ollama/.ollama/models")
                }
                #[cfg(target_os = "windows")]
                {
                    PathBuf::from(std::env::var("USERPROFILE").unwrap_or_default())
                        .join(".ollama")
                        .join("models")
                }
                #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
                {
                    PathBuf::from(".ollama/models")
                }
            });

        let config = AIModelConfig {
            ollama_models_dir,
            downloaded_models: models.clone(),
        };

        if let Ok(json) = serde_json::to_string_pretty(&config) {
            fs::write(&config_path, json).ok();
            eprintln!("💾 Saved AI config to {}", config_path.display());
        }

        Self {
            available: !models.is_empty(),
            models,
            warnings,
            setup_instructions,
        }
    }

    /// Check if suitable model is available for code review
    pub fn has_code_model(&self) -> bool {
        self.models
            .iter()
            .any(|m| m.contains("llama") || m.contains("code") || m.contains("qwen"))
    }
}

impl EnvironmentCheck {
    /// Setup and verify Python 3.12+ environment for PyO3 sandbox
    pub fn check_python() -> Self {
        let mut warnings = Vec::new();
        let mut setup_instructions = None;

        let data_dir = validator_data_dir();
        let venv_path = data_dir.join("python-sandbox");
        let config_path = data_dir.join("python-config.json");

        eprintln!(
            "🔧 Setting up Python sandbox environment at {}",
            venv_path.display()
        );

        // Step 1: Ensure base Python 3.12+ is installed
        let check_result = Command::new("python3").arg("--version").output();

        let python_ok = match check_result {
            Ok(output) if output.status.success() => {
                let version_str = String::from_utf8_lossy(&output.stdout);
                version_str.contains("3.12")
                    || version_str.contains("3.13")
                    || version_str.contains("3.14")
            }
            _ => false,
        };

        if !python_ok {
            eprintln!("📦 Installing Python 3.12...");

            #[cfg(target_os = "macos")]
            let install_result = Command::new("brew")
                .args(&["install", "python@3.12"])
                .status();

            #[cfg(target_os = "linux")]
            let install_result = Command::new("sudo")
                .args(&[
                    "apt",
                    "install",
                    "-y",
                    "python3.12",
                    "python3.12-dev",
                    "python3.12-venv",
                ])
                .status();

            #[cfg(target_os = "windows")]
            let install_result = Command::new("winget")
                .args(&["install", "Python.Python.3.12"])
                .status();

            #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
            let install_result: Result<std::process::ExitStatus, std::io::Error> =
                Err(std::io::Error::new(
                    std::io::ErrorKind::Unsupported,
                    "Auto-install not supported on this OS",
                ));

            match install_result {
                Ok(status) if status.success() => {
                    eprintln!("✅ Python 3.12 installed");
                }
                _ => {
                    return Self {
                        language: LanguageSupport::Python,
                        available: false,
                        version: None,
                        warnings: vec!["Python 3.12+ installation failed".to_string()],
                        setup_instructions: Some("Install Python 3.12+ manually:\n  macOS: brew install python@3.12\n  Linux: sudo apt install python3.12 python3.12-venv\n  Windows: winget install Python.Python.3.12".to_string()),
                    };
                }
            }
        }

        // Step 2: Create dedicated venv for sandbox
        fs::create_dir_all(&data_dir).ok();

        if !venv_path.exists() {
            eprintln!("🏗️  Creating Python virtual environment...");

            let venv_result = Command::new("python3")
                .args(&["-m", "venv", venv_path.to_str().unwrap()])
                .status();

            match venv_result {
                Ok(status) if status.success() => {
                    eprintln!("✅ Virtual environment created");
                }
                _ => {
                    return Self {
                        language: LanguageSupport::Python,
                        available: false,
                        version: None,
                        warnings: vec!["Failed to create venv".to_string()],
                        setup_instructions: Some(format!(
                            "Manually create: python3 -m venv {}",
                            venv_path.display()
                        )),
                    };
                }
            }
        }

        // Step 3: Install required packages in venv
        #[cfg(target_os = "windows")]
        let pip_bin = venv_path.join("Scripts").join("pip");
        #[cfg(not(target_os = "windows"))]
        let pip_bin = venv_path.join("bin").join("pip");

        let required_packages = vec![
            "cryptography", // For verification code that uses crypto
            "requests",     // Common in verification code
            "numpy",        // Data processing
        ];

        eprintln!("📦 Installing verification packages...");
        for package in &required_packages {
            let install = Command::new(&pip_bin)
                .args(&["install", "--quiet", package])
                .status();

            match install {
                Ok(status) if status.success() => {
                    eprintln!("  ✓ {}", package);
                }
                _ => {
                    warnings.push(format!("Failed to install {}", package));
                }
            }
        }

        // Step 4: Save config for node to use
        #[cfg(target_os = "windows")]
        let python_bin = venv_path.join("Scripts").join("python");
        #[cfg(not(target_os = "windows"))]
        let python_bin = venv_path.join("bin").join("python");

        let config = PythonSandboxConfig {
            venv_path: venv_path.clone(),
            python_bin: python_bin.clone(),
            pip_bin: pip_bin.clone(),
            installed_packages: required_packages.iter().map(|s| s.to_string()).collect(),
        };

        if let Ok(json) = serde_json::to_string_pretty(&config) {
            fs::write(&config_path, json).ok();
            eprintln!("💾 Saved config to {}", config_path.display());
        }

        // Step 5: Test execution with actual verification code
        eprintln!("🧪 Testing sandbox execution...");

        let test_code = r#"
import hashlib
import sys

def verify(input_data, output_data):
    # Test actual cryptographic verification
    expected = hashlib.sha256(input_data).digest()
    actual = hashlib.sha256(output_data).digest()
    return expected == actual

# Test
input_data = b"test_verification_input"
output_data = b"test_verification_input"
result = verify(input_data, output_data)
print("true" if result else "false")
"#;

        let test_script = venv_path.join("test_verification.py");
        fs::write(&test_script, test_code).ok();

        let test_result = Command::new(&python_bin).arg(&test_script).output();

        let (available, version) = match test_result {
            Ok(output) if output.status.success() => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if stdout.trim() == "true" {
                    eprintln!("✅ Sandbox verified - executed hash comparison");

                    let version_output = Command::new(&python_bin).arg("--version").output().ok();

                    let version = version_output
                        .and_then(|o| String::from_utf8(o.stdout).ok())
                        .unwrap_or_else(|| "Python 3.12+".to_string());

                    (true, format!("{} (isolated venv)", version.trim()))
                } else {
                    warnings.push("Verification returned unexpected result".to_string());
                    (false, "Python 3.12+ (venv)".to_string())
                }
            }
            _ => {
                warnings.push("Sandbox execution failed".to_string());
                (false, "Python 3.12+ (venv)".to_string())
            }
        };

        if !available {
            setup_instructions = Some(format!(
                "Sandbox test failed\nCheck venv: {}\nRecreate: rm -rf {} && cargo run --bin hardclaw",
                venv_path.display(),
                venv_path.display()
            ));
        }

        Self {
            language: LanguageSupport::Python,
            available,
            version: Some(version),
            warnings,
            setup_instructions,
        }
    }

    /// Check Deno environment (deno_core embedded - always available)
    pub fn check_nodejs() -> Self {
        use super::{JavaScriptRuntime, VerificationRuntime};
        use deno_core::{JsRuntime, RuntimeOptions};

        let mut warnings = Vec::new();

        eprintln!("🧪 Testing JavaScript sandbox with verification code...");

        // Test actual verification code execution
        let runtime = JavaScriptRuntime::new();
        let test_code = r#"
function verify() {
    // Test actual array comparison verification
    if (inputData.length !== outputData.length) return false;
    
    for (let i = 0; i < inputData.length; i++) {
        if (inputData[i] !== outputData[i]) return false;
    }
    
    return true;
}
"#;

        let test_input = b"test_verification_input";
        let test_output = b"test_verification_input"; // Same = should return true

        let available = match runtime.execute(test_code, test_input, test_output) {
            Ok(true) => {
                eprintln!("✅ JavaScript sandbox verified - executed array comparison");
                true
            }
            Ok(false) => {
                warnings.push("Verification returned false (unexpected)".to_string());
                false
            }
            Err(e) => {
                warnings.push(format!("Sandbox execution failed: {}", e));
                false
            }
        };

        let setup_instructions = if !available {
            Some(
                "Deno runtime failed - rebuild binary:\n  cargo clean\n  cargo build --release"
                    .to_string(),
            )
        } else {
            None
        };

        Self {
            language: LanguageSupport::JavaScript,
            available,
            version: Some("embedded (deno_core verified)".to_string()),
            warnings,
            setup_instructions,
        }
    }

    /// Check TypeScript environment
    pub fn check_typescript() -> Self {
        // TypeScript is handled by Deno, same as JavaScript
        let mut check = Self::check_nodejs();
        check.language = LanguageSupport::TypeScript;
        check
    }

    /// Check WASM environment
    pub fn check_wasm() -> Self {
        // WASM runtime is embedded via wasmer
        Self {
            language: LanguageSupport::Wasm,
            available: true,
            version: Some("embedded (wasmer)".to_string()),
            warnings: Vec::new(),
            setup_instructions: None,
        }
    }

    /// Check all language environments
    pub fn check_all() -> Vec<Self> {
        vec![
            Self::check_python(),
            Self::check_nodejs(),
            Self::check_typescript(),
            Self::check_wasm(),
        ]
    }
}

/// Validator's language capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorCapabilities {
    /// Languages this validator can execute
    pub supported_languages: Vec<LanguageSupport>,
    /// Preference weights (0.0 to 1.0) for each language
    /// Higher weight = more capacity/preference for this language
    pub weights: HashMap<LanguageSupport, f64>,
}

impl ValidatorCapabilities {
    /// Create capabilities from environment check
    pub fn from_environment() -> Self {
        let checks = EnvironmentCheck::check_all();
        let mut supported = Vec::new();
        let mut weights = HashMap::new();

        for check in checks {
            if check.available {
                supported.push(check.language);
                // Default weight of 1.0 for available languages
                weights.insert(check.language, 1.0);
            }
        }

        Self {
            supported_languages: supported,
            weights,
        }
    }

    /// Create with specific languages
    pub fn new(languages: Vec<LanguageSupport>) -> Self {
        let mut weights = HashMap::new();
        for lang in &languages {
            weights.insert(*lang, 1.0);
        }

        Self {
            supported_languages: languages,
            weights,
        }
    }

    /// Set preference weight for a language (0.0 to 1.0)
    pub fn set_weight(&mut self, language: LanguageSupport, weight: f64) {
        if self.supported_languages.contains(&language) {
            self.weights.insert(language, weight.clamp(0.0, 1.0));
        }
    }

    /// Get weight for a language
    pub fn get_weight(&self, language: &LanguageSupport) -> f64 {
        self.weights.get(language).copied().unwrap_or(0.0)
    }

    /// Check if language is supported
    pub fn supports(&self, language: &LanguageSupport) -> bool {
        self.supported_languages.contains(language)
    }

    /// Get weighted preference score for a language
    /// Returns 0.0 if not supported
    pub fn preference_score(&self, language: &LanguageSupport) -> f64 {
        if self.supports(language) {
            self.get_weight(language)
        } else {
            0.0
        }
    }
}

/// Calculate supply/demand weights for job distribution
pub struct JobDistribution;

impl JobDistribution {
    /// Calculate weights for job assignment based on validator capabilities
    ///
    /// This implements a supply/demand weighting system:
    /// - If few validators support a language, weight increases (scarcity premium)
    /// - If many validators support a language, weight decreases (abundance discount)
    pub fn calculate_weights(
        language: LanguageSupport,
        all_validators: &[ValidatorCapabilities],
    ) -> Vec<(usize, f64)> {
        let total_validators = all_validators.len() as f64;
        if total_validators == 0.0 {
            return Vec::new();
        }

        // Count validators supporting this language
        let supporting_count = all_validators
            .iter()
            .filter(|v| v.supports(&language))
            .count() as f64;

        if supporting_count == 0.0 {
            return Vec::new();
        }

        // Calculate scarcity multiplier (fewer validators = higher multiplier)
        let scarcity = total_validators / supporting_count;

        // Calculate weights for each validator
        all_validators
            .iter()
            .enumerate()
            .filter_map(|(idx, validator)| {
                if validator.supports(&language) {
                    let base_weight = validator.get_weight(&language);
                    let final_weight = base_weight * scarcity;
                    Some((idx, final_weight))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Select a validator for a job based on weighted random selection
    pub fn select_validator(
        language: LanguageSupport,
        all_validators: &[ValidatorCapabilities],
        random_value: f64, // 0.0 to 1.0
    ) -> Option<usize> {
        let weights = Self::calculate_weights(language, all_validators);
        if weights.is_empty() {
            return None;
        }

        let total_weight: f64 = weights.iter().map(|(_, w)| w).sum();
        let mut cumulative = 0.0;
        let target = random_value * total_weight;

        for (idx, weight) in &weights {
            cumulative += weight;
            if cumulative >= target {
                return Some(*idx);
            }
        }

        // Fallback to last validator
        weights.last().map(|(idx, _)| *idx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_checks() {
        let checks = EnvironmentCheck::check_all();
        assert!(!checks.is_empty());

        // WASM should always be available (embedded)
        let wasm_check = checks.iter().find(|c| c.language == LanguageSupport::Wasm);
        assert!(wasm_check.is_some());
        assert!(wasm_check.unwrap().available);
    }

    #[test]
    fn test_capabilities_from_environment() {
        let caps = ValidatorCapabilities::from_environment();
        // At minimum, WASM should be supported
        assert!(caps.supports(&LanguageSupport::Wasm));
    }

    #[test]
    fn test_job_distribution() {
        let validators = vec![
            ValidatorCapabilities::new(vec![LanguageSupport::Python, LanguageSupport::JavaScript]),
            ValidatorCapabilities::new(vec![LanguageSupport::Python]),
            ValidatorCapabilities::new(vec![LanguageSupport::JavaScript]),
        ];

        // Python is supported by 2/3 validators
        let weights = JobDistribution::calculate_weights(LanguageSupport::Python, &validators);
        assert_eq!(weights.len(), 2);

        // JavaScript is supported by 2/3 validators
        let weights = JobDistribution::calculate_weights(LanguageSupport::JavaScript, &validators);
        assert_eq!(weights.len(), 2);
    }

    #[test]
    fn test_scarcity_premium() {
        let validators = vec![
            ValidatorCapabilities::new(vec![LanguageSupport::Python]),
            ValidatorCapabilities::new(vec![LanguageSupport::Python]),
            ValidatorCapabilities::new(vec![LanguageSupport::Python]),
            ValidatorCapabilities::new(vec![LanguageSupport::JavaScript]), // Rare
        ];

        let python_weights =
            JobDistribution::calculate_weights(LanguageSupport::Python, &validators);
        let js_weights =
            JobDistribution::calculate_weights(LanguageSupport::JavaScript, &validators);

        // JavaScript should have higher weight due to scarcity (4/1 = 4x multiplier)
        assert!(js_weights[0].1 > python_weights[0].1);
    }
}
