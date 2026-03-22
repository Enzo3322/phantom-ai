#![allow(unexpected_cfgs)]

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct EnvironmentReport {
    pub hw_model: String,
    pub is_vm: bool,
    pub vm_indicators: Vec<String>,
    pub stealth_score: u8,
}

pub fn generate_report() -> EnvironmentReport {
    let hw_model = get_hw_model();
    let vm_indicators = detect_vm_indicators();
    let is_vm = !vm_indicators.is_empty();

    let stealth_score = calculate_stealth_score(&hw_model, &vm_indicators);

    EnvironmentReport {
        hw_model,
        is_vm,
        vm_indicators,
        stealth_score,
    }
}

fn get_hw_model() -> String {
    let output = std::process::Command::new("sysctl")
        .args(["-n", "hw.model"])
        .output();

    match output {
        Ok(o) => String::from_utf8_lossy(&o.stdout).trim().to_string(),
        Err(_) => "unknown".to_string(),
    }
}

fn detect_vm_indicators() -> Vec<String> {
    let mut indicators = Vec::new();

    // Check hw.model for VM signatures
    let model = get_hw_model().to_lowercase();
    let vm_models = ["vmware", "virtualbox", "parallels", "qemu", "xen"];
    for vm in &vm_models {
        if model.contains(vm) {
            indicators.push(format!("hw.model contains '{}'", vm));
        }
    }

    // Check for VM kexts
    let vm_kexts = [
        "/Library/Extensions/VMwareGfx.kext",
        "/Library/Extensions/VBoxGuest.kext",
        "/Library/Extensions/prl_hypervisor.kext",
    ];
    for kext in &vm_kexts {
        if std::path::Path::new(kext).exists() {
            indicators.push(format!("VM kext found: {}", kext));
        }
    }

    // Check for VM-specific processes
    let output = std::process::Command::new("ps")
        .args(["-eo", "comm"])
        .output();

    if let Ok(o) = output {
        let ps = String::from_utf8_lossy(&o.stdout).to_lowercase();
        let vm_procs = ["vmware-tools", "vboxservice", "prl_tools", "qemu-ga"];
        for proc in &vm_procs {
            if ps.contains(proc) {
                indicators.push(format!("VM process running: {}", proc));
            }
        }
    }

    // Check MAC address for known VM OUIs
    let output = std::process::Command::new("ifconfig")
        .output();

    if let Ok(o) = output {
        let ifconfig = String::from_utf8_lossy(&o.stdout).to_lowercase();
        let vm_ouis = [
            ("00:0c:29", "VMware"),
            ("00:50:56", "VMware"),
            ("08:00:27", "VirtualBox"),
            ("00:1c:42", "Parallels"),
        ];
        for (oui, vendor) in &vm_ouis {
            if ifconfig.contains(oui) {
                indicators.push(format!("{} MAC OUI detected: {}", vendor, oui));
            }
        }
    }

    indicators
}

fn calculate_stealth_score(hw_model: &str, vm_indicators: &[String]) -> u8 {
    let mut score: u8 = 100;

    // VM penalty
    if !vm_indicators.is_empty() {
        score = score.saturating_sub(30);
        score = score.saturating_sub((vm_indicators.len() as u8).saturating_mul(5));
    }

    // Non-Apple Silicon penalty (less common, more suspicious)
    if !hw_model.starts_with("Mac") {
        score = score.saturating_sub(10);
    }

    score
}
