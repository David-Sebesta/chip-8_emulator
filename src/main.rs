use egui_dock::{DockArea, DockState};

mod chip8;
use chip8::Chip8;

use crate::gui::{Chip8Tab, Chip8TabViewer};

mod gui;

struct Chip8Timing {
    pub cpu_speed_hz: f32,
    pub timer_speed_hz: f32,
    pub total_speed_mod: f32,
    pub cpu_accumulator: f32,
    pub timer_accumulator: f32,
}

impl Default for Chip8Timing {
    fn default() -> Self {
        Self {
            cpu_speed_hz: 700.0,
            timer_speed_hz: 60.0,
            total_speed_mod: 1.0,
            cpu_accumulator: 0.0,
            timer_accumulator: 0.0,
        }
    }
}

struct Chip8App {
    chip8: Chip8,
    dock_state: DockState<Chip8Tab>,
    texture: Option<egui::TextureHandle>,
    timing: Chip8Timing,
    rom_loaded: bool,
}

impl Default for Chip8App {
    fn default() -> Self {
        let mut chip8 = Chip8::new();
        let mut rom_loaded = false;
        // Load your ROM here (Desktop only)
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Ok(rom) = std::fs::read("D:\\RustProjects\\chip_8_emulator\\test_roms\\6-keypad.ch8") {
                rom_loaded = chip8.load_rom(&rom);
            }
        }

        let mut dock_state = DockState::new(vec![Chip8Tab::CentralDisplay]);
        
        // Split to the left
        let [_right_node, left_panel] = dock_state.main_surface_mut().split_left(
            egui_dock::NodeIndex::root(),
            0.33,
            vec![Chip8Tab::Controls, Chip8Tab::MemoryViewer]
        );

        // Registers below
        dock_state.main_surface_mut().split_below(
            left_panel, 
            0.5, // 50% height of the left side
            vec![Chip8Tab::Registers]
        );



        Self {
            chip8,
            dock_state,
            texture: None,
            timing: Chip8Timing::default(),
            rom_loaded,
        }
    }
}

impl eframe::App for Chip8App {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        eframe::egui::CentralPanel::default().show(ctx, |ui| {
            if self.rom_loaded {
                let dt = ctx.input(|i| i.stable_dt);
    
                self.timing.cpu_accumulator += dt;
                let cpu_interval = 1.0 / (self.timing.cpu_speed_hz * self.timing.total_speed_mod);
                while self.timing.cpu_accumulator >= cpu_interval {
                    self.chip8.tick();
                    self.timing.cpu_accumulator -= cpu_interval;
                }
    
                self.timing.timer_accumulator += dt;
                let timer_interval = 1.0 / (self.timing.timer_speed_hz * self.timing.total_speed_mod);
                while self.timing.timer_accumulator >= timer_interval {
                    self.chip8.update_timers();
                    self.timing.timer_accumulator -= timer_interval;
                }
    
                self.handle_input(ctx);
            }

            egui::CentralPanel::default().show(ctx, |ui| {
                            let mut viewer = Chip8TabViewer {
                                chip8: &mut self.chip8,
                                texture_handle: &mut self.texture,
                                timing: &mut self.timing,
                            };
                
                            DockArea::new(&mut self.dock_state)
                                .style(egui_dock::Style::from_egui(ui.style()))
                                .show_inside(ui, &mut viewer);
            });

            ctx.request_repaint();
        });
    }

}

impl Chip8App {
    fn handle_input(&mut self, ctx: &egui::Context) {
        ctx.input(|i| {
            self.chip8.keys[0x1] = i.key_down(egui::Key::Num1);
            self.chip8.keys[0x2] = i.key_down(egui::Key::Num2);
            self.chip8.keys[0x3] = i.key_down(egui::Key::Num3);
            self.chip8.keys[0xC] = i.key_down(egui::Key::Num4);
            self.chip8.keys[0x4] = i.key_down(egui::Key::Q);
            self.chip8.keys[0x5] = i.key_down(egui::Key::W);
            self.chip8.keys[0x6] = i.key_down(egui::Key::E);
            self.chip8.keys[0xD] = i.key_down(egui::Key::R);
            self.chip8.keys[0x7] = i.key_down(egui::Key::A);
            self.chip8.keys[0x8] = i.key_down(egui::Key::S);
            self.chip8.keys[0x9] = i.key_down(egui::Key::D);
            self.chip8.keys[0xE] = i.key_down(egui::Key::F);
            self.chip8.keys[0xA] = i.key_down(egui::Key::Z);
            self.chip8.keys[0x0] = i.key_down(egui::Key::X);
            self.chip8.keys[0xB] = i.key_down(egui::Key::C);
            self.chip8.keys[0xF] = i.key_down(egui::Key::V);
        });
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    // Desktop main
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };
    eframe::run_native(
        "CHIP-8 Emulator",
        native_options,
        Box::new(|_cc| Ok(Box::new(Chip8App::default()))),
    )
}

#[cfg(target_arch = "wasm32")]
fn main() {
    use wasm_bindgen::JsCast;

    // Set up panic hook for better error messages in the browser console
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    // Redirect `log` message to `console.log` and friends
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .expect("No window")
            .document()
            .expect("No document");

        let canvas = document
            .get_element_by_id("the_canvas_id")
            .expect("Failed to find the_canvas_id")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("the_canvas_id was not a HtmlCanvasElement");

        let start_result = eframe::WebRunner::new()
            .start(
                canvas, 
                web_options,
                Box::new(|_cc| Ok(Box::new(Chip8App::default()))),
            )
            .await;
            
        if let Err(e) = start_result {
            log::error!("Failed to start eframe: {:?}", e);
        }
    });
}