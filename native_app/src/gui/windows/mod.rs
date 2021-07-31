use imgui::{StyleVar, Ui};
use serde::{Deserialize, Serialize};

use crate::uiworld::UiWorld;
use egregoria::Egregoria;

mod config;
pub mod debug;
mod economy;
mod map;
pub mod network;
pub mod settings;

pub trait ImguiWindow: Send + Sync {
    fn render_window(
        &mut self,
        window: imgui::Window<'_>,
        ui: &Ui<'_>,
        uiworld: &mut UiWorld,
        goria: &Egregoria,
    );
}

impl<F> ImguiWindow for F
where
    F: Fn(imgui::Window<'_>, &Ui<'_>, &mut UiWorld, &Egregoria) + Send + Sync,
{
    fn render_window(
        &mut self,
        window: imgui::Window<'_>,
        ui: &Ui<'_>,
        uiworld: &mut UiWorld,
        goria: &Egregoria,
    ) {
        self(window, ui, uiworld, goria);
    }
}

struct ImguiWindowStruct {
    w: Box<dyn ImguiWindow>,
    name: &'static imgui::ImStr,
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct ImguiWindows {
    #[serde(skip)]
    windows: Vec<ImguiWindowStruct>,
    opened: Vec<bool>,
}

impl Default for ImguiWindows {
    fn default() -> Self {
        let mut s = Self {
            windows: vec![],
            opened: vec![],
        };
        s.insert(imgui::im_str!("Map"), map::map, true);
        s.insert(imgui::im_str!("Economy"), economy::economy, false);
        s.insert(imgui::im_str!("Config"), config::config, false);
        s.insert(imgui::im_str!("Debug"), debug::debug, false);
        s.insert(imgui::im_str!("Settings"), settings::settings, false);
        s.insert(imgui::im_str!("Network"), network::network, false);
        s
    }
}

impl ImguiWindows {
    pub fn insert(
        &mut self,
        name: &'static imgui::ImStr,
        w: impl ImguiWindow + 'static,
        opened: bool,
    ) {
        self.windows.push(ImguiWindowStruct {
            w: Box::new(w),
            name,
        });
        if self.opened.len() < self.windows.len() {
            self.opened.push(opened)
        }
    }

    pub fn menu(&mut self, ui: &Ui<'_>) {
        if self.opened.len() < self.windows.len() {
            self.opened
                .extend(std::iter::repeat(false).take(self.windows.len() - self.opened.len()))
        }
        let h = ui.window_size()[1];
        for (opened, w) in self.opened.iter_mut().zip(self.windows.iter()) {
            let tok = ui.push_style_var(StyleVar::Alpha(if *opened { 1.0 } else { 0.5 }));
            *opened ^= ui.button(w.name, [80.0, h]);
            tok.pop(ui);
        }
    }

    pub fn render(&mut self, ui: &Ui<'_>, uiworld: &mut UiWorld, goria: &Egregoria) {
        for (ws, opened) in self.windows.iter_mut().zip(self.opened.iter_mut()) {
            if *opened {
                ws.w.render_window(
                    imgui::Window::new(ws.name).opened(opened),
                    ui,
                    uiworld,
                    goria,
                );
            }
        }
    }
}
