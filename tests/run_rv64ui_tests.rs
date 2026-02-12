use std::fs;
use std::path::PathBuf;
use std::process::Command;

enum TestResult {
    Pass,
    Fail(String),
    Skipped(String),
}

/// run a single rv64ui test binary and verify it passes
fn run_test_binary(test_path: &str) -> TestResult {
    let output = match Command::new(env!("CARGO_BIN_EXE_riscv-emu"))
        .arg("--elf")
        .arg(test_path)
        .arg("--max-insns")
        .arg("100000") // 100K instructions per test
        .output()
    {
        Ok(out) => out,
        Err(e) => return TestResult::Fail(format!("Failed to run test: {}", e)),
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);

    // Check for [PASS] or [FAIL] markers in the output
    if combined.contains("[PASS]")
        || combined.contains("CPU halted") && !combined.contains("[FAIL]")
    {
        TestResult::Pass
    } else if combined.contains("[FAIL]") {
        TestResult::Fail(
            combined
                .lines()
                .find(|l| l.contains("[FAIL]"))
                .unwrap_or("Test failed")
                .to_string(),
        )
    } else if !output.status.success() {
        TestResult::Fail(format!(
            "exited with code: {}",
            output.status.code().unwrap_or(-1)
        ))
    } else {
        // Assume success if status is 0 and no [FAIL] marker
        TestResult::Pass
    }
}

/// Collect all rv64ui test binaries
fn get_test_binaries() -> Vec<PathBuf> {
    let tests_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/riscv-tests");
    let mut tests = Vec::new();

    match fs::read_dir(&tests_dir) {
        Ok(entries) => {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    let file_name = path.file_name().unwrap().to_string_lossy().to_string();
                    // Match rv64ui test patterns (p and v variants)
                    if file_name.starts_with("rv64ui-p-") || file_name.starts_with("rv64ui-v-") {
                        tests.push(path);
                    }
                }
            }
        }
        Err(e) => {
            eprintln!(
                "Failed to read tests directory {}: {}",
                tests_dir.display(),
                e
            );
            return tests;
        }
    }

    // Sort for deterministic test order
    tests.sort();
    tests
}

#[test]
fn run_all_rv64ui_tests() {
    let test_binaries = get_test_binaries();

    if test_binaries.is_empty() {
        eprintln!("Warning: No rv64ui test binaries found in tests/riscv-tests/");
        return;
    }

    let mut failed_tests = Vec::new();
    let mut passed_tests = 0;
    let mut skipped_tests = 0;

    for test_path in test_binaries {
        let test_name = test_path.file_name().unwrap().to_string_lossy().to_string();
        eprint!("Running {}... ", test_name);

        match run_test_binary(test_path.to_str().unwrap()) {
            TestResult::Pass => {
                eprintln!("✓");
                passed_tests += 1;
            }
            TestResult::Skipped(reason) => {
                eprintln!("⊘ ({})", reason);
                skipped_tests += 1;
            }
            TestResult::Fail(e) => {
                eprintln!("✗ ({})", e);
                failed_tests.push((test_name, e));
            }
        }
    }

    if !failed_tests.is_empty() {
        println!(
            "\n{} passed, {} skipped, {} failed / {} total\n",
            passed_tests,
            skipped_tests,
            failed_tests.len(),
            passed_tests + skipped_tests + failed_tests.len()
        );
        println!("Failed tests:");
        for (name, error) in &failed_tests {
            println!("  - {}: {}", name, error);
        }
        panic!("{} test(s) failed", failed_tests.len());
    }
}
