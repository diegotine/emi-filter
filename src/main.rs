#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod engine;
mod history;
mod parameter_editor;
mod storage;
mod ui;

fn main() -> eframe::Result {
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(String::as_str) == Some("--version") {
        println!("EMI Filter {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    if args.get(1).map(String::as_str) == Some("--write-defaults") {
        let path = args
            .get(2)
            .map(String::as_str)
            .unwrap_or("modelo_usuario.json");
        let result = serde_json::to_string_pretty(&engine::Parameters::default())
            .map_err(|error| error.to_string())
            .and_then(|json| std::fs::write(path, json).map_err(|error| error.to_string()));
        if let Err(error) = result {
            eprintln!("Não foi possível salvar {path}: {error}");
            std::process::exit(1);
        }
        return Ok(());
    }
    if matches!(
        args.get(1).map(String::as_str),
        Some("--calculate" | "--transfer" | "--waveforms")
    ) {
        let result = (|| -> Result<(), String> {
            let path = args
                .get(2)
                .ok_or("Use --calculate, --transfer ou --waveforms parametros.json saida.csv")?;
            let destination = args.get(3).ok_or("Informe o arquivo CSV de destino")?;
            let data = std::fs::read_to_string(path)
                .map_err(|error| format!("Não foi possível ler {path}: {error}"))?;
            let parameters = storage::read_document(&data)?.parameters;
            let result = engine::calculate(&parameters)?;
            let csv = match args[1].as_str() {
                "--transfer" => engine::export_transfer_csv(&result),
                "--waveforms" => engine::export_waveform_csv(&result)?,
                _ => engine::export_csv(&result),
            };
            std::fs::write(destination, csv)
                .map_err(|error| format!("Não foi possível salvar {destination}: {error}"))?;
            Ok(())
        })();
        if let Err(error) = result {
            eprintln!("{error}");
            std::process::exit(1);
        }
        return Ok(());
    }
    if std::env::args().any(|a| a == "--calculate-defaults") {
        match engine::calculate(&engine::Parameters::default()) {
            Ok(result) => {
                for row in &result.integrated {
                    println!(
                        "derivada={} conjunta={:.8} dB DM={:.8} dB CM={:.8} dB",
                        row.order, row.combined_db, row.dm_db, row.cm_db
                    );
                }
            }
            Err(message) => {
                eprintln!("{message}");
                std::process::exit(1);
            }
        }
        return Ok(());
    }
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title(format!(
                "EMI Filter v{} -- Made with Codex",
                env!("CARGO_PKG_VERSION")
            ))
            .with_inner_size([1380.0, 900.0])
            .with_min_inner_size([1024.0, 700.0]),
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };
    eframe::run_native(
        "EMI Filter",
        options,
        Box::new(|cc| Ok(Box::new(ui::EmiApp::new(cc)))),
    )
}
