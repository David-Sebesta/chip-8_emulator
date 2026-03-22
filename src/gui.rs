use std::{fmt::format, sync::atomic::AtomicBool};

use eframe::egui;
use egui::{Color32, RichText, epaint::text};
use egui_dock::TabViewer;
use rfd::AsyncFileDialog;
use tokio;

use crate::{Chip8, Chip8Timing};

pub enum Chip8Tab {
    CentralDisplay,
    Controls,
    Registers,
    MemoryViewer,
    InstructionHistory,
}

pub struct Chip8TabViewer<'a> {
    pub chip8: &'a mut Chip8,
    pub texture_handle: &'a mut Option<egui::TextureHandle>,
    pub timing: &'a mut Chip8Timing,
    pub is_paused: &'a mut AtomicBool,
}

impl<'a> egui_dock::TabViewer for Chip8TabViewer<'a> {
    type Tab = Chip8Tab;

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab {
            Chip8Tab::CentralDisplay => {
                // 64x32 is Chip8 display size, so keep it to that ratio
                let available = ui.available_size();
                let current_ratio = available.x / available.y;

                let (width, height) = if current_ratio > 2.0 {
                    (available.y * 2.0, available.y)
                } else {
                    (available.x, available.x * 0.5)
                };

                let (rect, _) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());

                ui.painter().rect_filled(rect, 0.0, egui::Color32::DARK_GRAY);

                let texture = self.texture_handle.get_or_insert_with(|| {
                    let initial_pixels = vec![Color32::BLACK; 64 * 32];
                    ui.ctx().load_texture(
                        "chip-8_screen",
                        egui::ColorImage::new([64, 32], initial_pixels),
                        egui::TextureOptions::NEAREST
                    )
                });

                let pixels: Vec<u8> = self.chip8.display
                    .iter()
                    .map(|&p| if p {255} else {0})
                    .collect();

                let image = egui::ColorImage::from_gray([64, 32], &pixels);
                texture.set(image, egui::TextureOptions::NEAREST);

                ui.painter().image(
                    texture.id(), 
                    rect, 
                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), 
                    egui::Color32::WHITE
                );

            },
            Chip8Tab::Controls => {

                egui::ScrollArea::both().show(ui,|ui| {
                    ui.group(|ui| {
                        if ui.button(RichText::new("Load ROM").strong()).clicked() {
                            #[cfg(target_arch = "wasm32")] {
                                wasm_bindgen_futures::spawn_local(self.upload_file());
                            }
                            #[cfg(not(target_arch = "wasm32"))] {
                                pollster::block_on(self.upload_file());
                            } 

                        }


                        ui.label(RichText::new("Timing").strong());
                        
                        // Current CPU hz and timer hz
                        egui::Grid::new("current_hz")
                            .num_columns(2)
                            .spacing([10.0, 4.0])
                            .show(ui, |ui| {
                                ui.label("Current CPU Hz:");
                                ui.label(RichText::new(format!("{:} Hz", self.timing.cpu_speed_hz * self.timing.total_speed_mod)).monospace());
                                ui.end_row();

                                ui.label("Current Timer Hz:");
                                ui.label(RichText::new(format!("{:} Hz", self.timing.timer_speed_hz * self.timing.total_speed_mod)).monospace());
                                ui.end_row();
                            });

                        ui.add_space(10.0);

                        // Speed adjusters
                        egui::Grid::new("adjust_hz")
                            .num_columns(2)
                            .spacing([10.0, 4.0])
                            .show(ui, |ui| {
                                ui.label("CPU Hz:");
                                ui.add(
                                    egui::Slider::new( &mut self.timing.cpu_speed_hz, 0.0..=1000.0)
                                    .suffix(" Hz")
                                    .show_value(true)
                                );
                                ui.end_row();

                                ui.label("Timer Hz:");
                                ui.add(
                                    egui::Slider::new( &mut self.timing.timer_speed_hz, 0.0..=120.0)
                                    .suffix(" Hz")
                                    .show_value(true)
                                );
                                ui.end_row();

                                ui.label("Total Modifier:");
                                ui.add(
                                    egui::Slider::new( &mut self.timing.total_speed_mod, 0.0..=3.0)
                                    .step_by(0.1)
                                    .show_value(true)
                                );
                                ui.end_row();

                                if ui.button("Reset").clicked() {
                                    self.timing.cpu_speed_hz = 700.0;
                                    self.timing.timer_speed_hz = 60.0;
                                    self.timing.total_speed_mod = 1.0;
                                }

                            });

                    });
                });


            },
            Chip8Tab::Registers => {
                // Registers
                egui::ScrollArea::both().show(ui,|ui| {
                    ui.group(|ui| {
                        ui.label(RichText::new("Special Registers").strong());
                        egui::Grid::new("special_regs")
                            .num_columns(3)
                            .spacing([40.0, 4.0])
                            .striped(true)
                            .show(ui, |ui| {
                            ui.label("PC:");
                            ui.label(RichText::new(format!("0x{:03X}", self.chip8.pc())).monospace());
                            ui.label(RichText::new(format!("0b{:012b}", self.chip8.pc())).monospace());
                            ui.end_row();

                            ui.label("I:");
                            ui.label(RichText::new(format!("0x{:03X}", self.chip8.i())).monospace());
                            ui.label(RichText::new(format!("0b{:012b}", self.chip8.i())).monospace());
                            ui.end_row();

                            ui.label("SP:");
                            ui.label(RichText::new(format!("0x{:02X}", self.chip8.sp())).monospace());
                            ui.label(RichText::new(format!("0b{:08b}", self.chip8.sp())).monospace());
                            ui.end_row();

                            ui.label("DT:");
                            ui.label(RichText::new(format!("{:03}", self.chip8.dt())).monospace());
                            ui.end_row();

                            ui.label("ST:");
                            ui.label(RichText::new(format!("{:03}", self.chip8.st())).monospace());
                            ui.end_row();
                        });

                        
                        
                    });

                    ui.add_space(10.0);
                
                    let v = self.chip8.registers();
                    
                    ui.group(|ui| {
                        ui.label(RichText::new("General Registers").strong());
                        egui::Grid::new("v_regs")
                            .num_columns(6)
                            .spacing([10.0, 4.0])
                            .striped(true)
                            .show(ui, |ui| {
                                for i in 0..8 {
                                    // Left 0-8
                                    ui.label(format!("V{:X}:", i));
                                    ui.label(RichText::new(format!("0x{:02X}", v[i])).monospace());
                                    ui.label(RichText::new(format!("0b{:08b}", v[i])).monospace());
                                
                                    // Right
                                    let j = i + 8;
                                    ui.label(format!("V{:X}:", j));
                                    ui.label(RichText::new(format!("0x{:02X}", v[j])).monospace());
                                    ui.label(RichText::new(format!("0b{:08b}", v[j])).monospace());
                                    ui.end_row();
                                }
                        });
                    });
                
                });


            },
            Chip8Tab::MemoryViewer => {
                
                egui::ScrollArea::both().show(ui, |ui| {
                    ui.group(|ui| {
                        // Follow program counter
                        // ui.checkbox(checked, atoms)

                        let memory = self.chip8.memory();
                        let pc = self.chip8.pc();

                        egui::Grid::new("memory_grid")
                            .num_columns(16)
                            .spacing([0.0, 0.0])
                            .striped(true)
                            .show(ui, |ui| {
                                for row in (0..memory.len()).step_by(16) {
                                    ui.label(format!("0x{:0000X} ", row));
                                    for col in 0..16 {
                                        ui.label(RichText::new(format!("{:0X} ", memory[row+col]))
                                            .background_color(if row+col == pc as usize {Color32::WHITE} else {Color32::default()})
                                            .monospace()
                                        );
                                    }
                                    ui.end_row();
                                }
                            });

                    });
                });
            },
            Chip8Tab::InstructionHistory => {
                egui::ScrollArea::both().show(ui, |ui| {
                    ui.group(|ui| {
                        let instuctions = self.chip8.instruction_history();
                        for instruction in instuctions {
                            ui.label(RichText::new(format!("{:?}", instruction)).monospace());
                        }
                    });
                });
            }
        }
    }

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        match tab {
            Chip8Tab::CentralDisplay => "Chip 8".into(),
            Chip8Tab::Controls => "Controls".into(),
            Chip8Tab::Registers => "Registers".into(),
            Chip8Tab::MemoryViewer => "Memory".into(),
            Chip8Tab::InstructionHistory => "Instruction History".into(),
        }
    }

    fn clear_background(&self, _tab: &Self::Tab) -> bool {
        false
    }

    fn is_closeable(&self, _tab: &Self::Tab) -> bool {
        false
    }

}

impl<'a> Chip8TabViewer<'a> {

    async fn upload_file(&mut self) {

        self.is_paused.store(true, std::sync::atomic::Ordering::Relaxed);

        let file_handle = AsyncFileDialog::new()
            .add_filter("chip8", &["ch8", "bin"])
            .set_directory("/")
            .pick_file()
            .await;

        if let Some(file) = file_handle {
            let data = file.read().await;
            let file_name = file.file_name();

            self.chip8.reset();
            self.chip8.load_rom(&data);


        }

        self.is_paused.store(false, std::sync::atomic::Ordering::Relaxed);



    }


}