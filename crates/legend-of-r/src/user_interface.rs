use std::{borrow::Cow, sync::Arc};

use bevy::{
    diagnostic::FrameTimeDiagnosticsPlugin,
    prelude::{default, Deref, DerefMut, IntoExclusiveSystem, Plugin, Res, World},
};
use bevy_egui::EguiContext;
use bevy_framepace::FramePaceDiagnosticsPlugin;
use bevy_inspector_egui::{
    egui::{
        panel::{Side, TopBottomSide},
        Align, CentralPanel, Color32, Context, Frame, Layout, Pos2, Rect, Response, RichText,
        ScrollArea, Ui, Widget as EguiWidget,
    },
    world_inspector::WorldUIContext,
    InspectableRegistry, WorldInspectorParams,
};
use parking_lot::Mutex;

use crate::prelude::{EguiDiagnosticLabel, EguiDiagnosticPlot};

pub struct UserInterfacePlugin;

impl Plugin for UserInterfacePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<WorldInspectorParams>();
        app.init_resource::<InspectableRegistry>();
        app.init_resource::<Widgets>();
        app.init_resource::<AspectRatio>();
        app.add_system(user_interface.exclusive_system());
        app.add_startup_system(setup);
    }
}

pub enum WidgetVariant {
    WorldInspector,
    Owned(Box<dyn Widget>),
}

impl WidgetVariant {
    pub fn update(&mut self, world: &mut World) {
        match self {
            WidgetVariant::WorldInspector => (),
            WidgetVariant::Owned(widget) => widget.update(world),
        }
    }

    pub fn title(&self) -> Cow<'static, str> {
        match self {
            WidgetVariant::WorldInspector => Cow::Borrowed("World Inspector"),
            WidgetVariant::Owned(widget) => widget.title(),
        }
    }

    pub fn ui(
        &mut self,
        world: &mut World,
        ctx: &Context,
        params: &mut WorldInspectorParams,
        ui: &mut Ui,
    ) {
        match self {
            WidgetVariant::WorldInspector => {
                ScrollArea::new([false, true])
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        let mut world_ctx = WorldUIContext::new(world, Some(ctx));
                        world_ctx.world_ui::<()>(ui, params);
                    });
            }
            WidgetVariant::Owned(widget) => {
                widget.ui(ui);
            }
        }
    }
}

pub trait Widget: 'static + Send + Sync {
    fn update(&mut self, world: &mut World);
    fn title(&self) -> Cow<'static, str>;
    fn ui(&mut self, ui: &mut Ui) -> Response;
}

impl<T> Widget for T
where
    T: 'static + Send + Sync + Copy + FnMut(&mut Ui) -> Response,
{
    fn update(&mut self, _: &mut World) {}

    fn title(&self) -> Cow<'static, str> {
        default()
    }

    fn ui(&mut self, ui: &mut Ui) -> Response {
        EguiWidget::ui(*self, ui)
    }
}

#[derive(Clone, Default, Deref, DerefMut)]
pub struct WidgetList {
    widgets: Arc<Mutex<Vec<WidgetVariant>>>,
}

impl WidgetList {
    pub fn add_widget<W>(&self, widget: W) -> &Self
    where
        W: Widget,
    {
        self.widgets
            .lock()
            .push(WidgetVariant::Owned(Box::new(widget)));

        self
    }

    pub fn add_world_inspector(&self) -> &Self {
        self.widgets.lock().push(WidgetVariant::WorldInspector);
        self
    }
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Deref, DerefMut)]
pub struct AspectRatio(pub f32);

impl Default for AspectRatio {
    fn default() -> Self {
        Self(1.0)
    }
}

#[derive(Clone, Default)]
pub struct Widgets {
    pub panel_outer_min: WidgetList,
    pub panel_outer_max: WidgetList,
}

pub fn user_interface(world: &mut World) {
    let widgets = world.resource::<Widgets>().clone();
    let base_aspect = world.resource::<AspectRatio>().clone();

    let world_ptr: *mut _ = world;

    let world: &mut World = unsafe { &mut *world_ptr };
    let mut ctx = world.resource_mut::<EguiContext>();
    let ctx = ctx.ctx_mut();

    let style_inner = (*ctx.style()).clone();

    let mut style_panel = style_inner.clone();
    style_panel.visuals.widgets.noninteractive.bg_fill = Color32::TRANSPARENT;

    let world: &mut World = unsafe { &mut *world_ptr };
    let mut params = world.resource_mut::<WorldInspectorParams>();

    let world: &mut World = unsafe { &mut *world_ptr };

    let rect = ctx.available_rect();
    let aspect = rect.aspect_ratio();
    let (position_min, position_max) = if aspect > *base_aspect {
        (PanelPosition::left(), PanelPosition::right())
    } else if aspect < *base_aspect {
        (PanelPosition::top(), PanelPosition::bottom())
    } else {
        return;
    };

    CentralPanel::default()
        .frame(Frame::default())
        .show(ctx, |ui| {
            UserInterfacePanel::new(position_min, *base_aspect).show(ui, |ui| {
                let mut widgets = widgets.panel_outer_min.lock();
                let last_idx = widgets.len() - 1;

                for (widget, is_last) in widgets
                    .iter_mut()
                    .enumerate()
                    .map(|(i, widget)| (widget, i == last_idx))
                {
                    ui.vertical(|ui| {
                        ui.heading(RichText::new(widget.title()));
                        widget.update(world);
                        widget.ui(world, ctx, &mut params, ui);
                    });

                    if !is_last {
                        ui.separator();
                    }
                }
            });

            UserInterfacePanel::new(position_max, *base_aspect).show(ui, |ui| {
                let mut widgets = widgets.panel_outer_max.lock();
                let last_idx = widgets.len() - 1;

                for (widget, is_last) in widgets
                    .iter_mut()
                    .enumerate()
                    .map(|(i, widget)| (widget, i == last_idx))
                {
                    ui.vertical(|ui| {
                        ui.heading(RichText::new(widget.title()));
                        widget.update(world);
                        widget.ui(world, ctx, &mut params, ui);
                    });

                    if !is_last {
                        ui.separator();
                    }
                }
            });
        });
}

