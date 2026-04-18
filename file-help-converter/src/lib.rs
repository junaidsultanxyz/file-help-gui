pub mod error;

use crate::error::ConversionError;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

/// Checks if an input extension can be converted to the target format.
pub fn can_convert(input_ext: &str, target_format: &str) -> bool {
    let input_ext_lower = input_ext.to_lowercase();
    let target_format_lower = target_format.to_lowercase();

    if input_ext_lower == target_format_lower {
        return true;
    }

    match target_format_lower.as_str() {
        "pdf" => matches!(
            input_ext_lower.as_str(),
            "docx" | "pptx" | "xlsx" | "md"
        ),
        "txt" => matches!(
            input_ext_lower.as_str(),
            "pdf" | "docx" | "odt" | "pptx"
        ),
        "png" | "jpg" | "jpeg" | "webp" => matches!(
            input_ext_lower.as_str(),
            "png" | "jpg" | "jpeg" | "webp"
        ),
        _ => false,
    }
}

/// Recursively scans a directory for files that can be converted to `target_format`.
/// Returns a deduplicated, alphabetically sorted list of file extensions.
pub fn scan_convertible_extensions(
    source_dir: &Path,
    target_format: &str,
) -> Result<Vec<String>, ConversionError> {
    let mut extensions = HashSet::new();

    fn visit_dirs(
        dir: &Path,
        target_format: &str,
        extensions: &mut HashSet<String>,
    ) -> Result<(), ConversionError> {
        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    visit_dirs(&path, target_format, extensions)?;
                } else if path.is_file() {
                    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                        if can_convert(ext, target_format) {
                            extensions.insert(ext.to_lowercase());
                        }
                    }
                }
            }
        }
        Ok(())
    }

    visit_dirs(source_dir, target_format, &mut extensions)?;

    let mut result: Vec<String> = extensions.into_iter().collect();
    result.sort();

    Ok(result)
}

