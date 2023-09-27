extern crate syslog;
use syslog::{Facility, Formatter3164};

pub fn error(message: &str) {
    let formatter = Formatter3164 {
        facility: Facility::LOG_LOCAL0,
        hostname: None,
        process: "nordvpn_tray_app".into(),
        pid: std::process::id(),
    };
    let mut writer = syslog::unix(formatter).expect("could not connect to syslog");
    writer
        .err(message)
        .expect("could not write error message to syslog");
}
