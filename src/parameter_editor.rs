use crate::engine::{extended::ZPoint, Parameters};
use eframe::egui::{self, RichText};
use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Clone, Deserialize)]
pub struct Label {
    pub group: String,
    pub label: String,
    pub unit: String,
    pub note: String,
    pub kind: String,
}
#[derive(Default)]
pub struct Editor {
    pub group: String,
    search: String,
    labels: BTreeMap<String, Label>,
}
impl Editor {
    pub fn new() -> Self {
        Self {
            group: "Cx1".into(),
            search: String::new(),
            labels: serde_json::from_str(include_str!("../assets/parameter-labels.json"))
                .expect("rótulos incorporados válidos"),
        }
    }
    pub fn select(&mut self, group: &str) {
        self.group = group.into();
        self.search.clear();
    }
    pub fn show(&mut self, ui: &mut egui::Ui, p: &mut Parameters) -> Result<(), String> {
        let mut value = serde_json::to_value(&*p).map_err(|e| e.to_string())?;
        let mut fields = Vec::new();
        flatten("", &value, &mut fields);
        let fields = fields
            .into_iter()
            .map(|path| {
                let label = self
                    .labels
                    .get(&path)
                    .cloned()
                    .unwrap_or_else(|| extra_label(&path));
                (path, label)
            })
            .collect::<Vec<_>>();
        let mut groups = fields
            .iter()
            .map(|(_, m)| m.group.clone())
            .collect::<Vec<_>>();
        groups.sort();
        groups.dedup();
        let mut changed = false;
        ui.horizontal_wrapped(|ui| {
            ui.strong("Componente / parâmetros");
            egui::ComboBox::from_id_salt("component-group")
                .selected_text(&self.group)
                .width(155.0)
                .show_ui(ui, |ui| {
                    for group in &groups {
                        ui.selectable_value(&mut self.group, group.clone(), group);
                    }
                });
            ui.label("Buscar");
            ui.add(
                egui::TextEdit::singleline(&mut self.search)
                    .hint_text("ESR, frequência, Q...")
                    .desired_width(180.0),
            );
            if !self.search.is_empty() && ui.small_button("Limpar").clicked() {
                self.search.clear();
            }
        });
        let query = self.search.to_lowercase();
        let visible = |path: &str, label: &Label| {
            if query.is_empty() {
                label.group == self.group
                    || (path.starts_with("cy_e") && matches!(self.group.as_str(), "Cy1" | "Cy2"))
                    || (path == "extra.l2_l3_k" && matches!(self.group.as_str(), "L2" | "L3"))
            } else {
                format!("{} {} {}", label.group, label.label, path)
                    .to_lowercase()
                    .contains(&query)
            }
        };
        let mut active = fields
            .iter()
            .filter(|(k, m)| visible(k, m) && applicable(k) && m.kind != "Cadastro")
            .collect::<Vec<_>>();
        active.sort_by_key(|(path, _)| {
            let primary = path.ends_with("_uf")
                || path.ends_with("_nf")
                || path.ends_with("_uh")
                || path.ends_with("_mh")
                || matches!(path.as_str(), "rd_ohm" | "r1_kohm" | "r2_kohm");
            (!primary, path.as_str())
        });
        let catalog = fields
            .iter()
            .filter(|(k, m)| visible(k, m) && m.kind == "Cadastro")
            .collect::<Vec<_>>();
        let column_count = if ui.available_width() < 600.0 { 1 } else { 3 };
        egui::ScrollArea::vertical().id_salt("all-component-parameters").max_height(182.0).auto_shrink([false,true]).show(ui,|ui| {
            ui.columns(column_count,|cols| {
                for (i,(path,meta)) in active.iter().enumerate() {
                    cols[i%column_count].push_id(path,|ui| {changed|=field(ui,path,meta,&mut value);});
                }
            });
            if !catalog.is_empty() {
                egui::CollapsingHeader::new("Medições e limites do componente (cadastro)").id_salt(&self.group).show(ui,|ui| {
                    ui.label(RichText::new("Medições avulsas e limites ficam registrados. A curva usa os valores efetivos acima ou a tabela Z(f) ativada abaixo.").small());
                    ui.columns(column_count,|cols| {for(i,(path,meta))in catalog.iter().enumerate(){cols[i%column_count].push_id(path,|ui|{changed|=field(ui,path,meta,&mut value);});}});
                });
            }
            if self.group.starts_with('C') || self.group=="Bypass" || matches!(self.group.as_str(),"Rd"|"R1"|"R2") {
                ui.label(RichText::new("α=β=0: componente linear ideal. Coeficientes C(V)/R(V) não nulos podem ser salvos, mas bloqueiam o cálculo linear.").small());
            }
        });
        if changed {
            *p = serde_json::from_value(value).map_err(|e| format!("Campo inválido: {e}"))?;
        }
        if let Some(c) = p.extra.components.get_mut(&self.group) {
            egui::CollapsingHeader::new("Impedância medida Z(f) e observações").id_salt(format!("z-{}",self.group)).show(ui,|ui| {
                ui.label(RichText::new("Re/Im interpolados em log(f), apenas dentro da faixa medida. Substitui RLC e Cp deste componente; Rd continua externo a Cx2. Uma tabela escalar não modela L1 acoplado. A interpolação não garante um modelo causal de transientes.").small());
                ui.checkbox(&mut c.use_z_curve,"Usar Z(f) no cálculo");
                egui::ScrollArea::vertical().id_salt("z-table").max_height(130.0).show(ui,|ui|{
                    let mut remove=None;
                    egui::Grid::new("z-values").striped(true).show(ui,|ui| {
                        ui.label("f / Hz");ui.label("Re Z / Ω");ui.label("Im Z / Ω");ui.end_row();
                        for(i,point)in c.z_curve.iter_mut().enumerate(){
                            ui.push_id(i,|ui|{number(ui,&mut point.frequency_hz);number(ui,&mut point.re_ohm);number(ui,&mut point.im_ohm);if ui.small_button("×").clicked(){remove=Some(i);}ui.end_row();});
                        }
                    });
                    if let Some(i)=remove{c.z_curve.remove(i);}
                });
                if ui.small_button("+ Ponto medido").clicked(){let f=c.z_curve.last().map_or(1000.0,|p|p.frequency_hz*10.0);c.z_curve.push(ZPoint{frequency_hz:f,re_ohm:0.0,im_ohm:0.0});}
                ui.add(egui::TextEdit::multiline(&mut c.notes).hint_text("Ligação, instrumento, classe de segurança e observações").desired_rows(2).desired_width(f32::INFINITY));
            });
        }
        Ok(())
    }
}
fn flatten(prefix: &str, value: &Value, out: &mut Vec<String>) {
    match value {
        Value::Object(map) => {
            for (k, v) in map {
                flatten(
                    &if prefix.is_empty() {
                        k.clone()
                    } else {
                        format!("{prefix}.{k}")
                    },
                    v,
                    out,
                );
            }
        }
        Value::Array(_) => {}
        _ => out.push(prefix.into()),
    }
}
fn get_mut<'a>(v: &'a mut Value, path: &str) -> &'a mut Value {
    let mut cursor = v;
    for key in path.split('.') {
        cursor = &mut cursor[key];
    }
    cursor
}
fn field(ui: &mut egui::Ui, path: &str, meta: &Label, data: &mut Value) -> bool {
    let value = get_mut(data, path);
    if value.is_string() {
        return false;
    } // Observações editadas junto à tabela Z(f).
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(&meta.label)
            .on_hover_text(format!("{}\n{} · {}", meta.note, meta.kind, path));
        match value {
            Value::Bool(v) => {
                changed |= ui.checkbox(v, "").changed();
            }
            _ => {
                if value.is_null() {
                    if ui
                        .small_button("Definir")
                        .on_hover_text("Ainda não medido/informado")
                        .clicked()
                    {
                        *value = Value::from(0.0);
                        changed = true;
                    }
                } else if let Some(mut number_value) = value.as_f64() {
                    changed |= number(ui, &mut number_value);
                    if changed && number_value.is_finite() {
                        *value = if path.ends_with("servo_count") || path == "sweep_points" {
                            Value::from(number_value.max(0.0).round() as u64)
                        } else {
                            Value::from(number_value)
                        };
                    }
                    if ((meta.kind == "Cadastro" && path != "extra.servo_count")
                        || path.ends_with("ringing_duration_s")
                        || path.ends_with("ringing_decay_s"))
                        && ui
                            .small_button("×")
                            .on_hover_text("Marcar como não informado")
                            .clicked()
                    {
                        *value = Value::Null;
                        changed = true;
                    }
                }
                if !meta.unit.is_empty() {
                    ui.small(&meta.unit);
                }
            }
        }
    });
    changed
}
fn number(ui: &mut egui::Ui, value: &mut f64) -> bool {
    let speed = if *value == 0.0 {
        0.001
    } else {
        value.abs() * 0.01
    };
    ui.add(
        egui::DragValue::new(value)
            .speed(speed)
            .custom_formatter(|v, _| {
                if v != 0.0 && (v.abs() < 0.00001 || v.abs() >= 1e7) {
                    format!("{v:.7e}")
                } else {
                    format!("{v:.9}")
                        .trim_end_matches('0')
                        .trim_end_matches('.')
                        .to_owned()
                }
            })
            .custom_parser(|s| s.trim().replace(',', ".").parse().ok()),
    )
    .changed()
}
fn extra_label(path: &str) -> Label {
    let key = path.rsplit('.').next().unwrap_or(path);
    let (group, label, unit, note, kind) = if path.starts_with("extra.components.") {
        let component = path.split('.').nth(2).unwrap_or("Componente");
        let (label, unit, active) = match key {
            "leakage_mohm" => ("R de fuga (0 = infinito)", "MΩ", true),
            "series_l_nh" => ("L série do resistor", "nH", true),
            "parallel_c_pf" => ("C paralelo do resistor", "pF", true),
            "alpha_per_v" => ("α de C(V) / R(V)", "1/V", true),
            "beta_per_v2" => ("β de C(V) / R(V)", "1/V²", true),
            "tolerance_percent" => ("Tolerância", "%", false),
            "rated_v_ac" => ("Tensão nominal AC", "V RMS", false),
            "rated_v_dc" => ("Tensão nominal DC", "V", false),
            "rated_rms_a" => ("Corrente nominal RMS", "A", false),
            "rated_peak_a" => ("Corrente nominal de pico", "A", false),
            "saturation_a" => ("Corrente de saturação", "A", false),
            "rated_power_w" => ("Potência nominal", "W", false),
            "temperature_c" => ("Temperatura", "°C", false),
            "tcr_ppm_k" => ("Coeficiente térmico", "ppm/K", false),
            "measurement_hz" => ("Frequência de medição", "Hz", false),
            "measured_q" => ("Q medido (cadastro)", "", false),
            "measured_z_ohm" => ("|Z| medido (cadastro)", "Ω", false),
            "measured_dcr_mohm" => ("DCR medida (cadastro)", "mΩ", false),
            "lcr_equivalent_c_uf" => ("C equivalente do LCR", "µF", false),
            "self_resonance_hz" => ("Autorressonância medida", "Hz", false),
            "test_amplitude_v" => ("Amplitude do ensaio", "V", false),
            "bias_v" => ("Polarização de tensão", "V", false),
            "bias_a" => ("Polarização de corrente", "A", false),
            "use_z_curve" => ("Usar tabela Z(f)", "", true),
            "notes" => ("Observações", "", false),
            _ => (key, "", false),
        };
        (
            component,
            label,
            unit,
            "Leituras e limites são cadastro. Use os parâmetros efetivos para alterar a curva.",
            if active {
                "Ideal / efetivo"
            } else {
                "Cadastro"
            },
        )
    } else {
        let (label, unit, group, note) = match key {
            "servo_count" => (
                "Quantidade de servos",
                "",
                "Alimentação",
                "Cadastro: fonte agregada; não multiplica automaticamente a amplitude por servo.",
            ),
            "noise_duty_percent" => (
                "Duty das quadradas",
                "%",
                "Ruído",
                "Componente AC da série; média DC omitida.",
            ),
            "l2_l3_k" => (
                "Acoplamento L2 ↔ L3",
                "",
                "L2 / L3",
                "M=k√(L2L3), sinal conforme orientação. Zero = independentes.",
            ),
            "l1_interwinding_pf" => (
                "C entre enrolamentos",
                "pF",
                "L1",
                "Aproximação entre potenciais médios dos enrolamentos.",
            ),
            "l1_core_pe_pf" => (
                "C total enrolamentos–PE",
                "pF",
                "L1",
                "Núcleo equipotencial ao PE; metade da capacitância total por enrolamento médio.",
            ),
            "derivative_weights_enabled" => (
                "Ativar soma ponderada",
                "",
                "Ruído",
                "W(jω)=c0+jωc1−ω²c2. Desativado mantém a quadrada original.",
            ),
            "derivative_c0" => (
                "Peso da onda c0",
                "",
                "Ruído",
                "Valor efetivo inicial 1; não é medição.",
            ),
            "derivative_c1_s" => ("Peso de dx/dt", "s", "Ruído", "Valor efetivo inicial zero."),
            "derivative_c2_s2" => (
                "Peso de d²x/dt²",
                "s²",
                "Ruído",
                "Valor efetivo inicial zero.",
            ),
            "ringing_enabled" => (
                "Usar pacotes de ringing",
                "",
                "Ringing",
                "Substitui a quadrada por seno amortecido periódico. Não sincroniza com 60 Hz.",
            ),
            "ringing_frequency_hz" => (
                "Frequência do ringing",
                "Hz",
                "Ringing",
                "Hipótese de exemplo: 50 kHz.",
            ),
            "ringing_interval_s" => (
                "Intervalo entre pacotes",
                "s",
                "Ringing",
                "Hipótese de exemplo: 0,1 ms.",
            ),
            "ringing_relative_peak" => (
                "Escala relativa",
                "×",
                "Ringing",
                "Multiplica A·exp(−t/τ)·sin(ωct); máximo efetivo pode ser menor.",
            ),
            "ringing_duration_s" => (
                "Duração do pacote",
                "s",
                "Ringing",
                "Ainda não informada; necessária ao ativar ringing.",
            ),
            "ringing_decay_s" => (
                "Constante de decaimento τ",
                "s",
                "Ringing",
                "Ainda não informada; necessária ao ativar ringing.",
            ),
            _ => (key, "", "Outros", ""),
        };
        (
            group,
            label,
            unit,
            note,
            if key == "servo_count" {
                "Cadastro"
            } else {
                "Hipótese / efetivo"
            },
        )
    };
    Label {
        group: group.into(),
        label: label.into(),
        unit: unit.into(),
        note: note.into(),
        kind: kind.into(),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_parameter_has_a_label_and_remains_editable_after_serialization() {
        let p = Parameters::default();
        let value = serde_json::to_value(&p).unwrap();
        let mut keys = Vec::new();
        flatten("", &value, &mut keys);
        let editor = Editor::new();
        for key in keys {
            let label = editor
                .labels
                .get(&key)
                .cloned()
                .unwrap_or_else(|| extra_label(&key));
            assert_ne!(
                label.label,
                key.rsplit('.').next().unwrap(),
                "Falta rótulo: {key}"
            );
            assert_ne!(label.group, "Outros", "Falta grupo: {key}");
        }
        assert_eq!(editor.labels.len(), 53);
    }
}

fn applicable(path: &str) -> bool {
    if !path.starts_with("extra.components.") {
        return true;
    }
    let component = path.split('.').nth(2).unwrap_or("");
    let key = path.rsplit('.').next().unwrap_or("");
    let resistor = matches!(component, "Rd" | "R1" | "R2");
    let cap = component.starts_with('C') || component == "Bypass";
    match key {
        "leakage_mohm" => cap,
        "series_l_nh" | "parallel_c_pf" => resistor,
        "alpha_per_v" | "beta_per_v2" => cap || resistor,
        "use_z_curve" => false,
        _ => true,
    }
}
