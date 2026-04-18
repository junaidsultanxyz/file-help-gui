use regex::Regex;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

/// Extracts materials matching allowed file extensions (regex patterns) from `source_dir`
/// into `target_dir` using zero-cost abstractions and recursive traversal.
/// Returns the total number of bytes copied.
pub fn extract_materials(
    source_dir: &Path,
    target_dir: &Path,
    allowed_extensions: &[&str],
) -> Result<u64, crate::error::ExtractionError> {
    // Ensure target directory exists
    if !target_dir.exists() {
        fs::create_dir_all(target_dir)?;
    }

    // Precompile regexes to avoid recompiling on each file
    let mut compiled_regexes = Vec::with_capacity(allowed_extensions.len());
    for ext in allowed_extensions {
        let re = Regex::new(ext)?;
        compiled_regexes.push(re);
    }

    extract_recursive(source_dir, target_dir, &compiled_regexes)
}

/// Recursively iterates over the directory content and copies matching files
fn extract_recursive(
    current_dir: &Path,
    target_dir: &Path,
    regexes: &[Regex],
) -> Result<u64, crate::error::ExtractionError> {
    let mut total_bytes = 0;

    for entry in fs::read_dir(current_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            total_bytes += extract_recursive(&path, target_dir, regexes)?;
        } else if path.is_file() {
            if let Some(file_name) = path.file_name() {
                let file_name_str = file_name.to_string_lossy();
                
                if regexes.iter().any(|re| re.is_match(&file_name_str)) {
                    let dest_path = target_dir.join(file_name);
                    total_bytes += fs::copy(&path, &dest_path)?;
                }
            }
        }
    }

    Ok(total_bytes)
}

/// Scans the source_dir recursively and returns a unique, alphabetically sorted
/// list of file extensions found, converted to lowercase.
pub fn scan_unique_extensions(source_dir: &Path) -> Result<Vec<String>, crate::error::ExtractionError> {
    let mut extensions = BTreeSet::new();
    scan_recursive(source_dir, &mut extensions)?;
    Ok(extensions.into_iter().collect())
}

fn scan_recursive(
    current_dir: &Path,
    extensions: &mut BTreeSet<String>,
) -> Result<(), crate::error::ExtractionError> {
    for entry in fs::read_dir(current_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            scan_recursive(&path, extensions)?;
        } else if path.is_file() {
            if let Some(ext) = path.extension() {
                extensions.insert(ext.to_string_lossy().to_lowercase());
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_extract_materials() {
        // Prepare temporary setup
        let temp_dir = env::temp_dir().join("extract_materials_test_dir");
        let source_dir = temp_dir.join("source");
        let target_dir = temp_dir.join("target");

        // Clean up from any potential previous test run
        let _ = fs::remove_dir_all(&temp_dir);

        // Set up directories
        fs::create_dir_all(&source_dir).expect("Failed to create source dir");
        let nested_dir = source_dir.join("nested");
        fs::create_dir_all(&nested_dir).expect("Failed to create nested dir");

        // Create dummy files
        let valid_files = vec![
            source_dir.join("test_note.pdf"),
            nested_dir.join("assignment.docx"),
        ];

        let invalid_files = vec![
            source_dir.join("image.png"),
            nested_dir.join("notes.txt"),
            nested_dir.join("fake_pdf.pdf.tmp"),
        ];

        let content = b"dummy file content";
        let content_len = content.len() as u64;
        
        for path in valid_files.iter().chain(invalid_files.iter()) {
            let mut file = File::create(path).expect("Failed to create dummy file");
            file.write_all(content).expect("Failed to write to dummy file");
        }

        let allowed_extensions = vec![r"\.pdf$", r"\.docx$"];

        let copied_bytes = extract_materials(&source_dir, &target_dir, &allowed_extensions)
            .expect("Failed to execute extract_materials");

        // Validate expectations
        assert_eq!(copied_bytes, valid_files.len() as u64 * content_len);
        assert!(target_dir.join("test_note.pdf").exists());
        assert!(target_dir.join("assignment.docx").exists());
        assert!(!target_dir.join("image.png").exists());
        assert!(!target_dir.join("notes.txt").exists());
        assert!(!target_dir.join("fake_pdf.pdf.tmp").exists());

        // Clean up afterwards
        fs::remove_dir_all(&temp_dir).expect("Failed to clean up test directory");
    }

    #[test]
    fn test_scan_unique_extensions() {
        let temp_dir = env::temp_dir().join("scan_unique_extensions_test_dir");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).expect("Failed to create test setup dir");

        // Create dummy files with extensions
        let files = vec!["doc.pdf", "doc2.PDF", "word.docx", "text.txt", "no_extension"];
        for file in files {
            File::create(temp_dir.join(file)).expect("Failed to create test file");
        }

        let extensions = scan_unique_extensions(&temp_dir).expect("Failed to scan directory");
        assert_eq!(extensions, vec!["docx".to_string(), "pdf".to_string(), "txt".to_string()]);

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
