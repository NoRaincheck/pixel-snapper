use pixel_snapper::*;
use std::env;
use std::path::{Path, PathBuf};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: pixel-snapper <input> <output> [k_colors] [--pixel-size <size>]");
        std::process::exit(1);
    }

    let input_path = PathBuf::from(&args[1]);
    let output_path = PathBuf::from(&args[2]);

    let mut k_colors: usize = 16;
    let mut pixel_size_override: Option<f64> = None;

    let mut i = 3;
    while i < args.len() {
        match args[i].as_str() {
            "--pixel-size" => {
                let Some(val) = args.get(i + 1) else {
                    eprintln!("Warning: --pixel-size requires a value");
                    break;
                };
                match val.parse::<f64>() {
                    Ok(px) if px.is_finite() && px > 0.0 => pixel_size_override = Some(px),
                    _ => eprintln!("Warning: invalid --pixel-size '{}', ignoring", val),
                }
                i += 2;
            }
            arg if arg.starts_with("--") => {
                eprintln!("Warning: unknown argument '{}', ignoring", arg);
                i += 1;
            }
            k_arg => {
                match k_arg.parse::<usize>() {
                    Ok(k) if k > 0 => k_colors = k,
                    _ => eprintln!(
                        "Warning: invalid k_colors '{}', falling back to default ({})",
                        k_arg, k_colors
                    ),
                }
                i += 1;
            }
        }
    }

    let config = Config {
        k_colors,
        pixel_size_override,
        ..Default::default()
    };

    if input_path.is_dir() {
        process_batch_dir(&input_path, &output_path, &config)
    } else {
        process_single(&input_path, &output_path, &config)
    }
}

fn process_single(input_path: &Path, output_path: &Path, config: &Config) -> Result<()> {
    let processed = process_file(input_path, output_path, config)?;
    println!("Processing: {}", input_path.display());
    println!(
        "Pixel size: {:.1}px ({})",
        processed.pixel_size(),
        if processed.pixel_size_override() {
            "override"
        } else {
            "auto-detected"
        }
    );
    println!(
        "Output size: {}x{}",
        processed.output_width(),
        processed.output_height()
    );
    println!("Saved to: {}", output_path.display());
    Ok(())
}

fn process_batch_dir(input_dir: &Path, output_dir: &Path, config: &Config) -> Result<()> {
    let batch_config = BatchConfig {
        input_dir: input_dir.to_path_buf(),
        output_dir: output_dir.to_path_buf(),
        k_colors: config.k_colors,
        pixel_size_override: config.pixel_size_override,
    };

    process_batch_with_reporter(&batch_config, |event| match event {
        BatchEvent::BatchStarted { input_dir, total } => {
            println!(
                "Batch processing {} image{} from: {}",
                total,
                if total == 1 { "" } else { "s" },
                input_dir.display()
            );
        }
        BatchEvent::Started {
            input,
            index,
            total,
        } => {
            println!("Processing {}/{}: {}", index + 1, total, input.display());
        }
        BatchEvent::Finished {
            input,
            output,
            index,
            total,
        } => {
            println!(
                "Done {}/{}: {} -> {}",
                index + 1,
                total,
                input.display(),
                output.display()
            );
        }
        BatchEvent::Failed {
            input,
            output,
            error,
            index,
            total,
        } => {
            eprintln!(
                "Failed {}/{}: {} -> {} ({})",
                index + 1,
                total,
                input.display(),
                output.display(),
                error
            );
        }
        BatchEvent::BatchFinished { input_dir, total } => {
            println!(
                "Processed {} image{} in: {}",
                total,
                if total == 1 { "" } else { "s" },
                input_dir.display()
            );
        }
    })
}
