use ksni::{self, Tray};
use std::thread;
use vpn::DEFAULT_COUNTRY;
mod log;
mod vpn;

#[derive(Debug)]
struct VpnTray {
    countries: Vec<String>,
    groups: Vec<String>,
    target_index: usize,
    use_country: bool,
    connected: bool,
    connectivity_label: String,
    status_details: Vec<(String, String)>,
    settings: Vec<(String, String)>,
}

impl VpnTray {
    fn connect(&mut self, index: usize, use_country: bool) {
        let mut target = String::new();
        if use_country {
            target.push_str(&self.countries[index]);
            self.target_index = index;
        } else {
            target.push_str(&self.groups[index]);
            self.target_index = index;
        }

        if vpn::connect(target.clone()) {
            self.connected = true;
            self.use_country = use_country;
            self.connectivity_label = "Disconnect".into();
        }
    }

    fn disconnect(&mut self) {
        if vpn::disconnect() {
            self.connected = false;
            self.connectivity_label = "Connect".into();
        }
    }

    fn change_connectivity_state(&mut self) {
        if self.connected {
            self.disconnect();
        } else {
            self.connect(self.target_index, self.use_country);
        }
    }

    fn change_status(&mut self, status: bool) {
        self.connected = status;
        if status {
            self.connectivity_label = "Disconnect".into();
        } else {
            self.connectivity_label = "Connect".into();
        }
        self.icon_name();
    }

    fn change_settings(&mut self, index: usize) {
        let (setting_key, setting_value) = &self.settings[index];

        if vpn::set_settings(setting_key.clone(), setting_value.clone()) {
            self.settings[index].1 = match setting_value.as_str() {
                "enabled" => "disabled".into(),
                "disabled" => "enabled".into(),
                _ => setting_value.clone(),
            };

            self.settings = vpn::settings().unwrap_or_default();
        }
    }
}

impl ksni::Tray for VpnTray {
    fn icon_name(&self) -> String {
        if self.connected {
            "emblem-default".into()
        } else {
            "face-monkey".into()
        }
    }

    fn id(&self) -> String {
        env!("CARGO_PKG_NAME").into()
    }

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        use ksni::menu::*;
        vec![
            SubMenu {
                label: "Countries".into(),
                submenu: vec![RadioGroup {
                    selected: if self.use_country {
                        self.target_index
                    } else {
                        0
                    },
                    select: Box::new(|this: &mut Self, current| {
                        this.connect(current, true);
                    }),
                    options: {
                        let mut result = vec![];
                        for country in self.countries.iter() {
                            result.push(RadioItem {
                                label: country.clone(),
                                ..Default::default()
                            });
                        }
                        result
                    },
                }
                .into()],
                ..Default::default()
            }
            .into(),
            SubMenu {
                label: "Groups".into(),
                submenu: vec![RadioGroup {
                    selected: if !self.use_country {
                        self.target_index
                    } else {
                        0
                    },
                    select: Box::new(|this: &mut Self, current| {
                        this.connect(current, false);
                    }),
                    options: {
                        let mut result = vec![];
                        for group in self.groups.iter() {
                            result.push(RadioItem {
                                label: group.clone(),
                                ..Default::default()
                            });
                        }
                        result
                    },
                }
                .into()],
                ..Default::default()
            }
            .into(),
            SubMenu {
                label: "Connection info".into(),
                submenu: {
                    let mut result = vec![];
                    for (key, value) in self.status_details.iter() {
                        result.push(
                            StandardItem {
                                enabled: false,
                                label: format!("{}: {}", key, value),
                                ..Default::default()
                            }
                            .into(),
                        );
                    }
                    result
                },
                ..Default::default()
            }
            .into(),
            SubMenu {
                label: "Settings".into(),
                submenu: {
                    let mut result = vec![];
                    for (index, (key, value)) in self.settings.iter().enumerate() {
                        result.push(
                            CheckmarkItem {
                                enabled: (value == "enabled" || value == "disabled"),
                                checked: value == "enabled",
                                label: format!("{}: {}", key, value),
                                activate: Box::new(move |this: &mut Self| {
                                    this.change_settings(index);
                                }),
                                ..Default::default()
                            }
                            .into(),
                        );
                    }
                    result
                },
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: self.connectivity_label.clone(),
                activate: Box::new(|this: &mut Self| {
                    this.change_connectivity_state();
                }),
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: "Exit".into(),
                activate: Box::new(|_| std::process::exit(0)),
                ..Default::default()
            }
            .into(),
        ]
    }
}

fn get_details() -> (bool, Option<Vec<(String, String)>>) {
    if let Some(status_details) = vpn::status_details() {
        let status = status_details
            .iter()
            .any(|(key, value)| key.contains("Status") && value == "Connected");
        return (status, Some(status_details));
    }

    (false, None)
}

fn get_countries(details: Option<Vec<(String, String)>>) -> (usize, String, Vec<String>) {
    if let Some(details) = details {
        if let Some((_, default_country)) = details.iter().find(|(key, _)| key == "Country") {
            if let Some(countries) = vpn::countries() {
                if let Some(country_index) = countries.iter().position(|x| x == default_country) {
                    return (country_index, default_country.clone(), countries);
                }
            }
        }
    }

    (0, DEFAULT_COUNTRY.into(), vec![DEFAULT_COUNTRY.into()])
}

fn get_settings() -> Vec<(String, String)> {
    if let Some(settings) = vpn::settings() {
        return settings;
    }

    vec![]
}

fn get_groups() -> Vec<String> {
    if let Some(groups) = vpn::groups() {
        return groups;
    }

    vec![]
}

fn main() {
    let (connected, status_details) = get_details();

    if !connected {
        log::error("Failed to connect. Please ensure that the NordVPN daemon is running.");
    }

    let (country_index, _, countries) = get_countries(status_details.clone());

    let vpn_tray = VpnTray {
        connected,
        connectivity_label: if connected {
            "Disconnect".into()
        } else {
            "Connect".into()
        },
        use_country: true,
        target_index: country_index,
        countries,
        status_details: status_details.unwrap_or(vec![]),
        groups: get_groups(),
        settings: get_settings(),
    };

    let service = ksni::TrayService::new(vpn_tray);
    let handle = service.handle();

    service.spawn();
    thread::sleep(std::time::Duration::from_secs(3));

    thread::spawn(move || loop {
        handle.update(|this: &mut VpnTray| {
            let (connected, status_details) = get_details();

            if this.connected != connected {
                this.change_status(connected);

                let (country_index, _, countries) = get_countries(status_details.clone());

                this.target_index = country_index;
                this.countries = countries;
                this.settings = get_settings();
                this.groups = get_groups();
            }

            this.status_details = status_details.unwrap_or(vec![]);
        });
        thread::sleep(std::time::Duration::from_secs(3));
    })
    .join()
    .unwrap();

    loop {
        std::thread::park();
    }
}
