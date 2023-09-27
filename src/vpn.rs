use crate::log;
use std::process::{Command, Output};

fn command() -> Command {
    Command::new("nordvpn")
}

const X: &[char] = &[' ', '\n', '\r', '-'];
pub const DEFAULT_COUNTRY: &str = "Netherlands";

pub fn connect(target: String) -> bool {
    match command()
        .arg("connect")
        .arg(target.replace(" ", "_"))
        .output()
    {
        Ok(output) => output.status.success(),
        Err(error) => {
            log::error(&format!("Failed to connect: {:?}", error));
            false
        }
    }
}

pub fn disconnect() -> bool {
    match command().arg("disconnect").output() {
        Ok(output) => output.status.success(),
        Err(error) => {
            log::error(&format!("Failed to disconnect: {:?}", error));
            false
        }
    }
}

pub fn status_details() -> Option<Vec<(String, String)>> {
    match command().arg("status").output() {
        Ok(output) => Some(if output.status.success() {
            format_kv(output)
        } else {
            vec![
                ("Status".into(), "Disconnected".into()),
                ("Country".into(), DEFAULT_COUNTRY.into()),
            ]
        }),
        Err(error) => {
            log::error(&format!("Failed to get status: {:?}", error));
            None
        }
    }
}

pub fn countries() -> Option<Vec<String>> {
    match command().arg("countries").output() {
        Ok(output) => Some(format_list(output)),
        Err(error) => {
            log::error(&format!("Failed to get countries: {:?}", error));
            None
        }
    }
}

pub fn groups() -> Option<Vec<String>> {
    match command().arg("groups").output() {
        Ok(output) => Some(format_list(output)),
        Err(error) => {
            log::error(&format!("Failed to get groups: {:?}", error));
            None
        }
    }
}

pub fn settings() -> Option<Vec<(String, String)>> {
    match command().arg("settings").output() {
        Ok(output) => Some(format_kv(output)),
        Err(error) => {
            log::error(&format!("Failed to get settings: {:?}", error));
            None
        }
    }
}

pub fn set_settings(key: String, value: String) -> bool {
    let key = key.to_lowercase();
    let mut args: Vec<String> = vec!["set".into()];
    let option = if value == "enabled" { "off" } else { "on" };

    if key == "dns" {
        args.push(key);
        args.push("103.86.96.100".into());
        args.push("103.86.99.100".into());
    } else if key == "lan discovery" {
        args.push("lan-discovery".into());
        args.push(option.into());
    } else {
        args.push(key.replace(" ", ""));
        args.push(option.into());
    }

    match command().args(args).output() {
        Ok(output) => output.status.success(),
        Err(error) => {
            log::error(&format!("Failed to set settings: {:?}", error));
            false
        }
    }
}

fn format_kv(output: Output) -> Vec<(String, String)> {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut result = Vec::new();

    for line in stdout.lines() {
        if let Some((key, value)) = line.trim_start_matches(X).split_once(":") {
            result.push((key.into(), value.trim().into()));
        }
    }

    result
}

fn format_list(output: Output) -> Vec<String> {
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout
        .trim_start_matches(X)
        .replace("_", " ")
        .split(',')
        .map(|x| x.trim().into())
        .collect::<Vec<_>>()
}
