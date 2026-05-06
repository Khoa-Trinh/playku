use eframe::egui;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([440.0, 240.0])
            .with_resizable(false)
            .with_title("Playku Launcher"),
        ..Default::default()
    };
    eframe::run_native(
        "Playku Launcher",
        options,
        Box::new(|cc| {
            setup_custom_styles(&cc.egui_ctx);
            Box::new(PlaykuApp::default())
        }),
    )
}

struct PlaykuApp {
    url: String,
    always_on_top: bool,
    status_message: String,
}

impl Default for PlaykuApp {
    fn default() -> Self {
        Self {
            url: String::new(),
            always_on_top: true,
            status_message: String::new(),
        }
    }
}

fn setup_custom_styles(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = egui::Color32::from_rgb(9, 9, 11); // Deep OLED Black (Zinc-950)
    visuals.window_fill = egui::Color32::from_rgb(24, 24, 27); // Matte Charcoal Card (Zinc-900)
    
    // Inactive widget states (input box background, checkboxes)
    visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(9, 9, 11);
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(63, 63, 70)); // Zinc-700 border
    visuals.widgets.inactive.rounding = 12.0.into(); // Ultra-rounded inputs
    
    // Hover widget states
    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(39, 39, 42); // Zinc-800
    visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.2, egui::Color32::from_rgb(161, 161, 170)); // Zinc-400
    visuals.widgets.hovered.rounding = 12.0.into();
    
    // Active / Pressed states
    visuals.widgets.active.bg_fill = egui::Color32::WHITE;
    visuals.widgets.active.fg_stroke = egui::Stroke::new(1.5, egui::Color32::WHITE);
    visuals.widgets.active.rounding = 12.0.into();
    
    visuals.selection.bg_fill = egui::Color32::from_rgb(63, 63, 70); // Modern slate-700 selection
    ctx.set_visuals(visuals);
    
    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(10.0, 12.0);
    style.spacing.button_padding = egui::vec2(24.0, 10.0);
    ctx.set_style(style);
}

impl eframe::App for PlaykuApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(10.0);
                
                // Floating Matte Card Panel with high rounding & deep padding
                let card_frame = egui::Frame::none()
                    .fill(egui::Color32::from_rgb(24, 24, 27)) // Matte Charcoal (Zinc-900)
                    .inner_margin(24.0) // Deep inner padding
                    .rounding(20.0) // Large modern rounded corners
                    .shadow(egui::epaint::Shadow {
                        offset: [0.0, 6.0].into(),
                        blur: 20.0,
                        spread: 0.0,
                        color: egui::Color32::from_black_alpha(150),
                    });
                
                card_frame.show(ui, |ui| {
                    ui.set_max_width(390.0);
                    ui.vertical_centered(|ui| {
                        // Header Layout (Title left, Toggle right)
                        ui.horizontal(|ui| {
                            ui.heading(
                                egui::RichText::new("⚡ PLAYKU")
                                    .size(22.0)
                                    .strong()
                                    .color(egui::Color32::WHITE)
                            );
                            
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.checkbox(&mut self.always_on_top, "Always on top");
                            });
                        });
                        
                        ui.add_space(8.0);
                        
                        // URL Input (OLED Black background)
                        let url_input = egui::TextEdit::singleline(&mut self.url)
                            .hint_text("Enter video or YouTube URL...")
                            .margin(egui::vec2(12.0, 10.0))
                            .desired_width(340.0);
                        
                        ui.add(url_input);
                        
                        ui.add_space(10.0);
                        
                        // High Contrast Apple/Vercel Matte White Play Button
                        let play_button = egui::Button::new(
                            egui::RichText::new("▶  PLAY VIDEO")
                                .size(13.0)
                                .strong()
                                .color(egui::Color32::from_rgb(9, 9, 11)) // Pitch black text
                        )
                        .fill(egui::Color32::from_rgb(244, 244, 245)) // Pure white-ish (Zinc-100)
                        .min_size(egui::vec2(340.0, 36.0));
                        
                        if ui.add(play_button).clicked() {
                            let trimmed_url = self.url.trim();
                            if trimmed_url.is_empty() {
                                self.status_message = "❌ Please enter a valid URL!".to_string();
                            } else {
                                // Resolve path to playku-engine process
                                let mut exe_path = std::env::current_exe().expect("Failed to get executable path");
                                exe_path.pop();
                                
                                let engine_binary = if cfg!(target_os = "windows") {
                                    "playku-engine.exe"
                                } else {
                                    "playku-engine"
                                };
                                let engine_path = exe_path.join(engine_binary);
                                
                                let mut cmd = std::process::Command::new(&engine_path);
                                if self.always_on_top {
                                    cmd.arg("--ontop");
                                }
                                cmd.arg(trimmed_url);
                                
                                match cmd.spawn() {
                                    Ok(_) => {
                                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                                    }
                                    Err(e) => {
                                        self.status_message = format!("❌ Engine start failed: {}", e);
                                    }
                                }
                            }
                        }
                        
                        // Status message area
                        if !self.status_message.is_empty() {
                            ui.add_space(6.0);
                            ui.label(
                                egui::RichText::new(&self.status_message)
                                    .size(11.0)
                                    .color(egui::Color32::from_rgb(248, 113, 113)) // red-400
                            );
                        }
                    });
                });
            });
        });
    }
}
