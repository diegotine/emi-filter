use crate::engine::{self, Parameters, Simulation};
use crate::history::{GraphRecord, History, Snapshot};
use crate::{parameter_editor, storage};
use eframe::egui::{self, Align2, Color32, FontId, Pos2, Rect, RichText, Sense, Stroke, Vec2};
use std::sync::Arc;

const BLUE: Color32 = Color32::from_rgb(42, 120, 186);
const CYAN: Color32 = Color32::from_rgb(222, 238, 249);
const ORANGE: Color32 = Color32::from_rgb(204, 91, 38);
const GREEN: Color32 = Color32::from_rgb(22, 137, 97);
const INK: Color32 = Color32::from_rgb(32, 57, 75);
const MUTED: Color32 = Color32::from_rgb(90, 115, 130);
const BORDER: Color32 = Color32::from_rgb(198, 215, 226);

type PlotSeries<'a> = (&'a str, Color32, Vec<[f64; 2]>);

/// Título e legenda sobre a grade, sem reservar altura fora das curvas.
fn plot_header(
    painter: &egui::Painter,
    rect: Rect,
    title: &str,
    timestamp: Option<&str>,
    legend: &[(&str, Color32, bool)],
) {
    let title_rect = painter.text(
        rect.min + Vec2::new(12.0, 15.0),
        Align2::LEFT_CENTER,
        title,
        FontId::proportional(17.0),
        INK,
    );
    if let Some(timestamp) = timestamp {
        painter.text(
            Pos2::new(title_rect.right() + 12.0, title_rect.center().y),
            Align2::LEFT_CENTER,
            timestamp,
            FontId::proportional(11.0),
            MUTED,
        );
    }
    let left = rect.left() + 12.0;
    let mut x = left;
    let mut y = rect.top() + 39.0;
    for &(label, color, dashed) in legend {
        let text = painter.layout_no_wrap(label.into(), FontId::proportional(12.0), color);
        let width = 28.0 + text.size().x + 18.0;
        if x > left && x + width > rect.right() - 12.0 {
            x = left;
            y += 19.0;
        }
        if label.contains("harmônicas") {
            painter.circle_filled(Pos2::new(x + 9.0, y), 3.5, color);
        } else if dashed {
            for start in [0.0, 8.0, 16.0] {
                painter.line_segment(
                    [Pos2::new(x + start, y), Pos2::new(x + start + 4.0, y)],
                    Stroke::new(1.8_f32, color),
                );
            }
        } else {
            painter.line_segment(
                [Pos2::new(x, y), Pos2::new(x + 20.0, y)],
                Stroke::new(1.8_f32, color),
            );
        }
        let text_height = text.size().y;
        painter.galley(Pos2::new(x + 28.0, y - text_height / 2.0), text, color);
        x += width;
    }
}

