use std::{borrow::Cow, collections::VecDeque};

use bevy::{
    diagnostic::{DiagnosticId, Diagnostics},
    prelude::{default, World},
};
use bevy_inspector_egui::{
    bevy_egui::egui::plot::{Line, PlotPoints},
    egui::plot::Plot,
};

use crate::user_interface::Widget;

pub struct EguiDiagnosticLabel {
    pub title: &'static str,
    pub diagnostic: DiagnosticId,
    pub value: f64,
}

impl Default for EguiDiagnosticLabel {
    fn default() -> Self {
        Self {
            title: "Diagnostic Label",
            diagnostic: default(),
            value: default(),
        }
    }
}

impl EguiDiagnosticLabel {
    pub fn update(&mut self, diagnostics: &Diagnostics) {
        self.value = diagnostics
            .get(self.diagnostic)
            .map(|diagnostic| diagnostic.value())
            .flatten()
            .unwrap_or_default();
    }
}

impl Widget for EguiDiagnosticLabel {
    fn update(&mut self, world: &mut World) {
        self.update(world.resource::<Diagnostics>())
    }

    fn title(&self) -> Cow<'static, str> {
        Cow::Borrowed(self.title)
    }

    fn ui(
        &mut self,
        ui: &mut bevy_inspector_egui::egui::Ui,
    ) -> bevy_inspector_egui::egui::Response {
        ui.vertical(|ui| {
            ui.label(format!("{}", self.value));
        })
        .response
    }
}

pub struct EguiDiagnosticPlot {
    pub buf: VecDeque<(f64, f64)>,
    pub title: &'static str,
    pub diagnostic_x: DiagnosticId,
    pub diagnostic_y: DiagnosticId,
}

impl Default for EguiDiagnosticPlot {
    fn default() -> Self {
        Self {
            buf: VecDeque::with_capacity(144),
            title: "Diagnostic Plot",
            diagnostic_x: default(),
            diagnostic_y: default(),
        }
    }
}

impl EguiDiagnosticPlot {
    pub fn update(&mut self, diagnostics: &Diagnostics) {
        if let (Some(x), Some(y)) = (
            diagnostics.get(self.diagnostic_x),
            diagnostics.get(self.diagnostic_y),
        ) {
            if let (Some(x), Some(y)) = (x.value(), y.value()) {
                if self.buf.len() == self.buf.capacity() {
                    self.buf.pop_front();
                }

                self.buf.push_back((x, y));
            }
        }
    }
}

impl Widget for EguiDiagnosticPlot {
    fn update(&mut self, world: &mut World) {
        self.update(world.resource::<Diagnostics>())
    }

    fn title(&self) -> Cow<'static, str> {
        Cow::Borrowed(&self.title)
    }

    fn ui(
        &mut self,
        ui: &mut bevy_inspector_egui::egui::Ui,
    ) -> bevy_inspector_egui::egui::Response {
        ui.vertical(|ui| {
            ui.label(format!(
                "{}",
                self.buf.back().copied().unwrap_or_default().1
            ));

            Plot::new(self.title)
                .width(ui.available_width().min(256.0))
                .height(64.0)
                .show(ui, |plot_ui| {
                    plot_ui.line(Line::new(PlotPoints::new(
                        self.buf.iter().map(|(x, y)| [*x, *y]).collect::<Vec<_>>(),
                    )))
                });
        })
        .response
    }
}
