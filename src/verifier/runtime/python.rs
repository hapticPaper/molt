//! Python verification runtime.
//!
//! By default, executes Python via subprocess (no build-time Python required).
//! With `embedded-python` feature, uses `PyO3` for embedded execution.

use super::{ExecutionStats, RuntimeError, SandboxConfig, VerificationRuntime};
use std::io::Write;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Python runtime for verification functions
pub struct PythonRuntime {
    config: SandboxConfig,
    stats: Arc<Mutex<ExecutionStats>>,
}

impl PythonRuntime {
    /// Create a new Python runtime with default configuration
    pub fn new() -> Self {
        Self::with_config(SandboxConfig::default())
    }

    /// Create a new Python runtime with custom configuration
    pub fn with_config(config: SandboxConfig) -> Self {
        Self {
            config,
            stats: Arc::new(Mutex::new(ExecutionStats::default())),
        }
    }

    /// Get the Python binary path - uses our standalone installation
    fn python_binary() -> std::path::PathBuf {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_else(|_| ".".to_string());

        let hardclaw_dir = std::path::PathBuf::from(home).join(".hardclaw");

        #[cfg(target_os = "windows")]
        let python_bin = hardclaw_dir
            .join("python")
            .join("python")
            .join("python.exe");
        #[cfg(not(target_os = "windows"))]
        let python_bin = hardclaw_dir
            .join("python")
            .join("python")
            .join("bin")
            .join("python3");

        python_bin
    }

    /// Execute Python code via subprocess (default, no `PyO3` linking)
    fn execute_subprocess(
        &self,
        code: &str,
        input: &[u8],
        output: &[u8],
    ) -> Result<bool, RuntimeError> {
        let start = Instant::now();
        let timeout_duration = Duration::from_millis(self.config.timeout_ms);

        // Build wrapper script that:
        // 1. Sets up restricted builtins
        // 2. Injects input_data and output_data
        // 3. Runs user code
        // 4. Calls verify() and prints result

        // Convert bytes to Python bytes literal
        let input_hex = hex::encode(input);
        let output_hex = hex::encode(output);
        let escaped_code = code.replace('\\', "\\\\").replace("'''", r"\'\'\'");

        let wrapper = format!(
            r"
import sys
import builtins

# Restricted builtins
SAFE_BUILTINS = {{
    'abs', 'all', 'any', 'ascii', 'bin', 'bool', 'bytearray', 'bytes',
    'chr', 'dict', 'divmod', 'enumerate', 'filter', 'float', 'format',
    'frozenset', 'hash', 'hex', 'int', 'isinstance', 'issubclass', 'iter',
    'len', 'list', 'map', 'max', 'min', 'oct', 'ord', 'pow', 'print',
    'range', 'repr', 'reversed', 'round', 'set', 'slice', 'sorted', 'str',
    'sum', 'tuple', 'type', 'zip', 'ImportError', 'True', 'False', 'None'
}}

# Safe import that only allows hashlib
_real_import = builtins.__import__
def _safe_import(name, globals=None, locals=None, fromlist=(), level=0):
    if name in {{'hashlib'}}:
        return _real_import(name, globals, locals, fromlist, level)
    raise ImportError(f'import {{name}} blocked')

restricted = {{k: getattr(builtins, k) for k in SAFE_BUILTINS if hasattr(builtins, k)}}
restricted['__import__'] = _safe_import

# Input/output data (passed as hex, decoded here)
input_data = bytes.fromhex('{input_hex}')
output_data = bytes.fromhex('{output_hex}')

# User code namespace
ns = {{'__builtins__': restricted, 'input_data': input_data, 'output_data': output_data}}
exec(compile('''{escaped_code}''', '<verification>', 'exec'), ns)

# Get verify function from the executed namespace
if 'verify' in ns and callable(ns['verify']):
    result = ns['verify']()
    print('VERIFY_RESULT:' + str(bool(result)))
    sys.exit(0)

print('VERIFY_ERROR:verify function not found')
sys.exit(1)
",
        );

        let python_bin = Self::python_binary();

        let mut child = Command::new(&python_bin)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                RuntimeError::ExecutionFailed(format!(
                    "Failed to spawn Python ({}): {}",
                    python_bin.display(),
                    e
                ))
            })?;

        // Write wrapper script to stdin
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(wrapper.as_bytes()).map_err(|e| {
                RuntimeError::ExecutionFailed(format!("Failed to write to stdin: {e}"))
            })?;
        }

        // Wait for output
        let output_data = child
            .wait_with_output()
            .map_err(|e| RuntimeError::ExecutionFailed(format!("Process error: {e}")))?;

        let duration = start.elapsed();

        if duration > timeout_duration {
            return Err(RuntimeError::Timeout(duration.as_millis() as u64));
        }

        // Update stats
        if let Ok(mut stats) = self.stats.lock() {
            stats.duration_ms = duration.as_millis() as u64;
            stats.success = output_data.status.success();
        }

        let stdout = String::from_utf8_lossy(&output_data.stdout);
        let stderr = String::from_utf8_lossy(&output_data.stderr);

        // Parse result
        if let Some(line) = stdout.lines().find(|l| l.starts_with("VERIFY_RESULT:")) {
            let result_str = line.strip_prefix("VERIFY_RESULT:").unwrap_or("False");
            return Ok(result_str == "True");
        }

        if let Some(line) = stdout.lines().find(|l| l.starts_with("VERIFY_ERROR:")) {
            let error = line.strip_prefix("VERIFY_ERROR:").unwrap_or("unknown");
            return Err(RuntimeError::FunctionNotFound(error.to_string()));
        }

        // Check for Python errors
        if !output_data.status.success() {
            return Err(RuntimeError::ExecutionFailed(format!(
                "Python exited with error:\n{stdout}{stderr}"
            )));
        }

        Err(RuntimeError::ExecutionFailed(format!(
            "No verification result found in output:\n{stdout}{stderr}"
        )))
    }
}

