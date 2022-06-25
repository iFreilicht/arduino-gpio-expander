use gpio_actions::{Action, PinState};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize, Default)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {}

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

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let action = Action::Output('4', PinState::High);
        let serialized_action = postcard::to_stdvec(&action).expect("Failed to serialize action!");
        let deserialized_action: Action = postcard::from_bytes(&serialized_action).expect("Failed to deserialize!");

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Serial GUI");
            egui::warn_if_debug_build(ui);
            ui.heading("Action object");
            ui.label(format!("{:?}", action));
            ui.heading("in hex:");
            ui.label(format!("{:x?}", serialized_action));
            ui.heading("deserialized again:");
            ui.label(format!("{:?}", deserialized_action));
        });
    }
}