pub fn setup(widgets: Res<Widgets>) {
    widgets.panel_outer_min.add_world_inspector();
    widgets
        .panel_outer_max
        .add_widget(EguiDiagnosticLabel {
            title: "Frame Count",
            diagnostic: FrameTimeDiagnosticsPlugin::FRAME_COUNT,
            ..default()
        })
        .add_widget(EguiDiagnosticPlot {
            title: "Frames Per Second",
            diagnostic_x: FrameTimeDiagnosticsPlugin::FRAME_COUNT,
            diagnostic_y: FrameTimeDiagnosticsPlugin::FPS,
            ..default()
        })
        .add_widget(EguiDiagnosticPlot {
            title: "Frame Time",
            diagnostic_x: FrameTimeDiagnosticsPlugin::FRAME_COUNT,
            diagnostic_y: FrameTimeDiagnosticsPlugin::FRAME_TIME,
            ..default()
        })
        .add_widget(EguiDiagnosticPlot {
            title: "Framepace Oversleep",
            diagnostic_x: FrameTimeDiagnosticsPlugin::FRAME_COUNT,
            diagnostic_y: FramePaceDiagnosticsPlugin::FRAMEPACE_OVERSLEEP,
            ..default()
        })
        .add_widget(EguiDiagnosticPlot {
            title: "Framepace Error",
            diagnostic_x: FrameTimeDiagnosticsPlugin::FRAME_COUNT,
            diagnostic_y: FramePaceDiagnosticsPlugin::FRAMEPACE_ERROR,
            ..default()
        });
}

#[derive(Debug, Copy, Clone)]
enum PanelPosition {
    Horizontal(Side),
    Vertical(TopBottomSide),
}

impl PanelPosition {
    pub fn top() -> Self {
        PanelPosition::Vertical(TopBottomSide::Top)
    }

    pub fn bottom() -> Self {
        PanelPosition::Vertical(TopBottomSide::Bottom)
    }

    pub fn left() -> Self {
        PanelPosition::Horizontal(Side::Left)
    }

    pub fn right() -> Self {
        PanelPosition::Horizontal(Side::Right)
    }
}

#[derive(Debug, Copy, Clone)]
struct UserInterfacePanel {
    position: PanelPosition,
    aspect: f32,
}

impl UserInterfacePanel {
    pub fn new(position: PanelPosition, aspect: f32) -> Self {
        UserInterfacePanel { position, aspect }
    }

    pub fn show(self, ui: &mut Ui, add_contents: impl FnOnce(&mut Ui)) -> Response {
        let available = ui.max_rect();

        let available_width = available.width();
        let available_height = available.height();

        let style_inner = (*ui.style()).clone();

        let mut style_panel = (*style_inner).clone();
        style_panel.visuals.widgets.noninteractive.bg_fill = Color32::TRANSPARENT;

        ui.set_style(style_panel.clone());
        match self.position {
            PanelPosition::Horizontal(side) => {
                let fac = available_width / (available_height * self.aspect);

                let half_width = available_width * 0.5;
                let width = half_width - half_width / fac;

                let rect = Rect {
                    min: match side {
                        Side::Left => Pos2::new(0.0, 0.0),
                        Side::Right => Pos2::new(available_width - width, 0.0),
                    },
                    max: match side {
                        Side::Left => Pos2::new(width, available_height),
                        Side::Right => Pos2::new(available_width, available_height),
                    },
                };

                ui.set_clip_rect(rect);

                ui.allocate_ui_at_rect(rect, |ui| {
                    ui.set_style(style_inner.clone());
                    ui.with_layout(Layout::top_down(Align::Min), add_contents)
                })
                .response
            }
            PanelPosition::Vertical(side) => {
                let fac = available_height / (available_width / self.aspect);

                let half_height = available_height * 0.5;
                let height = half_height - half_height / fac;

                let rect = Rect {
                    min: match side {
                        TopBottomSide::Top => Pos2::new(0.0, 0.0),
                        TopBottomSide::Bottom => Pos2::new(0.0, available_height - height),
                    },
                    max: match side {
                        TopBottomSide::Top => Pos2::new(available_width, height),
                        TopBottomSide::Bottom => Pos2::new(available_width, available_height),
                    },
                };

                ui.set_clip_rect(rect);

                ui.allocate_ui_at_rect(rect, |ui| {
                    ui.set_style(style_inner.clone());
                    ui.with_layout(Layout::left_to_right(Align::Min), add_contents)
                })
                .response
            }
        }
    }
}
