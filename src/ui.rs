use crate::engine::{self, Parameters, Simulation};
use eframe::egui::{self, Align2, Color32, FontId, Pos2, Rect, RichText, Sense, Stroke, Vec2};
use std::f64::consts::PI;

const BLUE: Color32 = Color32::from_rgb(42, 120, 186);
const CYAN: Color32 = Color32::from_rgb(222, 238, 249);
const ORANGE: Color32 = Color32::from_rgb(204, 91, 38);
const GREEN: Color32 = Color32::from_rgb(22, 137, 97);
const INK: Color32 = Color32::from_rgb(32, 57, 75);
const MUTED: Color32 = Color32::from_rgb(90, 115, 130);
const BORDER: Color32 = Color32::from_rgb(198, 215, 226);

pub struct EmiApp {
    parameters: Parameters,
    calculated: Parameters,
    simulation: Option<Simulation>,
    error: Option<String>,
    status: String,
    duration_ms: f64,
    model_tab: usize,
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

        let parameters = Parameters::default();
        let started = std::time::Instant::now();
        let result = engine::calculate(&parameters);
        let (simulation, error) = match result {
            Ok(result) => (Some(result), None),
            Err(error) => (None, Some(error)),
        };
        Self {
            calculated: parameters.clone(),
            parameters,
            simulation,
            error,
            status: "Modelo original carregado".into(),
            duration_ms: started.elapsed().as_secs_f64() * 1000.0,
            model_tab: 0,
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
                self.simulation = Some(simulation);
                self.calculated = self.parameters.clone();
                self.error = None;
                self.duration_ms = started.elapsed().as_secs_f64() * 1000.0;
                self.status = format!("Gráfico atualizado · {:.1} ms", self.duration_ms);
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
                ui.label(RichText::new("Bancada de simulação").small().color(MUTED));
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
                "2 × quadrada",
                &format!(
                    "{} kHz · {}°",
                    compact(self.parameters.noise_frequency_hz / 1000.0),
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
                if ui.button("Restaurar valores").clicked() {
                    self.parameters = Parameters::default();
                    self.calculate();
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
            "Duas quadradas",
            "Harmônicas presentes",
        );
        ui.add_space(6.0);
        ui.checkbox(&mut self.automatic_db, "Escala dB automática");
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
                match serde_json::to_string_pretty(&self.parameters)
                    .map_err(|e| e.to_string())
                    .and_then(|s| std::fs::write(&self.json_path, s).map_err(|e| e.to_string()))
                {
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
                    .and_then(|s| serde_json::from_str::<Parameters>(&s).map_err(|e| e.to_string()))
                {
                    Ok(parameters) => {
                        self.parameters = parameters;
                        self.calculate();
                    }
                    Err(error) => self.error = Some(format!("Não foi possível abrir: {error}")),
                }
            }
        });
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
        ui.add_space(10.0);
        ui.label(
            RichText::new("Modelo linear · perdas extrapoladas\nSem saturação ou aquecimento")
                .small()
                .color(MUTED),
        );
    }

    fn diagram(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Circuito original").size(17.0).strong());
            ui.label(
                RichText::new("Edite os valores diretamente no esquema")
                    .small()
                    .color(MUTED),
            );
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
                let (rect, _) =
                    ui.allocate_exact_size(Vec2::new(width, 314.0 * scale), Sense::hover());
                let painter = ui.painter_at(rect);
                let at =
                    |x: f32, y: f32| Pos2::new(rect.left() + x * scale, rect.top() + y * scale);
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
                text(1137.0, 19.0, "3 SERVOS", Align2::RIGHT_CENTER, 12.0, BLUE);
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
                line(36.0, 284.0, 1126.0, 284.0);
                dot(730.0, 284.0);
                pe(400.0, 293.0);
                line(400.0, 284.0, 400.0, 287.0);
                text(18.0, 284.0, "PE", Align2::CENTER_CENTER, 11.0, INK);
                text(1142.0, 284.0, "PE", Align2::CENTER_CENTER, 11.0, INK);
                text(
                    880.0,
                    304.0,
                    "PE direto · todos os símbolos representam a mesma conexão",
                    Align2::CENTER_CENTER,
                    10.0,
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
        ui.label(RichText::new("Σ = série somando fluxos; Δ = série opondo. Cx: classe X2. Cy: classe Y. Valores do circuito final preservados.").small().color(MUTED));
    }

    fn model_controls(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Modelo").strong());
            for (index, label) in [
                "Impedâncias e fontes",
                "Perdas e parasitas",
                "Alimentação e varredura",
            ]
            .iter()
            .enumerate()
            {
                ui.selectable_value(&mut self.model_tab, index, *label);
            }
        });
        egui::Frame::NONE.fill(Color32::from_rgb(236, 244, 250)).stroke(Stroke::new(1.0f32, BORDER)).inner_margin(8.0).show(ui, |ui| {
            let p = &mut self.parameters;
            match self.model_tab {
                0 => {
                    ui.columns(4, |cols| {
                        numeric_row(&mut cols[0], "Rede / fase", &mut p.grid_phase_z_ohm, "Ω", 0.01);
                        numeric_row(&mut cols[0], "Referência", &mut p.grid_z_ref_hz, "Hz", 1.0);
                        numeric_row(&mut cols[0], "R da fase", &mut p.grid_phase_r_ohm, "Ω", 0.01);
                        numeric_row(&mut cols[1], "Rede CM total", &mut p.grid_cm_z_ohm, "Ω", 0.01);
                        numeric_row(&mut cols[1], "R rede CM", &mut p.grid_cm_r_ohm, "Ω", 0.01);
                        numeric_row(&mut cols[1], "Fonte / fase", &mut p.source_phase_r_ohm, "Ω", 1.0);
                        numeric_row(&mut cols[2], "Fonte CM total", &mut p.source_cm_r_ohm, "Ω", 10.0);
                        numeric_row(&mut cols[2], "R do PE", &mut p.pe_r_ohm, "Ω", 0.01);
                        numeric_row(&mut cols[2], "L do PE", &mut p.pe_l_uh, "µH", 0.01);
                        numeric_row(&mut cols[3], "Quadrada", &mut p.noise_frequency_hz, "Hz", 100.0);
                        numeric_row(&mut cols[3], "Atraso A -> B", &mut p.noise_delay_deg, "°", 1.0);
                        numeric_row(&mut cols[3], "Fase inicial A", &mut p.noise_phase_a_deg, "°", 1.0);
                    });
                    let l = ((p.grid_phase_z_ohm.powi(2)-p.grid_phase_r_ohm.powi(2)).max(0.0)).sqrt()/(2.0*PI*p.grid_z_ref_hz);
                    ui.label(RichText::new(format!("Zfase(f) = R + j·2πfL      L = √(Zref² − R²) / (2πfref) = {:.6} mH/fase      ZDM = 2·Zfase", l*1000.0)).font(FontId::monospace(11.0)).color(BLUE));
                    ui.label(RichText::new("Rede CM normalizada na mesma frequência. Fonte CM é a porta total. Portadoras livres, sem reinício nos zeros dos 60 Hz.").small().color(MUTED));
                }
                1 => {
                    ui.columns(4, |cols| {
                        numeric_row(&mut cols[0], "DCR L1 / enrol.", &mut p.l1_dcr_mohm, "mΩ", 0.1);
                        numeric_row(&mut cols[0], "Q CM de L1", &mut p.l1_q, "", 0.5);
                        numeric_row(&mut cols[0], "Medição de Q", &mut p.l1_q_hz, "Hz", 10.0);
                        numeric_row(&mut cols[0], "Cp / enrol.", &mut p.choke_cp_pf, "pF", 1.0);
                        numeric_row(&mut cols[1], "R AC de L2", &mut p.l2_rs_ohm, "Ω", 0.001);
                        numeric_row(&mut cols[1], "R AC de L3", &mut p.l3_rs_ohm, "Ω", 0.001);
                        numeric_row(&mut cols[1], "ESR Cx1", &mut p.cx1_esr_ohm, "Ω", 0.001);
                        numeric_row(&mut cols[1], "ESL Cx1", &mut p.cx1_esl_nh, "nH", 1.0);
                        numeric_row(&mut cols[2], "ESR Cx2", &mut p.cx2_esr_ohm, "Ω", 0.001);
                        numeric_row(&mut cols[2], "ESL Cx2", &mut p.cx2_esl_nh, "nH", 1.0);
                        numeric_row(&mut cols[2], "ESR Cx3", &mut p.cx3_esr_ohm, "Ω", 0.001);
                        numeric_row(&mut cols[2], "ESL Cx3", &mut p.cx3_esl_nh, "nH", 1.0);
                        numeric_row(&mut cols[3], "ESR de cada Cy", &mut p.cy_esr_ohm, "Ω", 0.001);
                        numeric_row(&mut cols[3], "ESL de cada Cy", &mut p.cy_esl_nh, "nH", 1.0);
                    });
                    ui.label(RichText::new("Perdas AC extrapoladas constantes. R AC de L2/L3 não é DCR para calcular aquecimento. Parasitas em zero = aproximação, não medição.").small().color(MUTED));
                }
                _ => {
                    ui.columns(4, |cols| {
                        numeric_row(&mut cols[0], "Tensão / fase", &mut p.mains_phase_v_rms, "V RMS", 1.0);
                        numeric_row(&mut cols[0], "Fundamental", &mut p.mains_frequency_hz, "Hz", 1.0);
                        numeric_row(&mut cols[0], "Atraso das fases", &mut p.mains_phase_deg, "°", 1.0);
                        numeric_row(&mut cols[1], "Consumo conjunto", &mut p.load_rms_a, "A RMS", 0.1);
                        numeric_row(&mut cols[1], "Pico", &mut p.load_peak_a, "A", 0.5);
                        numeric_row(&mut cols[1], "Quadrada ideal", &mut p.noise_peak_v, "V pico", 0.1);
                        numeric_row(&mut cols[2], "Início da curva", &mut p.sweep_min_hz, "Hz", 100.0);
                        numeric_row(&mut cols[2], "Fim da curva", &mut p.sweep_max_hz, "Hz", 1000.0);
                        cols[2].horizontal(|ui| { ui.label("Pontos"); ui.add(egui::DragValue::new(&mut p.sweep_points).range(32..=50000).speed(100)); });
                        numeric_row(&mut cols[3], "Série até", &mut p.noise_max_hz, "Hz", 1000.0);
                        cols[3].label(RichText::new(format!("Entre fases: {:.2} V RMS",2.0*p.mains_phase_v_rms*(0.5*p.mains_phase_deg.to_radians()).sin().abs())).small().color(BLUE));
                    });
                    ui.label(RichText::new("Fundamental e ruído têm fases independentes. O modelo linear não inclui os retificadores dos servos, saturação ou efeito da corrente.").small().color(MUTED));
                }
            }
        });
    }

    fn plot(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Atenuação na rede").size(17.0).strong());
            ui.label(
                RichText::new("dB · perda de inserção de tensão · frequência logarítmica")
                    .small()
                    .color(MUTED),
            );
        });
        let height =
            (ui.ctx().screen_rect().bottom() - ui.cursor().top() - 85.0).clamp(230.0, 345.0);
        let (rect, response) =
            ui.allocate_exact_size(Vec2::new(ui.available_width(), height), Sense::hover());
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 0.0, Color32::WHITE);
        let chart = Rect::from_min_max(
            rect.min + Vec2::new(51.0, 16.0),
            rect.max - Vec2::new(17.0, 37.0),
        );
        let Some(result) = &self.simulation else {
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
                for (value, visible) in [(point.dm_db, self.show_dm), (point.cm_db, self.show_cm)] {
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
                format!("{ytick:.0}"),
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
                        let db = if is_dm { p.dm_db } else { p.cm_db };
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
        let mut lx = chart.left() + 8.0;
        for (visible, color, label) in [
            (self.show_dm, BLUE, "DM"),
            (self.show_cm, ORANGE, "CM"),
            (self.show_harmonics, GREEN, "Quadradas · harmônicas"),
        ] {
            if visible {
                let y = chart.top() + 12.0;
                painter.line_segment(
                    [Pos2::new(lx, y), Pos2::new(lx + 16.0, y)],
                    Stroke::new(2.5f32, color),
                );
                painter.text(
                    Pos2::new(lx + 22.0, y),
                    Align2::LEFT_CENTER,
                    label,
                    FontId::proportional(12.0),
                    color,
                );
                lx += if label.len() > 3 { 190.0 } else { 75.0 };
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
                    ui.colored_label(BLUE, format!("DM  {:.3} dB", nearest.dm_db));
                    ui.colored_label(ORANGE, format!("CM  {:.3} dB", nearest.cm_db));
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
                ui.label(RichText::new("— indica modo ausente na excitação daquela harmônica. Atenuação conjunta = 10 log10(Σ|Vsem|² / Σ|Vcom|²).").small().color(MUTED));
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
                            RichText::new("Ctrl+Enter calcula · Rust nativo")
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
                self.diagram(ui);
                ui.add_space(7.0);
                self.model_controls(ui);
                if let Some(error)=&self.error { ui.colored_label(ORANGE,format!("Erro: {error}")); }
                ui.add_space(8.0);
                self.plot(ui);
                self.harmonic_table(ui);
                ui.label(RichText::new(format!("Hipóteses: rede indutiva normalizada a {} Hz; perdas constantes; série truncada. Valores editáveis não são validação elétrica dos componentes.",compact(self.parameters.grid_z_ref_hz))).small().color(MUTED));
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
            ui.label(RichText::new(label).strong().color(BLUE));
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