/// Escalas comuns às séries; x' é log10(x) somente para curvas espectrais.
fn scientific_plot(
    ui: &mut egui::Ui,
    heading: (&str, Option<&str>),
    xunit: &str,
    yunit: &str,
    series: &[PlotSeries<'_>],
    log_x: bool,
    y_bounds: Option<[f64; 2]>,
) {
    let (title, timestamp) = heading;
    let height = (ui.ctx().screen_rect().bottom() - ui.cursor().top() - 55.0).max(180.0);
    let (rect, response) =
        ui.allocate_exact_size(Vec2::new(ui.available_width(), height), Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 0.0, Color32::WHITE);
    let legend = series
        .iter()
        .map(|(label, color, _)| (*label, *color, label.contains("CM")))
        .collect::<Vec<_>>();
    let chart = Rect::from_min_max(
        rect.min + Vec2::new(85.0, 20.0),
        rect.max - Vec2::new(25.0, 38.0),
    );
    let transform = |x: f64| if log_x { x.log10() } else { x };
    let mut xmin = f64::INFINITY;
    let mut xmax = f64::NEG_INFINITY;
    let mut ymin = f64::INFINITY;
    let mut ymax = f64::NEG_INFINITY;
    for (_, _, points) in series {
        for &[x, y] in points {
            let x = transform(x);
            if x.is_finite() && y.is_finite() {
                xmin = xmin.min(x);
                xmax = xmax.max(x);
                ymin = ymin.min(y);
                ymax = ymax.max(y);
            }
        }
    }
    if !xmin.is_finite() {
        plot_header(
            &painter.with_clip_rect(chart),
            chart,
            title,
            timestamp,
            &legend,
        );
        return;
    }
    let dy = (ymax - ymin).max(1e-12) * 0.08;
    ymin -= dy;
    ymax += dy;
    if let Some([lo, hi]) = y_bounds {
        ymin = lo;
        ymax = hi.max(lo + 1.0);
    }
    let dx = (xmax - xmin).max(1e-12);
    let pos = |x: f64, y: f64| {
        Pos2::new(
            chart.left() + ((transform(x) - xmin) / dx) as f32 * chart.width(),
            chart.bottom() - ((y - ymin) / (ymax - ymin)) as f32 * chart.height(),
        )
    };
    for i in 0..=5 {
        let ratio = i as f64 / 5.0;
        let x = xmin + ratio * dx;
        let px = chart.left() + ratio as f32 * chart.width();
        painter.line_segment(
            [Pos2::new(px, chart.top()), Pos2::new(px, chart.bottom())],
            Stroke::new(0.5f32, BORDER),
        );
        painter.text(
            Pos2::new(px, chart.bottom() + 14.0),
            Align2::CENTER_CENTER,
            if log_x {
                frequency_label(10f64.powf(x))
            } else {
                format!("{x:.2}")
            },
            FontId::monospace(10.0),
            MUTED,
        );
    }
    for i in 0..=5 {
        let ratio = i as f64 / 5.0;
        let y = ymin + ratio * (ymax - ymin);
        let py = chart.bottom() - ratio as f32 * chart.height();
        painter.line_segment(
            [Pos2::new(chart.left(), py), Pos2::new(chart.right(), py)],
            Stroke::new(if y == 0.0 { 0.8_f32 } else { 0.5_f32 }, BORDER),
        );
        painter.text(
            Pos2::new(chart.left() - 6.0, py),
            Align2::RIGHT_CENTER,
            if y.abs() >= 10000.0 || (y != 0.0 && y.abs() < 0.001) {
                format!("{y:.2e}")
            } else {
                format!("{y:.3}")
            },
            FontId::monospace(10.0),
            MUTED,
        );
    }
    painter.text(
        Pos2::new(rect.left() + 8.0, chart.top()),
        Align2::LEFT_TOP,
        yunit,
        FontId::monospace(11.0),
        MUTED,
    );
    painter.text(
        rect.right_bottom(),
        Align2::RIGHT_BOTTOM,
        xunit,
        FontId::monospace(11.0),
        MUTED,
    );
    let clip = painter.with_clip_rect(chart);
    for (label, color, points) in series {
        let points = points
            .iter()
            .filter(|p| transform(p[0]).is_finite() && p[1].is_finite())
            .map(|p| pos(p[0], p[1]))
            .collect::<Vec<_>>();
        if label.contains("CM") {
            for segment in points.windows(2) {
                if ((segment[0].x - chart.left()) / 6.0) as i32 % 2 == 0 {
                    clip.line_segment([segment[0], segment[1]], Stroke::new(1.5f32, *color));
                }
            }
        } else {
            clip.add(egui::Shape::line(points, Stroke::new(1.5f32, *color)));
        }
    }
    if let Some(pointer) = response.hover_pos().filter(|p| chart.contains(*p)) {
        let x = xmin + ((pointer.x - chart.left()) / chart.width()) as f64 * dx;
        let x = if log_x { 10f64.powf(x) } else { x };
        response.on_hover_ui_at_pointer(|ui| {
            ui.monospace(format!("{x:.5} {xunit}"));
            for (label, color, points) in series {
                if let Some(p) = points
                    .iter()
                    .min_by(|a, b| (a[0] - x).abs().total_cmp(&(b[0] - x).abs()))
                {
                    ui.colored_label(*color, format!("{label}: {:.6e} {yunit}", p[1]));
                }
            }
        });
    }
    plot_header(
        &painter.with_clip_rect(chart),
        chart,
        title,
        timestamp,
        &legend,
    );
}

pub struct EmiApp {
    parameters: Parameters,
    calculated: Parameters,
    simulation: Option<Arc<Simulation>>,
    history: History,
    editor: parameter_editor::Editor,
    error: Option<String>,
    status: String,
    duration_ms: f64,
    plot_tab: usize,
    modal_total: bool,
    wave_mode: usize,
    derivative_order: usize,
    show_dm: bool,
    show_cm: bool,
    show_harmonics: bool,
    show_reference: bool,
    original_texture: Option<egui::TextureHandle>,
    json_path: String,
    csv_path: String,
    plot_min_db: f64,
    plot_max_db: f64,
    automatic_db: bool,
    screenshot_frames: usize,
}

impl EmiApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut style = (*cc.egui_ctx.style()).clone();
        style.visuals = egui::Visuals::light();
        style.visuals.panel_fill = Color32::from_rgb(243, 247, 250);
        style.visuals.window_fill = Color32::WHITE;
        style.visuals.extreme_bg_color = Color32::WHITE;
        style.visuals.faint_bg_color = Color32::from_rgb(235, 243, 249);
        style.visuals.selection.bg_fill = BLUE;
        style.visuals.selection.stroke = Stroke::new(1.0f32, Color32::WHITE);
        style.visuals.widgets.inactive.bg_fill = Color32::from_rgb(232, 242, 249);
        style.visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(236, 244, 249);
        style.visuals.widgets.inactive.bg_stroke = Stroke::new(1.0f32, BORDER);
        style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(208, 230, 246);
        style.visuals.widgets.active.bg_fill = Color32::from_rgb(179, 214, 239);
        style.visuals.override_text_color = Some(INK);
        style.spacing.item_spacing = Vec2::new(7.0, 5.0);
        style.spacing.button_padding = Vec2::new(9.0, 5.0);
        style.spacing.interact_size = Vec2::new(38.0, 23.0);
        style
            .text_styles
            .insert(egui::TextStyle::Body, FontId::proportional(13.0));
        style
            .text_styles
            .insert(egui::TextStyle::Button, FontId::proportional(13.0));
        style
            .text_styles
            .insert(egui::TextStyle::Small, FontId::proportional(11.0));
        cc.egui_ctx.set_style(style);

        let defaults_path = storage::defaults_path();
        let (parameters, startup) = if defaults_path.exists() {
            match std::fs::read_to_string(&defaults_path)
                .map_err(|e| e.to_string())
                .and_then(|s| storage::read_document(&s))
                .and_then(|d| {
                    d.parameters.validate()?;
                    Ok(d.parameters)
                }) {
                Ok(p) => (p, format!("Padrões pessoais: {}", defaults_path.display())),
                Err(e) => (
                    Parameters::default(),
                    format!("Padrões pessoais não carregados: {e}"),
                ),
            }
        } else {
            (Parameters::default(), "Modelo original carregado".into())
        };
        let started = std::time::Instant::now();
        let result = engine::calculate(&parameters);
        let (simulation, error) = match result {
            Ok(result) => (Some(Arc::new(result)), None),
            Err(error) => (None, Some(error)),
        };
        let mut history = History::default();
        if let Some(s) = &simulation {
            history.record(Snapshot {
                record: GraphRecord {
                    parameters: parameters.clone(),
                    timestamp: storage::timestamp(),
                },
                simulation: s.clone(),
            });
        }
        let mut editor = parameter_editor::Editor::new();
        if let Ok(group) = std::env::var("EMI_SCREENSHOT_COMPONENT") {
            editor.select(&group);
        }
        if std::env::var_os("EMI_SCREENSHOT_COMPARE").is_some() {
            for factor in [1.4, 2.0] {
                let mut p = parameters.clone();
                p.cx2_uf *= factor;
                if let Ok(s) = engine::calculate(&p) {
                    history.record(Snapshot {
                        record: GraphRecord {
                            parameters: p,
                            timestamp: storage::timestamp(),
                        },
                        simulation: Arc::new(s),
                    });
                }
            }
            history.overlay = true;
            history.selected = 0;
        }
        Self {
            history,
            editor,
            calculated: parameters.clone(),
            parameters,
            simulation,
            error,
            status: startup,
            duration_ms: started.elapsed().as_secs_f64() * 1000.0,
            plot_tab: std::env::var("EMI_SCREENSHOT_TAB")
                .ok()
                .and_then(|v| v.parse::<usize>().ok())
                .filter(|v| *v < 3)
                .unwrap_or(0),
            modal_total: true,
            wave_mode: 0,
            derivative_order: 0,
            show_dm: true,
            show_cm: true,
            show_harmonics: true,
            show_reference: false,
            original_texture: None,
            json_path: "emi-filter-config.json".into(),
            csv_path: "emi-filter-atenuacao.csv".into(),
            plot_min_db: -15.0,
            plot_max_db: 80.0,
            automatic_db: true,
            screenshot_frames: 0,
        }
    }

    fn calculate(&mut self) {
        let started = std::time::Instant::now();
        match engine::calculate(&self.parameters) {
            Ok(simulation) => {
                let simulation = Arc::new(simulation);
                self.history.record(Snapshot {
                    record: GraphRecord {
                        parameters: self.parameters.clone(),
                        timestamp: storage::timestamp(),
                    },
                    simulation: simulation.clone(),
                });
                self.simulation = Some(simulation);
                self.history.edit_target = (self.history.selected + 1 < self.history.graphs.len())
                    .then_some(self.history.selected);
                self.calculated = self.parameters.clone();
                self.error = None;
                self.duration_ms = started.elapsed().as_secs_f64() * 1000.0;
                self.status = format!(
                    "Gráfico {} atualizado · {:.1} ms",
                    self.history.selected + 1,
                    self.duration_ms
                );
            }
            Err(error) => {
                self.error = Some(error);
                self.status = "Confira os valores indicados".into();
            }
        }
    }

    fn toolbar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.add_space(4.0);
            ui.vertical(|ui| {
                ui.label(RichText::new("EMI FILTER").size(22.0).strong().color(BLUE));
                ui.label(
                    RichText::new(format!("v{} -- Made with Codex", env!("CARGO_PKG_VERSION")))
                        .small()
                        .color(MUTED),
                );
            });
            ui.add_space(20.0);
            toolbar_measure(
                ui,
                "Rede",
                &format!(
                    "{} / {:.1} V",
                    compact(self.parameters.mains_phase_v_rms),
                    self.parameters.line_voltage_rms()
                ),
                &format!(
                    "{} Hz · {}°",
                    compact(self.parameters.mains_frequency_hz),
                    compact(self.parameters.mains_phase_deg)
                ),
            );
            toolbar_measure(
                ui,
                "Sentido do ruído",
                "SERVOS -> REDE",
                "Perda de inserção",
            );
            toolbar_measure(
                ui,
                "Excitação",
                if self.parameters.extra.ringing_enabled {
                    "2 × ringing"
                } else {
                    "2 × quadrada"
                },
                &format!(
                    "{} kHz · {}°",
                    compact(
                        self.parameters
                            .extra
                            .fundamental(self.parameters.noise_frequency_hz)
                            / 1000.0
                    ),
                    compact(self.parameters.noise_delay_deg)
                ),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .add_sized(
                        [165.0, 39.0],
                        egui::Button::new(
                            RichText::new("Calcular atenuação")
                                .strong()
                                .color(Color32::WHITE),
                        )
                        .fill(BLUE),
                    )
                    .clicked()
                {
                    self.calculate();
                }
                if ui.button("Carregar padrões").clicked() {
                    let path = storage::defaults_path();
                    if path.exists() {
                        match std::fs::read_to_string(path)
                            .map_err(|e| e.to_string())
                            .and_then(|s| storage::read_document(&s))
                        {
                            Ok(doc) => {
                                self.parameters = doc.parameters;
                                self.status =
                                    "Padrões pessoais carregados; calcule para comparar".into();
                                self.error = None;
                            }
                            Err(e) => self.error = Some(e),
                        }
                    } else {
                        self.parameters = Parameters::default();
                        self.status = "Padrões originais carregados; calcule para comparar".into();
                    }
                }
                if ui
                    .button("Salvar padrões")
                    .on_hover_text(storage::defaults_path().display().to_string())
                    .clicked()
                {
                    match self.parameters.validate().and_then(|()| {
                        storage::save(
                            &storage::defaults_path(),
                            &storage::Document::new(self.parameters.clone()),
                        )
                    }) {
                        Ok(()) => {
                            self.status =
                                format!("Padrões salvos: {}", storage::defaults_path().display());
                            self.error = None;
                        }
                        Err(e) => self.error = Some(e),
                    }
                }
            });
        });
    }

    fn sidebar(&mut self, ui: &mut egui::Ui) {
        section_bar(ui, "VISUALIZAÇÃO");
        channel_toggle(
            ui,
            &mut self.show_dm,
            BLUE,
            "A",
            "Modo diferencial",
            "DM · entre fases",
        );
        channel_toggle(
            ui,
            &mut self.show_cm,
            ORANGE,
            "B",
            "Modo comum",
            "CM · relativo ao PE",
        );
        channel_toggle(
            ui,
            &mut self.show_harmonics,
            GREEN,
            "H",
            if self.parameters.extra.ringing_enabled {
                "Pacotes de ringing"
            } else {
                "Duas quadradas"
            },
            "Harmônicas presentes",
        );
        ui.add_space(6.0);
        ui.checkbox(&mut self.automatic_db, "Escala dB automática");
        ui.checkbox(&mut self.modal_total, "Incluir conversão DM↔CM");
        ui.label(
            RichText::new(
                "Com conversão: mede a norma das duas fases para cada excitação modal pura.",
            )
            .small()
            .color(MUTED),
        );
        if !self.automatic_db {
            numeric_row(ui, "Mínimo", &mut self.plot_min_db, "dB", 0.5);
            numeric_row(ui, "Máximo", &mut self.plot_max_db, "dB", 0.5);
        }
        ui.label(
            RichText::new("Passe o cursor pelo gráfico para ler frequência e atenuação.")
                .small()
                .color(MUTED),
        );
        ui.add_space(8.0);
        section_bar(ui, "ATENUAÇÃO RMS CONJUNTA");
        if let Some(result) = &self.simulation {
            for row in &result.integrated {
                let label = match row.order {
                    0 => "Onda x(t)",
                    1 => "Derivada dx/dt",
                    _ => "Derivada d²x/dt²",
                };
                egui::Frame::NONE
                    .fill(Color32::WHITE)
                    .stroke(Stroke::new(1.0f32, BORDER))
                    .inner_margin(8.0)
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.label(RichText::new(label).color(MUTED));
                        ui.label(
                            RichText::new(format!("{:.2} dB", row.combined_db))
                                .font(FontId::monospace(24.0))
                                .color(if row.combined_db < 0.0 { ORANGE } else { BLUE }),
                        );
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(format!("DM {}", finite_db(row.dm_db)))
                                    .small()
                                    .color(BLUE),
                            );
                            ui.label(
                                RichText::new(format!("CM {}", finite_db(row.cm_db)))
                                    .small()
                                    .color(ORANGE),
                            );
                        });
                    });
            }
        }
        ui.add_space(7.0);
        ui.label(
            RichText::new("Referência: a mesma rede e as mesmas fontes, sem filtro.")
                .small()
                .color(MUTED),
        );
        ui.label(
            RichText::new(
                "Valor negativo = reforço de tensão. As derivadas alteram os pesos espectrais.",
            )
            .small()
            .color(MUTED),
        );
        ui.add_space(8.0);
        section_bar(ui, "ARQUIVOS");
        ui.label("Configuração JSON");
        ui.add(egui::TextEdit::singleline(&mut self.json_path).desired_width(f32::INFINITY));
        ui.horizontal(|ui| {
            if ui.button("Salvar").clicked() {
                let mut document = storage::Document::new(self.parameters.clone());
                document.graphs = self.history.records();
                document.selected_graph = self.history.selected;
                document.overlay = self.history.overlay;
                match storage::save(std::path::Path::new(&self.json_path), &document) {
                    Ok(()) => {
                        self.status = format!("Salvo: {}", self.json_path);
                        self.error = None;
                    }
                    Err(error) => self.error = Some(format!("Não foi possível salvar: {error}")),
                }
            }
            if ui.button("Abrir").clicked() {
                match std::fs::read_to_string(&self.json_path)
                    .map_err(|e| e.to_string())
                    .and_then(|s| storage::read_document(&s))
                {
                    Ok(doc) => {
                        let restored = (|| -> Result<History, String> {
                            let mut h = History::default();
                            for record in doc.graphs {
                                let simulation = Arc::new(engine::calculate(&record.parameters)?);
                                h.record(Snapshot { record, simulation });
                            }
                            h.selected = doc.selected_graph;
                            h.overlay = doc.overlay;
                            Ok(h)
                        })();
                        match restored {
                            Ok(history) => {
                                self.history = history;
                                self.parameters = doc.parameters;
                                if let Some(snapshot) = self.history.select(self.history.selected) {
                                    self.simulation = Some(snapshot.simulation);
                                    self.calculated = snapshot.record.parameters;
                                } else {
                                    self.calculate();
                                }
                                self.error = None;
                                self.status = "Sessão carregada com parâmetros e histórico".into();
                            }
                            Err(e) => self.error = Some(format!("Histórico incompatível: {e}")),
                        }
                    }
                    Err(error) => self.error = Some(format!("Não foi possível abrir: {error}")),
                }
            }
        });
        if ui.small_button("Padrões de fábrica").clicked() {
            self.parameters = Parameters::default();
            self.status = "Valores originais carregados; calcule para comparar".into();
            self.error = None;
        }
        ui.label("Dados do gráfico CSV");
        ui.add(egui::TextEdit::singleline(&mut self.csv_path).desired_width(f32::INFINITY));
        if ui.button("Exportar CSV").clicked() {
            if let Some(simulation) = &self.simulation {
                let csv = engine::export_csv(simulation);
                match std::fs::write(&self.csv_path, csv) {
                    Ok(()) => {
                        self.status = format!("Exportado: {}", self.csv_path);
                        self.error = None;
                    }
                    Err(error) => self.error = Some(format!("Não foi possível exportar: {error}")),
                }
            }
        }
        for (label, suffix, temporal) in [
            ("Exportar H complexa", "transferencia", false),
            ("Exportar ondas", "ondas", true),
        ] {
            if ui.button(label).clicked() {
                if let Some(simulation) = &self.simulation {
                    let path = std::path::Path::new(&self.csv_path)
                        .with_extension(format!("{suffix}.csv"));
                    let data = if temporal {
                        engine::export_waveform_csv(simulation)
                    } else {
                        Ok(engine::export_transfer_csv(simulation))
                    };
                    match data.and_then(|csv| std::fs::write(&path, csv).map_err(|e| e.to_string()))
                    {
                        Ok(()) => {
                            self.status = format!("Exportado: {}", path.display());
                            self.error = None;
                        }
                        Err(error) => {
                            self.error = Some(format!("Não foi possível exportar: {error}"))
                        }
                    }
                }
            }
        }
        ui.add_space(10.0);
        ui.label(
            RichText::new(
                "Modelo linear · perdas extrapoladas\nSem H2/H3 geradas, saturação ou aquecimento",
            )
            .small()
            .color(MUTED),
        );
    }

    fn diagram(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Circuito original").size(17.0).strong())
                .on_hover_text("Edite valores; clique no nome para abrir parâmetros.\nCx: classe X2. Cy: classe Y.");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.checkbox(&mut self.show_reference, "Ver esquema final PDF");
            });
        });
        if self.show_reference {
            if self.original_texture.is_none() {
                if let Ok(image) =
                    image::load_from_memory(include_bytes!("../assets/esquema_filtro_final.png"))
                {
                    let rgba = image.into_rgba8();
                    let size = [rgba.width() as usize, rgba.height() as usize];
                    let color = egui::ColorImage::from_rgba_unmultiplied(size, rgba.as_raw());
                    self.original_texture = Some(ui.ctx().load_texture(
                        "esquema-final",
                        color,
                        egui::TextureOptions::LINEAR,
                    ));
                }
            }
            if let Some(texture) = &self.original_texture {
                ui.add(egui::Image::new(texture).fit_to_exact_size(Vec2::new(
                    ui.available_width(),
                    ui.available_width() * 2000.0 / 3600.0,
                )));
            }
            return;
        }

        egui::Frame::NONE
            .fill(Color32::WHITE)
            .stroke(Stroke::new(1.0f32, BORDER))
            .show(ui, |ui| {
                let width = ui.available_width();
                let scale = (width / 1160.0).min(1.0);
                let yscale = (ui.ctx().screen_rect().height() * 0.25 / 314.0)
                    .clamp(0.58, 0.84)
                    .min(scale);
                let (rect, _) =
                    ui.allocate_exact_size(Vec2::new(width, 314.0 * yscale), Sense::hover());
                let painter = ui.painter_at(rect);
                let at =
                    |x: f32, y: f32| Pos2::new(rect.left() + x * scale, rect.top() + y * yscale);
                let line = |x1: f32, y1: f32, x2: f32, y2: f32| {
                    painter.line_segment([at(x1, y1), at(x2, y2)], Stroke::new(1.6f32, INK));
                };
                let dot = |x: f32, y: f32| {
                    painter.circle_filled(at(x, y), 2.4 * scale, INK);
                };
                let text = |x: f32, y: f32, s: &str, align: Align2, size: f32, color: Color32| {
                    painter.text(
                        at(x, y),
                        align,
                        s,
                        FontId::proportional(size * scale),
                        color,
                    );
                };
                let cap = |x: f32, y: f32, top: f32, bottom: f32| {
                    line(x, top, x, y - 4.0);
                    line(x, y + 4.0, x, bottom);
                    line(x - 11.0, y - 4.0, x + 11.0, y - 4.0);
                    line(x - 11.0, y + 4.0, x + 11.0, y + 4.0);
                };
                let resistor = |x: f32, y: f32| {
                    painter.rect_filled(
                        Rect::from_center_size(at(x, y), Vec2::new(10.0 * scale, 23.0 * scale)),
                        0.0,
                        Color32::WHITE,
                    );
                    for (x1, y1, x2, y2) in [
                        (x - 5.0, y - 11.5, x + 5.0, y - 11.5),
                        (x + 5.0, y - 11.5, x + 5.0, y + 11.5),
                        (x + 5.0, y + 11.5, x - 5.0, y + 11.5),
                        (x - 5.0, y + 11.5, x - 5.0, y - 11.5),
                    ] {
                        line(x1, y1, x2, y2);
                    }
                };
                let coil = |x0: f32, x1: f32, y: f32, sign: f32| {
                    let mut points = Vec::new();
                    for turn in 0..4 {
                        for step in 0..13 {
                            let t = step as f32 / 12.0;
                            let x = x0 + (x1 - x0) * (turn as f32 + t) / 4.0;
                            points.push(at(x, y + sign * (t * std::f32::consts::PI).sin() * 10.0));
                        }
                    }
                    painter.add(egui::Shape::line(points, Stroke::new(1.8f32, INK)));
                };
                let pe = |x: f32, y: f32| {
                    line(x, y - 9.0, x, y);
                    for (dy, half) in [(0.0, 10.0), (4.0, 6.5), (8.0, 3.0)] {
                        line(x - half, y + dy, x + half, y + dy);
                    }
                };
                text(24.0, 19.0, "REDE", Align2::LEFT_CENTER, 12.0, BLUE);
                text(
                    1137.0,
                    19.0,
                    &format!("{} SERVOS", self.parameters.extra.servo_count),
                    Align2::RIGHT_CENTER,
                    12.0,
                    BLUE,
                );
                text(
                    580.0,
                    19.0,
                    "<- SENTIDO DO RUÍDO",
                    Align2::CENTER_CENTER,
                    11.0,
                    MUTED,
                );
                for y in [76.0, 220.0] {
                    for (a, b) in [(36.0, 354.0), (446.0, 834.0), (898.0, 1126.0)] {
                        line(a, y, b, y);
                    }
                    coil(354.0, 446.0, y, if y < 100.0 { 1.0 } else { -1.0 });
                    coil(834.0, 898.0, y, -1.0);
                    painter.circle_stroke(at(36.0, y), 3.0 * scale, Stroke::new(1.5f32, INK));
                    painter.circle_stroke(at(1126.0, y), 3.0 * scale, Stroke::new(1.5f32, INK));
                    for x in [85.0, 240.0, 555.0, 730.0, 1015.0] {
                        dot(x, y);
                    }
                }
                text(18.0, 76.0, "F1", Align2::CENTER_CENTER, 11.0, INK);
                text(18.0, 220.0, "F2", Align2::CENTER_CENTER, 11.0, INK);
                text(1142.0, 76.0, "F1", Align2::CENTER_CENTER, 11.0, INK);
                text(1142.0, 220.0, "F2", Align2::CENTER_CENTER, 11.0, INK);
                line(85.0, 76.0, 85.0, 220.0);
                resistor(85.0, 120.0);
                resistor(85.0, 175.0);
                cap(240.0, 148.0, 76.0, 220.0);
                dot(347.0, 89.0);
                dot(347.0, 207.0);
                line(361.0, 144.0, 439.0, 144.0);
                line(361.0, 152.0, 439.0, 152.0);
                text(
                    400.0,
                    121.0,
                    "L1 · choque CM",
                    Align2::CENTER_CENTER,
                    11.0,
                    MUTED,
                );
                text(
                    400.0,
                    174.0,
                    "Mesmo núcleo",
                    Align2::CENTER_CENTER,
                    10.0,
                    MUTED,
                );
                line(555.0, 76.0, 555.0, 160.0);
                resistor(555.0, 114.0);
                cap(555.0, 169.0, 160.0, 220.0);
                cap(730.0, 112.0, 76.0, 145.0);
                pe(730.0, 154.0);
                text(745.0, 154.0, "PE", Align2::LEFT_CENTER, 10.0, MUTED);
                cap(730.0, 253.0, 220.0, 284.0);
                cap(1015.0, 148.0, 76.0, 220.0);
                if self.parameters.cx3_bypass_nf > 0.0 {
                    cap(946.0, 148.0, 76.0, 220.0);
                    dot(946.0, 76.0);
                    dot(946.0, 220.0);
                    text(
                        946.0,
                        176.0,
                        &format!("Cb {} nF", compact(self.parameters.cx3_bypass_nf)),
                        Align2::CENTER_CENTER,
                        10.0,
                        BLUE,
                    );
                }
                line(36.0, 284.0, 1126.0, 284.0);
                dot(730.0, 284.0);
                pe(400.0, 293.0);
                line(400.0, 284.0, 400.0, 287.0);
                text(18.0, 284.0, "PE", Align2::CENTER_CENTER, 11.0, INK);
                text(1142.0, 284.0, "PE", Align2::CENTER_CENTER, 11.0, INK);
                text(
                    880.0,
                    304.0,
                    &format!(
                        "PE: {} Ω + jω·{} µH para a referência",
                        compact(self.parameters.pe_r_ohm),
                        compact(self.parameters.pe_l_uh)
                    ),
                    Align2::CENTER_CENTER,
                    10.0,
                    MUTED,
                );

                text(
                    400.0,
                    274.0,
                    "Σ: fluxos somando · Δ: fluxos opondo",
                    Align2::CENTER_CENTER,
                    11.0,
                    MUTED,
                );
                let p = &mut self.parameters;
                schematic_field(
                    ui,
                    &at,
                    scale,
                    106.0,
                    109.0,
                    "R1",
                    &mut p.r1_kohm,
                    "kΩ",
                    0.5,
                );
                schematic_field(
                    ui,
                    &at,
                    scale,
                    106.0,
                    164.0,
                    "R2",
                    &mut p.r2_kohm,
                    "kΩ",
                    0.5,
                );
                schematic_field(
                    ui,
                    &at,
                    scale,
                    258.0,
                    136.0,
                    "Cx1",
                    &mut p.cx1_uf,
                    "µF",
                    0.01,
                );
                schematic_field(
                    ui,
                    &at,
                    scale,
                    350.0,
                    38.0,
                    "L1 Σ",
                    &mut p.l1_sum_mh,
                    "mH",
                    0.001,
                );
                schematic_field(
                    ui,
                    &at,
                    scale,
                    350.0,
                    237.0,
                    "L1 Δ",
                    &mut p.l1_opp_uh,
                    "µH",
                    0.01,
                );
                schematic_field(ui, &at, scale, 574.0, 101.0, "Rd", &mut p.rd_ohm, "Ω", 0.1);
                schematic_field(
                    ui,
                    &at,
                    scale,
                    574.0,
                    160.0,
                    "Cx2",
                    &mut p.cx2_uf,
                    "µF",
                    0.01,
                );
                schematic_field(
                    ui,
                    &at,
                    scale,
                    750.0,
                    101.0,
                    "Cy1",
                    &mut p.cy1_nf,
                    "nF",
                    0.1,
                );
                schematic_field(
                    ui,
                    &at,
                    scale,
                    590.0,
                    244.0,
                    "Cy2",
                    &mut p.cy2_nf,
                    "nF",
                    0.1,
                );
                schematic_field(ui, &at, scale, 819.0, 38.0, "L2", &mut p.l2_uh, "µH", 0.01);
                schematic_field(ui, &at, scale, 819.0, 237.0, "L3", &mut p.l3_uh, "µH", 0.01);
                schematic_field(
                    ui,
                    &at,
                    scale,
                    1035.0,
                    136.0,
                    "Cx3",
                    &mut p.cx3_uf,
                    "µF",
                    0.01,
                );
            });
    }

    fn model_controls(&mut self, ui: &mut egui::Ui) {
        egui::Frame::NONE
            .fill(Color32::from_rgb(236, 244, 250))
            .stroke(Stroke::new(1.0f32, BORDER))
            .inner_margin(7.0)
            .show(ui, |ui| {
                if let Err(e) = self.editor.show(ui, &mut self.parameters) {
                    self.error = Some(e);
                }
            });
    }

    fn circuit_and_parameters(&mut self, ui: &mut egui::Ui) {
        let gap = ui.spacing().item_spacing.x;
        let width = ui.available_width();
        let parameters_width = (width * 0.31).clamp(310.0, 390.0);
        let diagram_width = (width - parameters_width - gap).max(1.0);
        ui.horizontal_top(|ui| {
            ui.allocate_ui_with_layout(
                Vec2::new(diagram_width, 0.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    self.diagram(ui);
                    if let Some(group) = ui
                        .ctx()
                        .data_mut(|d| d.remove_temp::<String>(egui::Id::new("component-click")))
                    {
                        self.editor.select(&group);
                    }
                    self.graph_controls(ui);
                },
            );
            ui.allocate_ui_with_layout(
                Vec2::new(parameters_width, 230.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| self.model_controls(ui),
            );
        });
    }

    fn graph_controls(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::horizontal()
            .id_salt("graph-toolbar-scroll")
            .auto_shrink([false, true])
            .show(ui, |ui| {
        ui.horizontal(|ui| {
            for (i, color) in [BLUE, ORANGE, GREEN].into_iter().enumerate() {
                let label = RichText::new(format!("Gráfico {}", i + 1))
                    .color(color)
                    .strong();
                let response = ui.add_enabled(
                    i <= self.history.graphs.len(),
                    egui::Button::new(label)
                        .fill(if i == self.history.selected {
                            Color32::from_rgb(218, 234, 246)
                        } else {
                            Color32::WHITE
                        })
                        .stroke(Stroke::new(
                            1.0f32,
                            if i == self.history.selected {
                                color
                            } else {
                                BORDER
                            },
                        )),
                );
                if response.clicked() {
                    if let Some(snapshot) = self.history.select(i) {
                        self.parameters = snapshot.record.parameters.clone();
                        self.calculated = snapshot.record.parameters;
                        self.simulation = Some(snapshot.simulation);
                        self.error = None;
                        self.status = format!(
                            "Gráfico {} selecionado · {}",
                            i + 1,
                            snapshot.record.timestamp
                        );
                    } else {
                        self.history.edit_target = None;
                        self.calculate();
                    }
                }
                response.context_menu(|ui| {
                    if ui.button("Novo gráfico").clicked() {
                        self.history.edit_target = None;
                        self.calculate();
                        ui.close_menu();
                    }
                });
                response.on_hover_text(self.history.graphs.get(i).map_or(
                    "Gerar próximo gráfico com os valores atuais".into(),
                    |s| {
                        format!(
                            "{} · carregar valores para comparar ou editar; botão direito: novo gráfico",
                            s.record.timestamp
                        )
                    },
                ));
            }
            if ui
                .add(egui::Button::new("Sobrepor").selected(self.history.overlay))
                .clicked()
            {
                self.history.overlay = !self.history.overlay;
            }
            for (i, label) in ["Atenuação", "Fase H", "Formas de onda"].iter().enumerate() {
                ui.selectable_value(&mut self.plot_tab, i, *label);
            }
        });
            });
    }

    fn wave_controls(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            for (i, label) in ["Fase A", "Fase B", "DM", "CM"].iter().enumerate() {
                ui.selectable_value(&mut self.wave_mode, i, *label);
            }
            ui.separator();
            for (i, label) in ["Onda", "dx/dt", "d²x/dt²"].iter().enumerate() {
                ui.selectable_value(&mut self.derivative_order, i, *label);
            }
        });
    }

    fn comparison_plot(&self, ui: &mut egui::Ui) {
        let snapshots = self
            .history
            .graphs
            .iter()
            .enumerate()
            .filter(|(i, _)| self.history.overlay || *i == self.history.selected)
            .collect::<Vec<_>>();
        let mut names = Vec::new();
        let mut data = Vec::new();
        for (i, g) in snapshots {
            let color = [BLUE, ORANGE, GREEN][i];
            if self.plot_tab == 0 {
                for (dm, visible, label) in
                    [(true, self.show_dm, "DM"), (false, self.show_cm, "CM")]
                {
                    if !visible {
                        continue;
                    }
                    let points = g
                        .simulation
                        .points
                        .iter()
                        .map(|p| {
                            [
                                p.frequency_hz,
                                match (dm, self.modal_total) {
                                    (true, true) => p.dm_total_db,
                                    (false, true) => p.cm_total_db,
                                    (true, false) => p.dm_db,
                                    (false, false) => p.cm_db,
                                },
                            ]
                        })
                        .collect();
                    names.push(format!("Gráfico {} · {}", i + 1, label));
                    data.push((color, points));
                }
            } else if self.plot_tab == 1 {
                for (dm, visible, label) in
                    [(true, self.show_dm, "H DM"), (false, self.show_cm, "H CM")]
                {
                    if !visible {
                        continue;
                    }
                    let mut previous = 0.0;
                    let mut phase = 0.0;
                    let points = g
                        .simulation
                        .points
                        .iter()
                        .enumerate()
                        .map(|(j, p)| {
                            let a = if dm {
                                p.transfer.dm_from_dm
                            } else {
                                p.transfer.cm_from_cm
                            }
                            .arg()
                            .to_degrees();
                            if j == 0 {
                                phase = a;
                            } else {
                                phase += (a - previous + 180.0).rem_euclid(360.0) - 180.0;
                            }
                            previous = a;
                            [p.frequency_hz, phase]
                        })
                        .collect();
                    names.push(format!("Gráfico {} · {}", i + 1, label));
                    data.push((color, points));
                }
            } else if let Some(wave) = &g.simulation.waveform {
                let points = wave
                    .iter()
                    .map(|p| {
                        let v = p.after[self.derivative_order];
                        let y = match self.wave_mode {
                            0 => v[0],
                            1 => v[1],
                            2 => v[0] - v[1],
                            _ => (v[0] + v[1]) / 2.0,
                        };
                        [p.time_s * 1e6, y]
                    })
                    .collect();
                names.push(format!("Gráfico {} · com filtro", i + 1));
                data.push((color, points));
            }
        }
        let series = names
            .iter()
            .zip(data)
            .map(|(name, (color, points))| (name.as_str(), color, points))
            .collect::<Vec<_>>();
        let (title, x, y) = match self.plot_tab {
            0 => ("Comparação · atenuação na rede", "Hz", "dB"),
            1 => ("Comparação · fase de H", "Hz", "graus"),
            _ => (
                "Comparação · ruído filtrado na rede",
                "µs",
                ["V", "V/s", "V/s²"][self.derivative_order],
            ),
        };
        scientific_plot(
            ui,
            (title, self.selected_timestamp()),
            x,
            y,
            &series,
            self.plot_tab != 2,
            (self.plot_tab == 0 && !self.automatic_db)
                .then_some([self.plot_min_db, self.plot_max_db]),
        );
        if self.plot_tab == 0 {
            ui.small("Perda de inserção de tensão em relação à mesma rede sem filtro. Cor identifica o gráfico; DM contínuo, CM tracejado. Harmônicas da excitação aparecem na tabela do gráfico selecionado.");
        }
    }

    fn plot(&self, ui: &mut egui::Ui) {
        let height = (ui.ctx().screen_rect().bottom() - ui.cursor().top() - 85.0).max(155.0);
        let (rect, response) =
            ui.allocate_exact_size(Vec2::new(ui.available_width(), height), Sense::hover());
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 0.0, Color32::WHITE);
        let legend = [
            (self.show_dm, "DM", BLUE),
            (self.show_cm, "CM", ORANGE),
            (self.show_harmonics, "Excitação · harmônicas", GREEN),
        ]
        .into_iter()
        .filter(|(visible, _, _)| *visible)
        .map(|(_, label, color)| (label, color, false))
        .collect::<Vec<_>>();
        let chart = Rect::from_min_max(
            rect.min + Vec2::new(51.0, 16.0),
            rect.max - Vec2::new(17.0, 37.0),
        );
        let Some(result) = &self.simulation else {
            plot_header(
                &painter.with_clip_rect(chart),
                chart,
                "Atenuação na rede",
                self.selected_timestamp(),
                &legend,
            );
            painter.text(
                rect.center(),
                Align2::CENTER_CENTER,
                "Calcule o gráfico para visualizar a resposta",
                FontId::proportional(15.0),
                MUTED,
            );
            return;
        };
        let lo = self.calculated.sweep_min_hz.max(1e-6).log10();
        let hi = self
            .calculated
            .sweep_max_hz
            .max(self.calculated.sweep_min_hz * 1.01)
            .log10();
        let (mut ymin, mut ymax) = (self.plot_min_db, self.plot_max_db);
        if self.automatic_db {
            ymin = 0.0;
            ymax = 0.0;
            for point in &result.points {
                for (value, visible) in [
                    (
                        if self.modal_total {
                            point.dm_total_db
                        } else {
                            point.dm_db
                        },
                        self.show_dm,
                    ),
                    (
                        if self.modal_total {
                            point.cm_total_db
                        } else {
                            point.cm_db
                        },
                        self.show_cm,
                    ),
                ] {
                    if visible && value.is_finite() {
                        ymin = ymin.min(value);
                        ymax = ymax.max(value);
                    }
                }
            }
            if self.show_harmonics {
                for point in &result.harmonics {
                    if point.combined_db.is_finite()
                        && point.frequency_hz >= self.calculated.sweep_min_hz
                        && point.frequency_hz <= self.calculated.sweep_max_hz
                    {
                        ymin = ymin.min(point.combined_db);
                        ymax = ymax.max(point.combined_db);
                    }
                }
            }
            ymin = (ymin / 10.0).floor() * 10.0 - 5.0;
            ymax = (ymax / 10.0).ceil() * 10.0 + 5.0;
        }
        if ymax <= ymin {
            ymax = ymin + 10.0;
        }
        let pos = |f: f64, db: f64| {
            Pos2::new(
                chart.left() + ((f.log10() - lo) / (hi - lo)) as f32 * chart.width(),
                chart.bottom() - ((db - ymin) / (ymax - ymin)) as f32 * chart.height(),
            )
        };
        let clip = painter.with_clip_rect(chart);
        for exponent in lo.floor() as i32..=hi.ceil() as i32 {
            for multiplier in 1..10 {
                let f = 10f64.powi(exponent) * multiplier as f64;
                let x = pos(f, 0.0).x;
                if x >= chart.left() && x <= chart.right() {
                    let major = multiplier == 1 || multiplier == 2 || multiplier == 5;
                    painter.line_segment(
                        [Pos2::new(x, chart.top()), Pos2::new(x, chart.bottom())],
                        Stroke::new(
                            if multiplier == 1 { 0.9f32 } else { 0.45f32 },
                            if multiplier == 1 {
                                BORDER
                            } else {
                                Color32::from_rgb(228, 237, 243)
                            },
                        ),
                    );
                    if major {
                        painter.text(
                            Pos2::new(x, chart.bottom() + 13.0),
                            Align2::CENTER_CENTER,
                            frequency_label(f),
                            FontId::monospace(11.0),
                            MUTED,
                        );
                    }
                }
            }
        }
        let ystep = if ymax - ymin > 120.0 { 20.0 } else { 10.0 };
        let mut ytick = (ymin / ystep).ceil() * ystep;
        while ytick <= ymax {
            let y = pos(10f64.powf(lo), ytick).y;
            painter.line_segment(
                [Pos2::new(chart.left(), y), Pos2::new(chart.right(), y)],
                Stroke::new(
                    if ytick.abs() < 0.01 { 1.0f32 } else { 0.5f32 },
                    if ytick.abs() < 0.01 { MUTED } else { BORDER },
                ),
            );
            painter.text(
                Pos2::new(chart.left() - 7.0, y),
                Align2::RIGHT_CENTER,
                compact(ytick),
                FontId::monospace(11.0),
                MUTED,
            );
            ytick += ystep;
        }
        painter.line_segment(
            [chart.left_bottom(), chart.right_bottom()],
            Stroke::new(1.0f32, BORDER),
        );
        painter.line_segment(
            [chart.left_bottom(), chart.left_top()],
            Stroke::new(1.0f32, BORDER),
        );
        painter.text(
            Pos2::new(rect.left() + 13.0, chart.top()),
            Align2::LEFT_CENTER,
            "dB",
            FontId::monospace(11.0),
            MUTED,
        );
        painter.text(
            Pos2::new(chart.right(), rect.bottom() - 6.0),
            Align2::RIGHT_BOTTOM,
            "Hz",
            FontId::monospace(11.0),
            MUTED,
        );
        for (is_dm, visible, color) in [(true, self.show_dm, BLUE), (false, self.show_cm, ORANGE)] {
            if visible {
                let points = result
                    .points
                    .iter()
                    .filter_map(|p| {
                        let db = match (is_dm, self.modal_total) {
                            (true, true) => p.dm_total_db,
                            (false, true) => p.cm_total_db,
                            (true, false) => p.dm_db,
                            (false, false) => p.cm_db,
                        };
                        (db.is_finite() && p.frequency_hz > 0.0).then(|| pos(p.frequency_hz, db))
                    })
                    .collect::<Vec<_>>();
                clip.add(egui::Shape::line(points, Stroke::new(1.9f32, color)));
            }
        }
        if self.show_harmonics {
            for h in &result.harmonics {
                if !h.combined_db.is_finite()
                    || h.frequency_hz < self.calculated.sweep_min_hz
                    || h.frequency_hz > self.calculated.sweep_max_hz
                {
                    continue;
                }
                let xy = pos(h.frequency_hz, h.combined_db);
                clip.circle_filled(xy, 4.2, GREEN);
                clip.circle_stroke(xy, 5.2, Stroke::new(1.2f32, Color32::WHITE));
                clip.text(
                    xy + Vec2::new(6.0, -8.0),
                    Align2::LEFT_BOTTOM,
                    format!("H{}", h.order),
                    FontId::monospace(10.0),
                    GREEN,
                );
            }
        }
        if let Some(cursor) = response.hover_pos().filter(|p| chart.contains(*p)) {
            let freq =
                10f64.powf(lo + ((cursor.x - chart.left()) / chart.width()) as f64 * (hi - lo));
            if let Some(nearest) = result.points.iter().min_by(|a, b| {
                (a.frequency_hz - freq)
                    .abs()
                    .total_cmp(&(b.frequency_hz - freq).abs())
            }) {
                let x = pos(nearest.frequency_hz, 0.0).x;
                painter.line_segment(
                    [Pos2::new(x, chart.top()), Pos2::new(x, chart.bottom())],
                    Stroke::new(0.8f32, MUTED),
                );
                response.clone().on_hover_ui_at_pointer(|ui| {
                    ui.label(
                        RichText::new(format!("{:.3} kHz", nearest.frequency_hz / 1000.0))
                            .strong()
                            .monospace(),
                    );
                    ui.colored_label(
                        BLUE,
                        format!(
                            "DM  {:.3} dB",
                            if self.modal_total {
                                nearest.dm_total_db
                            } else {
                                nearest.dm_db
                            }
                        ),
                    );
                    ui.colored_label(
                        ORANGE,
                        format!(
                            "CM  {:.3} dB",
                            if self.modal_total {
                                nearest.cm_total_db
                            } else {
                                nearest.cm_db
                            }
                        ),
                    );
                    if let Some(h) = result
                        .harmonics
                        .iter()
                        .min_by(|a, b| {
                            (a.frequency_hz - freq)
                                .abs()
                                .total_cmp(&(b.frequency_hz - freq).abs())
                        })
                        .filter(|h| (h.frequency_hz / freq).ln().abs() < 0.03)
                    {
                        ui.colored_label(
                            GREEN,
                            format!("H{} conjunta  {:.3} dB", h.order, h.combined_db),
                        );
                    }
                });
            }
        }
        plot_header(
            &painter.with_clip_rect(chart),
            chart,
            "Atenuação na rede",
            self.selected_timestamp(),
            &legend,
        );
    }

    fn selected_timestamp(&self) -> Option<&str> {
        self.history
            .graphs
            .get(self.history.selected)
            .map(|snapshot| snapshot.record.timestamp.as_str())
    }

    fn additional_plot(&mut self, ui: &mut egui::Ui) {
        if self.plot_tab == 1 {
            let Some(s) = &self.simulation else {
                return;
            };
            let mut series = Vec::new();
            for (dm, visible, label, color) in [
                (true, self.show_dm, "H DM←DM", BLUE),
                (false, self.show_cm, "H CM←CM", ORANGE),
            ] {
                if !visible {
                    continue;
                }
                let mut previous = 0.0;
                let mut phase = 0.0;
                let points = s
                    .points
                    .iter()
                    .enumerate()
                    .map(|(i, p)| {
                        let a = if dm {
                            p.transfer.dm_from_dm
                        } else {
                            p.transfer.cm_from_cm
                        }
                        .arg()
                        .to_degrees();
                        if i == 0 {
                            phase = a;
                        } else {
                            phase += (a - previous + 180.0).rem_euclid(360.0) - 180.0;
                        }
                        previous = a;
                        [p.frequency_hz, phase]
                    })
                    .collect();
                series.push((label, color, points));
            }
            scientific_plot(
                ui,
                (
                    "Fase da transferência · fonte em aberto para a rede",
                    self.selected_timestamp(),
                ),
                "Hz",
                "graus",
                &series,
                true,
                None,
            );
            ui.label(RichText::new("Fase desenrolada para visualização. CSV contém Re/Im e fase principal; termos cruzados DM↔CM também são exportados.").small().color(MUTED));
        } else {
            ui.horizontal(|ui| {
                for (i, label) in ["Fase A", "Fase B", "DM", "CM"].iter().enumerate() {
                    ui.selectable_value(&mut self.wave_mode, i, *label);
                }
                ui.separator();
                for (i, label) in ["Onda", "dx/dt", "d²x/dt²"].iter().enumerate() {
                    ui.selectable_value(&mut self.derivative_order, i, *label);
                }
            });
            let Some(s) = &self.simulation else {
                return;
            };
            let Some(wave) = &s.waveform else {
                ui.colored_label(ORANGE,"Série longa demais para a reconstrução temporal com amostragem adequada. Reduza a razão entre o limite da série e a fundamental.");
                return;
            };
            let mut series = Vec::new();
            for (filtered, label, color) in
                [(false, "Sem filtro", ORANGE), (true, "Com filtro", BLUE)]
            {
                let points = wave
                    .iter()
                    .map(|t| {
                        let v = if filtered {
                            t.after[self.derivative_order]
                        } else {
                            t.before[self.derivative_order]
                        };
                        let y = match self.wave_mode {
                            0 => v[0],
                            1 => v[1],
                            2 => v[0] - v[1],
                            _ => (v[0] + v[1]) / 2.0,
                        };
                        [t.time_s * 1e6, y]
                    })
                    .collect();
                series.push((label, color, points));
            }
            let unit = ["V", "V/s", "V/s²"][self.derivative_order];
            scientific_plot(
                ui,
                (
                    "Ruído na rede · mesma escala antes e depois",
                    self.selected_timestamp(),
                ),
                "µs",
                unit,
                &series,
                false,
                None,
            );
            ui.label(RichText::new(format!("Dois períodos da série até {} Hz, sem os 60 Hz da alimentação. Reconstrução com H complexa; não linearidade e harmônicas novas não incluídas.",compact(self.calculated.noise_max_hz))).small().color(MUTED));
        }
    }

    fn harmonic_table(&self, ui: &mut egui::Ui) {
        egui::CollapsingHeader::new("Tabela das harmônicas e perdas integradas").show(ui, |ui| {
            if let Some(result) = &self.simulation {
                egui::Grid::new("harmonic-table").striped(true).min_col_width(105.0).show(ui,|ui| {
                    for name in ["Harmônica","Frequência","Conjunta / dB","DM / dB","CM / dB","Fase A / dB","Fase B / dB"] { ui.strong(name); } ui.end_row();
                    for h in &result.harmonics {
                        ui.monospace(format!("H{}",h.order)); ui.monospace(format!("{} kHz",compact(h.frequency_hz/1000.0)));
                        ui.monospace(format!("{:.3}",h.combined_db)); ui.monospace(optional_db(h.dm_db));ui.monospace(optional_db(h.cm_db));
                        ui.monospace(format!("{:.3}",h.phase_a_db));ui.monospace(format!("{:.3}",h.phase_b_db));ui.end_row();
                    }
                });
                ui.label(RichText::new("— indica ausência de referência naquele modo de entrada; ainda pode haver conversão para ele. Veja a matriz H complexa. Atenuação conjunta = 10 log10(Σ|Vsem|² / Σ|Vcom|²).").small().color(MUTED));
            }
        });
    }
}

