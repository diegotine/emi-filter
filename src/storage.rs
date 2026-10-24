use crate::engine::Parameters;
use crate::history::GraphRecord;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub const MODEL_ID: &str = "emi-two-conductor-linear-v2";
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Document {
    pub application: String,
    pub app_version: String,
    pub model_id: String,
    pub schema_version: u32,
    pub saved_at: String,
    pub parameters: Parameters,
    #[serde(default)]
    pub graphs: Vec<GraphRecord>,
    #[serde(default)]
    pub selected_graph: usize,
    #[serde(default)]
    pub overlay: bool,
}
impl Document {
    pub fn new(parameters: Parameters) -> Self {
        Self {
            application: "EMI Filter — Made with Codex".into(),
            app_version: env!("CARGO_PKG_VERSION").into(),
            model_id: MODEL_ID.into(),
            schema_version: 2,
            saved_at: timestamp(),
            parameters,
            graphs: Vec::new(),
            selected_graph: 0,
            overlay: false,
        }
    }
}
pub fn read_document(text: &str) -> Result<Document, String> {
    let value: serde_json::Value =
        serde_json::from_str(text).map_err(|e| format!("JSON inválido: {e}"))?;
    if value.get("parameters").is_none() {
        let p = serde_json::from_value(value)
            .map_err(|e| format!("Configuração antiga incompatível: {e}"))?;
        return Ok(Document::new(p));
    }
    let doc: Document =
        serde_json::from_value(value).map_err(|e| format!("Configuração incompatível: {e}"))?;
    if doc.model_id != MODEL_ID || doc.schema_version != 2 {
        return Err(format!(
            "Modelo/schema incompatível: {} / {}. Este app usa {} / 2.",
            doc.model_id, doc.schema_version, MODEL_ID
        ));
    }
    if doc.graphs.len() > 3 || (!doc.graphs.is_empty() && doc.selected_graph >= doc.graphs.len()) {
        return Err("Histórico de gráficos inválido.".into());
    }
    Ok(doc)
}
pub fn defaults_path() -> PathBuf {
    if let Some(path) = std::env::var_os("EMI_DEFAULTS_PATH") {
        return PathBuf::from(path);
    }
    let base = if cfg!(target_os = "windows") {
        std::env::var_os("APPDATA").map(PathBuf::from)
    } else {
        std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|p| PathBuf::from(p).join(".config")))
    };
    base.unwrap_or_else(|| PathBuf::from("."))
        .join("emi-filter")
        .join("padroes.json")
}
pub fn save(path: &Path, doc: &Document) -> Result<(), String> {
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let data = serde_json::to_string_pretty(doc).map_err(|e| e.to_string())?;
    std::fs::write(path, data).map_err(|e| e.to_string())
}
pub fn timestamp() -> String {
    let elapsed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format_timestamp(elapsed.as_secs() as i64).replace(
        " UTC−03",
        &format!(".{:03} UTC−03", elapsed.subsec_millis()),
    )
}
/// Horário do projeto (São Paulo, UTC−03:00), explícito inclusive no Windows.
fn format_timestamp(unix: i64) -> String {
    let local = unix - 3 * 3600;
    let days = local.div_euclid(86400);
    let t = local.rem_euclid(86400);
    // Conversão civil do calendário gregoriano, independente do fuso do SO.
    let z = days + 719468;
    let era = z.div_euclid(146097);
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let mut y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    y += i64::from(month <= 2);
    format!(
        "{y:04}-{month:02}-{day:02} {:02}:{:02}:{:02} UTC−03",
        t / 3600,
        t / 60 % 60,
        t % 60
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn document_round_trip_preserves_all_fields_and_rejects_future_models() {
        let mut p = Parameters::default();
        p.extra.l2_l3_k = 0.3;
        p.extra.components.get_mut("Cx2").unwrap().alpha_per_v = 1e-5;
        let doc = Document::new(p.clone());
        let json = serde_json::to_string(&doc).unwrap();
        assert_eq!(read_document(&json).unwrap().parameters, p);
        assert_eq!(
            read_document(&serde_json::to_string(&p).unwrap())
                .unwrap()
                .parameters,
            p
        );
        assert!(read_document(&json.replace(MODEL_ID, "future-model")).is_err());
        assert!(json.contains("Made with Codex"));
    }
    #[test]
    fn timestamp_handles_local_midnight_and_leap_day() {
        assert_eq!(format_timestamp(0), "1969-12-31 21:00:00 UTC−03");
        assert_eq!(format_timestamp(951782400), "2000-02-28 21:00:00 UTC−03");
        assert_eq!(format_timestamp(951793200), "2000-02-29 00:00:00 UTC−03");
    }
}