impl Default for PythonRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl VerificationRuntime for PythonRuntime {
    fn execute(&self, code: &str, input: &[u8], output: &[u8]) -> Result<bool, RuntimeError> {
        self.execute_subprocess(code, input, output)
    }

    fn is_available() -> bool {
        // Check if our standalone Python is installed
        let python_bin = Self::python_binary();
        python_bin.exists()
            && Command::new(&python_bin)
                .arg("--version")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
    }

    fn language_name(&self) -> &'static str {
        "python"
    }

    fn last_execution_stats(&self) -> ExecutionStats {
        self.stats.lock().unwrap().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_verification() {
        if !PythonRuntime::is_available() {
            eprintln!("Skipping test: Python not available");
            return;
        }

        let runtime = PythonRuntime::new();

        let code = r"
def verify():
    # Check if input equals output
    return input_data == output_data
";

        let input = b"hello";
        let output = b"hello";

        let result = runtime.execute(code, input, output);
        assert!(result.is_ok(), "Error: {result:?}");
        assert!(result.unwrap());
    }

    #[test]
    fn test_verification_fails() {
        if !PythonRuntime::is_available() {
            eprintln!("Skipping test: Python not available");
            return;
        }

        let runtime = PythonRuntime::new();

        let code = r"
def verify():
    return input_data == output_data
";

        let input = b"hello";
        let output = b"world"; // Different!

        let result = runtime.execute(code, input, output);
        assert!(result.is_ok());
        assert!(!result.unwrap()); // Should be false
    }

    #[test]
    fn test_hash_verification() {
        use sha2::{Digest, Sha256};

        if !PythonRuntime::is_available() {
            eprintln!("Skipping test: Python not available");
            return;
        }

        let runtime = PythonRuntime::new();

        let code = r"
def verify():
    import hashlib
    expected = hashlib.sha256(input_data).digest()
    return expected == output_data
";

        let input = b"test input";
        // Pre-compute SHA256 of "test input"
        let mut hasher = Sha256::new();
        hasher.update(input);
        let hash = hasher.finalize();

        let result = runtime.execute(code, input, &hash);
        assert!(result.is_ok(), "Error: {result:?}");
    }

    #[test]
    fn test_dangerous_import_blocked() {
        if !PythonRuntime::is_available() {
            eprintln!("Skipping test: Python not available");
            return;
        }

        let runtime = PythonRuntime::new();

        let code = r#"
import os
def verify():
    os.system("ls")
    return True
"#;

        let result = runtime.execute(code, b"", b"");
        assert!(result.is_err());
    }
}