impl eframe::App for EmiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if ctx
            .input(|i| i.key_pressed(egui::Key::Enter) && (i.modifiers.ctrl || i.modifiers.command))
        {
            self.calculate();
        }
        egui::TopBottomPanel::top("toolbar")
            .frame(
                egui::Frame::NONE
                    .fill(Color32::from_rgb(235, 242, 247))
                    .inner_margin(9.0),
            )
            .show(ctx, |ui| self.toolbar(ui));
        egui::TopBottomPanel::bottom("status")
            .frame(
                egui::Frame::NONE
                    .fill(Color32::from_rgb(225, 236, 244))
                    .inner_margin(5.0),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let dirty = self.parameters != self.calculated;
                    ui.colored_label(
                        if dirty { ORANGE } else { GREEN },
                        if dirty {
                            "Valores alterados · calcular para atualizar"
                        } else {
                            "Calculado"
                        },
                    );
                    ui.separator();
                    ui.label(&self.status);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            RichText::new(format!(
                                "v{} · Ctrl+Enter calcula · Rust nativo",
                                env!("CARGO_PKG_VERSION")
                            ))
                            .small()
                            .color(MUTED),
                        );
                    });
                });
            });
        egui::SidePanel::left("channels")
            .exact_width(210.0)
            .resizable(false)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .id_salt("sidebar-scroll")
                    .show(ui, |ui| self.sidebar(ui));
            });
        egui::CentralPanel::default().frame(egui::Frame::NONE.fill(Color32::from_rgb(249,251,252)).inner_margin(10.0)).show(ctx,|ui| {
            egui::ScrollArea::vertical().id_salt("main-scroll").auto_shrink([false,false]).show(ui,|ui| {
                self.circuit_and_parameters(ui);
                if let Some(error)=&self.error { ui.colored_label(ORANGE,format!("Erro: {error}")); }
                ui.add_space(2.0);
                if self.history.overlay {
                    if self.plot_tab==2 {self.wave_controls(ui);}
                    self.comparison_plot(ui);
                } else if self.plot_tab==0 {self.plot(ui);} else {self.additional_plot(ui);}
                self.harmonic_table(ui);
                ui.label(RichText::new(format!("Rede: Z(f) = R + j2πfL, referência em {} Hz. Rs/ESR constantes sem tabela Z(f). Harmônicas até {} Hz.",compact(self.parameters.grid_z_ref_hz),compact(self.parameters.noise_max_hz))).small().color(MUTED))
                    .on_hover_text(format!("A reatância da rede cresce linearmente com a frequência.\n|Z| por fase: {:.3} Ω em {:.0} Hz; {:.3} Ω em 10 kHz; {:.3} Ω em 1 MHz.\nAs portas DM e CM são calculadas separadamente. As resistências equivalentes de perdas dos componentes são aproximações constantes, salvo tabela Z(f) ativa. Valores editáveis não validam eletricamente os componentes.",self.parameters.grid_phase_impedance(self.parameters.grid_z_ref_hz).norm(),self.parameters.grid_z_ref_hz,self.parameters.grid_phase_impedance(10000.0).norm(),self.parameters.grid_phase_impedance(1000000.0).norm()));
            });
        });
        if let Ok(path) = std::env::var("EMI_SCREENSHOT") {
            self.screenshot_frames += 1;
            if self.screenshot_frames == 4 {
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
            }
            let screenshot = ctx.input(|i| {
                i.events.iter().find_map(|event| {
                    if let egui::Event::Screenshot { image, .. } = event {
                        Some(image.clone())
                    } else {
                        None
                    }
                })
            });
            if let Some(image) = screenshot {
                let bytes: Vec<u8> = image
                    .pixels
                    .iter()
                    .flat_map(|pixel| pixel.to_array())
                    .collect();
                let result = image::save_buffer(
                    &path,
                    &bytes,
                    image.size[0] as u32,
                    image.size[1] as u32,
                    image::ColorType::Rgba8,
                );
                if let Err(error) = result {
                    eprintln!("Screenshot: {error}");
                }
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            ctx.request_repaint();
        }
    }
}

