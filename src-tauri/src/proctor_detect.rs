#![allow(unexpected_cfgs)]

use serde::Serialize;
use std::net::TcpStream;
use std::time::Duration;

#[derive(Debug, Clone, Serialize)]
pub struct ProctorScanResult {
    pub running_proctors: Vec<String>,
    pub suspicious_ports: Vec<u16>,
    pub launch_agents: Vec<String>,
}

pub fn full_scan() -> ProctorScanResult {
    ProctorScanResult {
        running_proctors: scan_running_processes(),
        suspicious_ports: scan_proctoring_ports(),
        launch_agents: scan_launch_agents(),
    }
}

fn scan_running_processes() -> Vec<String> {
    let output = std::process::Command::new("ps")
        .args(["-eo", "comm"])
        .output();

    let output = match output {
        Ok(o) => o,
        Err(_) => return Vec::new(),
    };

    let stdout = String::from_utf8_lossy(&output.stdout).to_lowercase();

    let keywords = [
        "proctorfree", "proctoru", "respondus", "lockdown",
        "examsoft", "examplify", "proctorio", "honorlock",
        "exammonitor", "securebrowser", "safeexam",
    ];

    keywords
        .iter()
        .filter(|kw| stdout.contains(**kw))
        .map(|kw| kw.to_string())
        .collect()
}

fn scan_proctoring_ports() -> Vec<u16> {
    let suspect_ports: Vec<u16> = vec![
        11750, 11751, // Respondus LockDown
        23456, 23457, // ProctorU
        18700, 18701, // ExamSoft
        21150, 21151, // Proctorio
    ];

    suspect_ports
        .into_iter()
        .filter(|port| {
            TcpStream::connect_timeout(
                &format!("127.0.0.1:{}", port).parse().unwrap(),
                Duration::from_millis(100),
            )
            .is_ok()
        })
        .collect()
}

fn scan_launch_agents() -> Vec<String> {
    let dirs = [
        "/Library/LaunchDaemons",
        "/Library/LaunchAgents",
    ];

    let keywords = [
        "proctorfree", "proctoru", "respondus", "examsoft",
        "proctorio", "honorlock", "lockdown", "securebrowser",
    ];

    let mut found = Vec::new();

    for dir in &dirs {
        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => continue,
        };

        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_lowercase();
            for kw in &keywords {
                if name.contains(kw) {
                    found.push(entry.path().to_string_lossy().to_string());
                    break;
                }
            }
        }
    }

    found
}
