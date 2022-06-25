use egui::{ComboBox, TextEdit};
use gpio_actions::{Action, PinState};

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
}

const DEFAULT_PIN_LABEL: char = '?';

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

            let ports = serialport::available_ports().expect("No serial ports found!");
            ui.collapsing("Serial ports", |ui| ui.label(format!("{:#?}", ports)));
        });
    }
}