fn section_bar(ui: &mut egui::Ui, label: &str) {
    egui::Frame::NONE
        .fill(CYAN)
        .inner_margin(Vec2::new(6.0, 5.0))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(RichText::new(label).small().strong().color(BLUE));
        });
}

fn toolbar_measure(ui: &mut egui::Ui, title: &str, value: &str, detail: &str) {
    egui::Frame::NONE
        .fill(Color32::WHITE)
        .inner_margin(Vec2::new(10.0, 5.0))
        .show(ui, |ui| {
            ui.vertical(|ui| {
                ui.label(RichText::new(title).small().color(MUTED));
                ui.label(RichText::new(value).strong().color(BLUE));
                ui.label(RichText::new(detail).small().color(MUTED));
            });
        });
}

fn channel_toggle(
    ui: &mut egui::Ui,
    enabled: &mut bool,
    color: Color32,
    channel: &str,
    title: &str,
    detail: &str,
) {
    egui::Frame::NONE
        .fill(Color32::WHITE)
        .stroke(Stroke::new(1.0f32, BORDER))
        .inner_margin(6.0)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                egui::Frame::NONE
                    .fill(color)
                    .inner_margin(Vec2::new(5.0, 6.0))
                    .show(ui, |ui| {
                        ui.label(RichText::new(channel).strong().color(Color32::WHITE));
                    });
                ui.vertical(|ui| {
                    ui.checkbox(enabled, title);
                    ui.label(RichText::new(detail).small().color(MUTED));
                });
            });
        });
}

