use std::{
    collections::{HashMap, VecDeque},
    io::{Read, Write},
    time::Duration,
};

use egui::{mutex::Mutex, ComboBox, TextEdit};
use gpio_actions::{Action, PinLabel, PinName, PinState, Response, TryFromIter, MAX_RESPONSE_WIRE_SIZE};
use serialport::{SerialPort, SerialPortInfo};

#[derive(serde::Deserialize, serde::Serialize, Default, Debug, PartialEq, Eq, PartialOrd)]
enum ActionType {
    #[default]
    Output,
    Input,
    List,
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize, Default)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    selected_action_type: ActionType,
    pin_label: String,
    pin_high: bool,
    #[serde(skip)]
    serial_port: Mutex<Option<serialport::TTYPort>>,
    #[serde(skip)]
    serial_responses: VecDeque<Response>,
    #[serde(skip)]
    bytes_read: usize,
    #[serde(skip)]
    pin_map: HashMap<PinLabel, PinName>,
}

const DEFAULT_PIN_LABEL: char = '?';

struct SerialIter<'a> {
    port: &'a mut serialport::TTYPort,
    bytes_read: usize,
}

impl<'a> SerialIter<'a> {
    pub fn new(port: &'a mut serialport::TTYPort) -> Self {
        Self { port, bytes_read: 0 }
    }
}

impl<'a> Iterator for SerialIter<'a> {
    type Item = u8;
    fn next(&mut self) -> Option<Self::Item> {
        let mut buf = [0_u8; 1];
        if self.port.read_exact(&mut buf).is_err() {
            return None;
        }
        self.bytes_read += 1;
        Some(buf[0])
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customized the look at feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }

    fn read_from_serial(&mut self, lines: usize) {
        let mut port_mutex_guard = self.serial_port.lock();
        let serial_port = port_mutex_guard.as_mut().expect("Not connected to serial port?");

        let mut serial_iter = SerialIter::new(serial_port);

        let response_result = Response::try_from_iter::<MAX_RESPONSE_WIRE_SIZE>(&mut serial_iter);
        self.bytes_read += serial_iter.bytes_read;

        if let Ok(response) = response_result {
            self.serial_responses.push_back(response);
            while self.serial_responses.len() > lines {
                self.serial_responses.pop_front();
            }

            if let Response::List(label, name) = response {
                self.pin_map.insert(label, name);
            }
        }
    }

    fn connect(&mut self, port: SerialPortInfo) {
        let tty_port = serialport::new(port.port_name, 57600)
            .timeout(Duration::from_millis(10))
            .open_native()
            .expect("Failed to open serial port!");

        self.serial_port = Mutex::new(Some(tty_port));
    }

    fn send_action(&self, action: Action) {
        let mut port_mutex_guard = self.serial_port.lock();
        let serial_port = port_mutex_guard.as_mut().expect("Not connected to serial port?");

        let serialized_list = postcard::to_stdvec(&action).expect("Failed to serialize action!");
        serial_port.write_all(&serialized_list).expect("Failed to send action!");
    }

    fn build_pin_list(&mut self, ui: &mut egui::Ui) {
        if self.pin_map.is_empty() {
            self.send_action(Action::List);
            return;
        }

        ui.vertical(|ui| {
            for (&pin_label, &pin_name) in &self.pin_map {
                ui.horizontal(|ui| {
                    ui.heading(String::from(pin_name));
                    ui.label(String::from(pin_label));
                    if ui.button("Set High").clicked() {
                        self.send_action(Action::Output(pin_label, PinState::High));
                    }
                    if ui.button("Set Low").clicked() {
                        self.send_action(Action::Output(pin_label, PinState::Low));
                    }
                    if ui.button("Input").clicked() {
                        self.send_action(Action::Output(pin_label, PinState::Low));
                    }
                });
            }
        });
    }

    fn serial_output_text(&mut self, ui: &mut egui::Ui, lines: usize) {
        self.read_from_serial(lines);

        ui.label(format!("Bytes read: {}", self.bytes_read));
        for response in &self.serial_responses {
            ui.label(format!("{:?}", response));
        }
    }
}

fn single_character_text<S>(ui: &mut egui::Ui, text: &mut S)
where
    S: egui::TextBuffer,
{
    text.delete_char_range(1..usize::MAX);
    ui.add(
        TextEdit::singleline(text)
            .hint_text(DEFAULT_PIN_LABEL.to_string())
            .desired_width(10.0),
    );
}

fn format_port(port: &SerialPortInfo) -> String {
    let path = port.port_name.clone();
    let name;
    if let serialport::SerialPortType::UsbPort(port_info) = port.port_type.clone() {
        name = port_info.product.unwrap_or_default();
    } else {
        name = format!("{:?}", port.port_type)
    }
    format!("{} ({})", path, name)
}

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Serial GUI");
            egui::warn_if_debug_build(ui);

            ui.horizontal(|ui| {
                ComboBox::from_label("")
                    .selected_text(format!("{:?}", self.selected_action_type))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.selected_action_type, ActionType::Output, "Output");
                        ui.selectable_value(&mut self.selected_action_type, ActionType::Input, "Input");
                        ui.selectable_value(&mut self.selected_action_type, ActionType::List, "List");
                    });

                match self.selected_action_type {
                    ActionType::Output => {
                        single_character_text(ui, &mut self.pin_label);
                        ui.checkbox(&mut self.pin_high, "Set pin high");
                    }
                    ActionType::Input => {
                        single_character_text(ui, &mut self.pin_label);
                    }
                    ActionType::List => (),
                };
            });

            let pin_label = self.pin_label.chars().next().unwrap_or(DEFAULT_PIN_LABEL);

            let action = match self.selected_action_type {
                ActionType::Output => {
                    Action::Output(pin_label, if self.pin_high { PinState::High } else { PinState::Low })
                }
                ActionType::Input => Action::Input(pin_label),
                ActionType::List => Action::List,
            };

            let serialized_action = postcard::to_stdvec(&action).expect("Failed to serialize action!");
            let deserialized_action: Action = postcard::from_bytes(&serialized_action).expect("Failed to deserialize!");
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.heading("Action object");
                    ui.label(format!("{:?}", action));
                });
                ui.vertical(|ui| {
                    ui.heading("in hex:");
                    ui.label(format!("{:02x?}", serialized_action));
                });
                ui.vertical(|ui| {
                    ui.heading("deserialized again:");
                    ui.label(format!("{:?}", deserialized_action))
                });
            });

            let mut disconnect = false;
            if self.serial_port.lock().is_some() {
                ui.heading("Serial connection");
                ui.horizontal(|ui| {
                    ui.label(format!(
                        "Connected to {}",
                        self.serial_port.lock().as_ref().unwrap().name().unwrap_or_default()
                    ));
                    if ui.button("Disconnect").clicked() {
                        disconnect = true;
                    }
                });

                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        if ui.button("Send action").clicked() {
                            self.send_action(action)
                        }

                        self.serial_output_text(ui, 30);
                    });

                    self.build_pin_list(ui)
                });
            } else {
                ui.heading("Serial ports");
                let ports = serialport::available_ports().expect("No serial ports found!");
                for port in ports {
                    ui.horizontal(|ui| {
                        ui.label(format_port(&port));
                        if let serialport::SerialPortType::UsbPort(_) = port.port_type.clone() {
                            if ui.button("Connect").clicked() {
                                self.connect(port)
                            }
                        };
                    });
                }
            }
            if disconnect {
                self.serial_responses = Default::default();
                self.pin_map = Default::default();
                self.bytes_read = 0;
                self.serial_port = Default::default();
            }
        });
    }
}