/// Processes the provided input paths (both files and folders), finds files matching allowed extensions, and
/// converts them into `target_dir`. Returns the number of files successfully converted.
pub fn batch_process_paths(
    input_paths: Vec<PathBuf>,
    target_dir: &Path,
    target_format: &str,
    allowed_input_extensions: &[String],
    remove_duplicates: bool,
    progress_callback: impl Fn(usize, usize, &str) + Send,
) -> Result<(usize, Vec<String>), ConversionError> {
    let mut count: usize = 0;
    let mut error_log: Vec<String> = Vec::new();

    std::fs::create_dir_all(target_dir)?;

    let mut paths_to_convert: Vec<PathBuf> = Vec::new();

    fn collect_paths(
        dir: &Path,
        allowed_input_extensions: &[String],
        target_format: &str,
        paths: &mut Vec<PathBuf>,
    ) -> Result<(), ConversionError> {
        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    collect_paths(&path, allowed_input_extensions, target_format, paths)?;
                } else if path.is_file() {
                    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                        let ext_lower = ext.to_lowercase();
                        if allowed_input_extensions.contains(&ext_lower)
                            && can_convert(ext, target_format)
                        {
                            paths.push(path);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    for input_path in input_paths {
        if input_path.is_dir() {
            if let Err(e) = collect_paths(&input_path, allowed_input_extensions, target_format, &mut paths_to_convert) {
                 error_log.push(format!("Failed to collect paths from [{}]: {}", input_path.display(), e));
            }
        } else if input_path.is_file() {
            if let Some(ext) = input_path.extension().and_then(|e| e.to_str()) {
                let ext_lower = ext.to_lowercase();
                if allowed_input_extensions.contains(&ext_lower)
                    && can_convert(ext, target_format)
                {
                    paths_to_convert.push(input_path);
                }
            }
        }
    }

    let total_files = paths_to_convert.len();

    for (i, path) in paths_to_convert.into_iter().enumerate() {
        let file_name_str = path.file_name().unwrap_or_default().to_string_lossy();
        progress_callback(i + 1, total_files, &file_name_str);

        let file_stem = match path.file_stem() {
            Some(stem) => stem,
            None => continue,
        };
        let file_stem_str = file_stem.to_string_lossy();
        let source_ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        let mut target_path = target_dir.join(file_stem).with_extension(target_format);

        if source_ext == target_format {
            if remove_duplicates && target_path.exists() {
                continue;
            }
            if let Err(e) = std::fs::copy(&path, &target_path) {
                error_log.push(format!("Failed [{}]: {}", file_name_str, e));
            } else {
                count += 1;
            }
            continue;
        }

        if remove_duplicates {
            if target_path.exists() {
                continue;
            }
        } else {
            let mut new_name = format!("converted_from_{}_{}", source_ext, file_stem_str);
            target_path = target_dir.join(&new_name).with_extension(target_format);

            let mut counter = 1;
            while target_path.exists() {
                new_name = format!(
                    "converted_from_{}_{}_{}",
                    source_ext, file_stem_str, counter
                );
                target_path = target_dir.join(&new_name).with_extension(target_format);
                counter += 1;
            }
        }

        // Pipeline verification
        if let Err(e) = fs::read(&path) {
            error_log.push(format!("Failed [{}]: {}", file_name_str, e));
            continue;
        }

        match target_format {
            "pdf" => match source_ext.as_str() {
                "md" => {
                    let md_text = match std::fs::read_to_string(&path) {
                        Ok(text) => text,
                        Err(e) => {
                            error_log.push(format!("Failed [{}]: {}", file_name_str, e));
                            continue;
                        }
                    };
                    if let Err(e) = markdown2pdf::parse_into_file(
                        md_text,
                        target_path.to_str().unwrap(),
                        markdown2pdf::config::ConfigSource::Default,
                        None,
                    ) {
                        error_log.push(format!("Failed [{}]: {}", file_name_str, e));
                        continue;
                    }
                }
                _ => {
                    let result = match office2pdf::convert(&path) {
                        Ok(r) => r,
                        Err(e) => {
                            error_log.push(format!("Failed [{}]: {}", file_name_str, e));
                            continue;
                        }
                    };
                    if let Err(e) = fs::write(&target_path, &result.pdf) {
                        error_log.push(format!("Failed [{}]: {}", file_name_str, e));
                        continue;
                    }
                }
            },
            "txt" => {
                use dotext::doc::OpenOfficeDoc;
                use dotext::MsDoc;
                use std::io::Read;
                let mut text = String::new();
                match source_ext.as_str() {
                    "pdf" => match pdf_extract::extract_text(&path) {
                        Ok(t) => text = t,
                        Err(e) => {
                            error_log.push(format!("Failed [{}]: {}", file_name_str, e));
                            continue;
                        }
                    },
                    "docx" => match dotext::Docx::open(&path) {
                        Ok(mut doc) => {
                            if let Err(e) = doc.read_to_string(&mut text) {
                                error_log.push(format!("Failed [{}]: {}", file_name_str, e));
                                continue;
                            }
                        }
                        Err(e) => {
                            error_log.push(format!("Failed [{}]: {}", file_name_str, e));
                            continue;
                        }
                    },
                    "odt" => match dotext::Odt::open(&path) {
                        Ok(mut doc) => {
                            if let Err(e) = doc.read_to_string(&mut text) {
                                error_log.push(format!("Failed [{}]: {}", file_name_str, e));
                                continue;
                            }
                        }
                        Err(e) => {
                            error_log.push(format!("Failed [{}]: {}", file_name_str, e));
                            continue;
                        }
                    },
                    "pptx" => match dotext::Pptx::open(&path) {
                        Ok(mut doc) => {
                            if let Err(e) = doc.read_to_string(&mut text) {
                                error_log.push(format!("Failed [{}]: {}", file_name_str, e));
                                continue;
                            }
                        }
                        Err(e) => {
                            error_log.push(format!("Failed [{}]: {}", file_name_str, e));
                            continue;
                        }
                    },
                    _ => {
                        error_log.push(format!("Failed [{}]: Unsupported txt extraction source", file_name_str));
                        continue;
                    }
                }
                if let Err(e) = fs::write(&target_path, text) {
                    error_log.push(format!("Failed [{}]: {}", file_name_str, e));
                    continue;
                }
            }
            "png" | "jpg" | "jpeg" | "webp" => {
                let img = match image::open(&path) {
                    Ok(i) => i,
                    Err(e) => {
                        error_log.push(format!("Failed [{}]: Image open error: {}", file_name_str, e));
                        continue;
                    }
                };
                if let Err(e) = img.save(&target_path) {
                    error_log.push(format!("Failed [{}]: Image save error: {}", file_name_str, e));
                    continue;
                }
            }
            _ => {
                error_log.push(format!("Failed [{}]: Unsupported target format", file_name_str));
                continue;
            }
        }

        count += 1;
    }

    Ok((count, error_log))
}