fn numeric_row(ui: &mut egui::Ui, label: &str, value: &mut f64, unit: &str, speed: f64) {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add(
                egui::DragValue::new(value)
                    .speed(speed)
                    .max_decimals(6)
                    .suffix(if unit.is_empty() {
                        String::new()
                    } else {
                        format!(" {unit}")
                    }),
            );
        });
    });
}

// Posição, escala e valor pertencem ao mesmo primitivo do esquema.
#[allow(clippy::too_many_arguments)]
fn schematic_field(
    ui: &mut egui::Ui,
    at: &impl Fn(f32, f32) -> Pos2,
    scale: f32,
    x: f32,
    y: f32,
    label: &str,
    value: &mut f64,
    unit: &str,
    speed: f64,
) {
    let rect = Rect::from_min_size(at(x, y), Vec2::new(115.0 * scale, 29.0 * scale));
    ui.allocate_new_ui(egui::UiBuilder::new().max_rect(rect), |ui| {
        ui.spacing_mut().item_spacing = Vec2::new(3.0, 2.0);
        ui.style_mut()
            .text_styles
            .insert(egui::TextStyle::Body, FontId::proportional(11.0 * scale));
        ui.style_mut()
            .text_styles
            .insert(egui::TextStyle::Button, FontId::monospace(11.0 * scale));
        ui.style_mut().spacing.interact_size.y = 20.0 * scale;
        ui.horizontal(|ui| {
            if ui
                .link(RichText::new(label).strong().color(BLUE))
                .on_hover_text("Abrir parâmetros deste componente")
                .clicked()
            {
                let group = label.split_whitespace().next().unwrap_or(label).to_owned();
                ui.ctx()
                    .data_mut(|d| d.insert_temp(egui::Id::new("component-click"), group));
            }
            ui.add(
                egui::DragValue::new(value)
                    .speed(speed)
                    .max_decimals(5)
                    .suffix(format!(" {unit}")),
            );
        });
    });
}

fn compact(value: f64) -> String {
    let s = format!("{value:.3}");
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

fn frequency_label(f: f64) -> String {
    if f >= 1000.0 {
        format!("{}k", compact(f / 1000.0))
    } else {
        compact(f)
    }
}

fn optional_db(value: Option<f64>) -> String {
    value
        .map(|v| format!("{v:.3}"))
        .unwrap_or_else(|| "—".into())
}

fn finite_db(value: f64) -> String {
    if value.is_finite() {
        format!("{value:.2}")
    } else {
        "—".into()
    }
}
