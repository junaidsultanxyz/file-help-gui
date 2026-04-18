slint::include_modules!();

use slint::{Model, SharedString};
use std::collections::HashSet;
use std::path::PathBuf;
use std::cell::RefCell;
use std::rc::Rc;

fn main() -> Result<(), slint::PlatformError> {
    unsafe {
        std::env::set_var("SLINT_STYLE", "fluent");
    }
    let ui = MainWindow::new()?;

    let extraction_inputs = Rc::new(RefCell::new(Vec::<PathBuf>::new()));
    let document_inputs = Rc::new(RefCell::new(Vec::<PathBuf>::new()));
    let image_inputs = Rc::new(RefCell::new(Vec::<PathBuf>::new()));

    // EXTRACTION
    let ui_handle = ui.as_weak();
    let inputs = Rc::clone(&extraction_inputs);
    ui.on_extraction_add_files(move || {
        if let Some(ui) = ui_handle.upgrade() {
            let ui_weak = ui.as_weak();
            let inputs_cloned = Rc::clone(&inputs);
            slint::spawn_local(async move {
                if let Some(files) = rfd::AsyncFileDialog::new().pick_files().await {
                    if let Some(ui) = ui_weak.upgrade() {
                        let mut vec = inputs_cloned.borrow_mut();
                        for f in files {
                            vec.push(f.path().to_path_buf());
                        }
                        ui.set_extraction_source_count(vec.len() as i32);
                        
                        let cloned_vec = vec.clone();
                        let ui_weak2 = ui.as_weak();
                        std::thread::spawn(move || {
                            let mut all_extensions = HashSet::new();
                            for path in cloned_vec {
                                if path.is_file() {
                                    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                                        all_extensions.insert(ext.to_lowercase());
                                    }
                                }
                            }
                            let mut extensions: Vec<String> = all_extensions.into_iter().collect();
                            extensions.sort();
                            
                            let _ = slint::invoke_from_event_loop(move || {
                                if let Some(ui) = ui_weak2.upgrade() {
                                    let items: Vec<ExtensionToggle> = extensions.into_iter().map(|ext| {
                                        ExtensionToggle { ext: SharedString::from(ext), selected: true }
                                    }).collect();
                                    let model = std::rc::Rc::new(slint::VecModel::from(items));
                                    ui.set_available_extensions(model.into());
                                }
                            });
                        });
                    }
                }
            }).unwrap();
        }
    });

    let ui_handle = ui.as_weak();
    let inputs = Rc::clone(&extraction_inputs);
    ui.on_extraction_add_folder(move || {
        if let Some(ui) = ui_handle.upgrade() {
            let ui_weak = ui.as_weak();
            let inputs_cloned = Rc::clone(&inputs);
            slint::spawn_local(async move {
                if let Some(folders) = rfd::AsyncFileDialog::new().pick_folders().await {
                    if let Some(ui) = ui_weak.upgrade() {
                        let mut vec = inputs_cloned.borrow_mut();
                        for f in folders {
                            vec.push(f.path().to_path_buf());
                        }
                        ui.set_extraction_source_count(vec.len() as i32);
                        
                        let cloned_vec = vec.clone();
                        let ui_weak2 = ui.as_weak();
                        std::thread::spawn(move || {
                            let mut all_extensions = HashSet::new();
                            for path in cloned_vec {
                                if path.is_dir() {
                                    if let Ok(exts) = app_core::extraction::scan_unique_extensions(&path) {
                                        all_extensions.extend(exts);
                                    }
                                }
                            }
                            let mut extensions: Vec<String> = all_extensions.into_iter().collect();
                            extensions.sort();
                            
                            let _ = slint::invoke_from_event_loop(move || {
                                if let Some(ui) = ui_weak2.upgrade() {
                                    let items: Vec<ExtensionToggle> = extensions.into_iter().map(|ext| {
                                        ExtensionToggle { ext: SharedString::from(ext), selected: true }
                                    }).collect();
                                    let model = std::rc::Rc::new(slint::VecModel::from(items));
                                    ui.set_available_extensions(model.into());
                                }
                            });
                        });
                    }
                }
            }).unwrap();
        }
    });

    let ui_handle = ui.as_weak();
    ui.on_select_target(move || {
        if let Some(ui) = ui_handle.upgrade() {
            let ui_weak = ui.as_weak();
            slint::spawn_local(async move {
                if let Some(folder) = rfd::AsyncFileDialog::new().pick_folder().await {
                    if let Some(ui) = ui_weak.upgrade() {
                        ui.set_target_dir(SharedString::from(folder.path().to_string_lossy().into_owned()));
                    }
                }
            }).unwrap();
        }
    });

    let ui_handle = ui.as_weak();
    let inputs = Rc::clone(&extraction_inputs);
    ui.on_reset_extraction_state(move || {
        if let Some(ui) = ui_handle.upgrade() {
            inputs.borrow_mut().clear();
            ui.set_extraction_source_count(0);
            ui.set_target_dir(SharedString::from(""));
            let empty_model: Vec<ExtensionToggle> = Vec::new();
            ui.set_available_extensions(std::rc::Rc::new(slint::VecModel::from(empty_model)).into());
            ui.set_status_message(SharedString::from("Ready"));
        }
    });

    let ui_handle = ui.as_weak();
    let inputs = Rc::clone(&extraction_inputs);
    ui.on_start_extraction(move || {
        if let Some(ui) = ui_handle.upgrade() {
            let target_dir = ui.get_target_dir().to_string();
            let model = ui.get_available_extensions();
            let mut selected_exts = Vec::new();
            for i in 0..model.row_count() {
                if let Some(item) = model.row_data(i) {
                    if item.selected { selected_exts.push(format!(r"\.{}$", item.ext)); }
                }
            }
            ui.set_status_message(SharedString::from("Extracting..."));

            let ui_weak = ui.as_weak();
            let inputs_to_process = inputs.borrow().clone();
            std::thread::spawn(move || {
                let tgt = std::path::Path::new(&target_dir);
                let extensions_refs: Vec<&str> = selected_exts.iter().map(|s| s.as_str()).collect();
                
                let mut total_bytes = 0;
                let mut extraction_error = None;
                
                for src in inputs_to_process {
                    // Extract material currently processes passing a single path (could be refactored inside app-core but we can just loop over them here for files or directories)
                    // app_core::extraction::extract_materials handles both files and directories?
                    // We'll iterate and accumulate.
                    if src.is_dir() || src.is_file() {
                        match app_core::extraction::extract_materials(&src, tgt, &extensions_refs) {
                            Ok(bytes) => total_bytes += bytes,
                            Err(e) => { extraction_error = Some(e); break; }
                        }
                    }
                }
                
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_weak.upgrade() {
                        if let Some(e) = extraction_error {
                            ui.set_status_message(SharedString::from(format!("Error: {}", e)));
                        } else {
                            ui.set_status_message(SharedString::from(format!("Success: Copied {} bytes", total_bytes)));
                            ui.invoke_show_extraction_success_popup();
                        }
                    }
                });
            });
        }
    });

    let ui_handle = ui.as_weak();
    ui.on_extraction_select_all(move || {
        if let Some(ui) = ui_handle.upgrade() {
            let model = ui.get_available_extensions();
            for i in 0..model.row_count() {
                if let Some(mut item) = model.row_data(i) { item.selected = true; model.set_row_data(i, item); }
            }
        }
    });

    let ui_handle = ui.as_weak();
    ui.on_extraction_deselect_all(move || {
        if let Some(ui) = ui_handle.upgrade() {
            let model = ui.get_available_extensions();
            for i in 0..model.row_count() {
                if let Some(mut item) = model.row_data(i) { item.selected = false; model.set_row_data(i, item); }
            }
        }
    });

    // DOCUMENT CONVERTER
    let ui_handle = ui.as_weak();
    let inputs = Rc::clone(&document_inputs);
    ui.on_document_add_files(move || {
        if let Some(ui) = ui_handle.upgrade() {
            let ui_weak = ui.as_weak();
            let inputs_cloned = Rc::clone(&inputs);
            slint::spawn_local(async move {
                if let Some(files) = rfd::AsyncFileDialog::new().pick_files().await {
                    if let Some(ui) = ui_weak.upgrade() {
                        let mut vec = inputs_cloned.borrow_mut();
                        for f in files { vec.push(f.path().to_path_buf()); }
                        ui.set_document_source_count(vec.len() as i32);
                        
                        let cloned_vec = vec.clone();
                        let target_format = ui.get_document_selected_target_format().to_string();
                        let ui_weak2 = ui.as_weak();
                        std::thread::spawn(move || {
                            let mut all_extensions = HashSet::new();
                            for path in cloned_vec {
                                if path.is_file() {
                                    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                                        if file_help_converter::can_convert(ext, &target_format) {
                                            all_extensions.insert(ext.to_lowercase());
                                        }
                                    }
                                }
                            }
                            let mut extensions: Vec<String> = all_extensions.into_iter().collect();
                            extensions.sort();
                            
                            let _ = slint::invoke_from_event_loop(move || {
                                if let Some(ui) = ui_weak2.upgrade() {
                                    let items: Vec<ExtensionToggle> = extensions.into_iter().map(|ext| {
                                        ExtensionToggle { ext: SharedString::from(ext), selected: true }
                                    }).collect();
                                    let model = std::rc::Rc::new(slint::VecModel::from(items));
                                    ui.set_document_available_input_extensions(model.into());
                                }
                            });
                        });
                    }
                }
            }).unwrap();
        }
    });

    let ui_handle = ui.as_weak();
    let inputs = Rc::clone(&document_inputs);
    ui.on_document_add_folder(move || {
        if let Some(ui) = ui_handle.upgrade() {
            let ui_weak = ui.as_weak();
            let inputs_cloned = Rc::clone(&inputs);
            slint::spawn_local(async move {
                if let Some(folders) = rfd::AsyncFileDialog::new().pick_folders().await {
                    if let Some(ui) = ui_weak.upgrade() {
                        let mut vec = inputs_cloned.borrow_mut();
                        for f in folders { vec.push(f.path().to_path_buf()); }
                        ui.set_document_source_count(vec.len() as i32);
                        
                        let cloned_vec = vec.clone();
                        let target_format = ui.get_document_selected_target_format().to_string();
                        let ui_weak2 = ui.as_weak();
                        std::thread::spawn(move || {
                            let mut all_extensions = HashSet::new();
                            for path in cloned_vec {
                                if path.is_dir() {
                                    if let Ok(exts) = file_help_converter::scan_convertible_extensions(&path, &target_format) {
                                        all_extensions.extend(exts);
                                    }
                                }
                            }
                            let mut extensions: Vec<String> = all_extensions.into_iter().collect();
                            extensions.sort();
                            
                            let _ = slint::invoke_from_event_loop(move || {
                                if let Some(ui) = ui_weak2.upgrade() {
                                    let items: Vec<ExtensionToggle> = extensions.into_iter().map(|ext| {
                                        ExtensionToggle { ext: SharedString::from(ext), selected: true }
                                    }).collect();
                                    let model = std::rc::Rc::new(slint::VecModel::from(items));
                                    ui.set_document_available_input_extensions(model.into());
                                }
                            });
                        });
                    }
                }
            }).unwrap();
        }
    });

    let ui_handle = ui.as_weak();
    ui.on_document_select_target(move || {
        if let Some(ui) = ui_handle.upgrade() {
            let ui_weak = ui.as_weak();
            slint::spawn_local(async move {
                if let Some(folder) = rfd::AsyncFileDialog::new().pick_folder().await {
                    if let Some(ui) = ui_weak.upgrade() {
                        ui.set_document_target_dir(SharedString::from(folder.path().to_string_lossy().into_owned()));
                    }
                }
            }).unwrap();
        }
    });

    let ui_handle = ui.as_weak();
    let inputs = Rc::clone(&document_inputs);
    ui.on_reset_document_state(move || {
        if let Some(ui) = ui_handle.upgrade() {
            inputs.borrow_mut().clear();
            ui.set_document_source_count(0);
            ui.set_document_target_dir(SharedString::from(""));
            let empty_model: Vec<ExtensionToggle> = Vec::new();
            ui.set_document_available_input_extensions(std::rc::Rc::new(slint::VecModel::from(empty_model)).into());
            ui.set_document_status_message(SharedString::from("Ready"));
            ui.set_document_selected_target_format(SharedString::from("pdf"));
        }
    });
    
    let ui_handle = ui.as_weak();
    ui.on_document_select_all(move || {
        if let Some(ui) = ui_handle.upgrade() {
            let model = ui.get_document_available_input_extensions();
            for i in 0..model.row_count() {
                if let Some(mut item) = model.row_data(i) { item.selected = true; model.set_row_data(i, item); }
            }
        }
    });

    let ui_handle = ui.as_weak();
    ui.on_document_deselect_all(move || {
        if let Some(ui) = ui_handle.upgrade() {
            let model = ui.get_document_available_input_extensions();
            for i in 0..model.row_count() {
                if let Some(mut item) = model.row_data(i) { item.selected = false; model.set_row_data(i, item); }
            }
        }
    });

    let ui_handle = ui.as_weak();
    let inputs = Rc::clone(&document_inputs);
    ui.on_start_document_conversion(move || {
        if let Some(ui) = ui_handle.upgrade() {
            let target_dir = PathBuf::from(ui.get_document_target_dir().to_string());
            let target_format = ui.get_document_selected_target_format().to_string();
            let remove_duplicates = ui.get_remove_duplicates();
            
            let model = ui.get_document_available_input_extensions();
            let mut allowed_input_extensions = Vec::new();
            for i in 0..model.row_count() {
                if let Some(item) = model.row_data(i) {
                    if item.selected { allowed_input_extensions.push(item.ext.to_string()); }
                }
            }

            ui.set_document_status_message(SharedString::from("Converting..."));
            let ui_weak = ui.as_weak();
            let ui_callback_weak = ui.as_weak();
            let inputs_to_process = inputs.borrow().clone();
            
            std::thread::spawn(move || {
                let progress_callback = move |current: usize, total: usize, file_name: &str| {
                    let file_name_owned = file_name.to_string();
                    let _ = slint::invoke_from_event_loop({
                        let weak = ui_callback_weak.clone();
                        move || {
                            if let Some(ui) = weak.upgrade() {
                                let progress = current as f32 / total as f32;
                                ui.set_progress_value(progress);
                                ui.set_status_text(SharedString::from(format!("Converting ({}/{}): {}", current, total, file_name_owned)));
                            }
                        }
                    });
                };

                let result = file_help_converter::batch_process_paths(
                    inputs_to_process,
                    &target_dir,
                    &target_format,
                    &allowed_input_extensions,
                    remove_duplicates,
                    progress_callback
                );

                let is_ok = result.is_ok();
                let (message, errors) = match result {
                    Ok((count, errs)) => (format!("Successfully converted {} files.", count), errs),
                    Err(e) => (format!("Error: {}", e), vec![]),
                };
                let error_log = errors.join("\n");

                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_weak.upgrade() {
                        ui.set_document_status_message(SharedString::from(message));
                        ui.set_converter_error_log(SharedString::from(error_log));
                        if is_ok { ui.invoke_show_document_success_popup(); }
                    }
                });
            });
        }
    });

    // IMAGE CONVERTER
    let ui_handle = ui.as_weak();
    let inputs = Rc::clone(&image_inputs);
    ui.on_image_add_files(move || {
        if let Some(ui) = ui_handle.upgrade() {
            let ui_weak = ui.as_weak();
            let inputs_cloned = Rc::clone(&inputs);
            slint::spawn_local(async move {
                if let Some(files) = rfd::AsyncFileDialog::new().pick_files().await {
                    if let Some(ui) = ui_weak.upgrade() {
                        let mut vec = inputs_cloned.borrow_mut();
                        for f in files { vec.push(f.path().to_path_buf()); }
                        ui.set_image_source_count(vec.len() as i32);
                        
                        let cloned_vec = vec.clone();
                        let target_format = ui.get_image_selected_target_format().to_string();
                        let ui_weak2 = ui.as_weak();
                        std::thread::spawn(move || {
                            let mut all_extensions = HashSet::new();
                            for path in cloned_vec {
                                if path.is_file() {
                                    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                                        if file_help_converter::can_convert(ext, &target_format) {
                                            all_extensions.insert(ext.to_lowercase());
                                        }
                                    }
                                }
                            }
                            let mut extensions: Vec<String> = all_extensions.into_iter().collect();
                            extensions.sort();
                            
                            let _ = slint::invoke_from_event_loop(move || {
                                if let Some(ui) = ui_weak2.upgrade() {
                                    let items: Vec<ExtensionToggle> = extensions.into_iter().map(|ext| {
                                        ExtensionToggle { ext: SharedString::from(ext), selected: true }
                                    }).collect();
                                    let model = std::rc::Rc::new(slint::VecModel::from(items));
                                    ui.set_image_available_input_extensions(model.into());
                                }
                            });
                        });
                    }
                }
            }).unwrap();
        }
    });

    let ui_handle = ui.as_weak();
    let inputs = Rc::clone(&image_inputs);
    ui.on_image_add_folder(move || {
        if let Some(ui) = ui_handle.upgrade() {
            let ui_weak = ui.as_weak();
            let inputs_cloned = Rc::clone(&inputs);
            slint::spawn_local(async move {
                if let Some(folders) = rfd::AsyncFileDialog::new().pick_folders().await {
                    if let Some(ui) = ui_weak.upgrade() {
                        let mut vec = inputs_cloned.borrow_mut();
                        for f in folders { vec.push(f.path().to_path_buf()); }
                        ui.set_image_source_count(vec.len() as i32);
                        
                        let cloned_vec = vec.clone();
                        let target_format = ui.get_image_selected_target_format().to_string();
                        let ui_weak2 = ui.as_weak();
                        std::thread::spawn(move || {
                            let mut all_extensions = HashSet::new();
                            for path in cloned_vec {
                                if path.is_dir() {
                                    if let Ok(exts) = file_help_converter::scan_convertible_extensions(&path, &target_format) {
                                        all_extensions.extend(exts);
                                    }
                                }
                            }
                            let mut extensions: Vec<String> = all_extensions.into_iter().collect();
                            extensions.sort();
                            
                            let _ = slint::invoke_from_event_loop(move || {
                                if let Some(ui) = ui_weak2.upgrade() {
                                    let items: Vec<ExtensionToggle> = extensions.into_iter().map(|ext| {
                                        ExtensionToggle { ext: SharedString::from(ext), selected: true }
                                    }).collect();
                                    let model = std::rc::Rc::new(slint::VecModel::from(items));
                                    ui.set_image_available_input_extensions(model.into());
                                }
                            });
                        });
                    }
                }
            }).unwrap();
        }
    });

    let ui_handle = ui.as_weak();
    ui.on_image_select_target(move || {
        if let Some(ui) = ui_handle.upgrade() {
            let ui_weak = ui.as_weak();
            slint::spawn_local(async move {
                if let Some(folder) = rfd::AsyncFileDialog::new().pick_folder().await {
                    if let Some(ui) = ui_weak.upgrade() {
                        ui.set_image_target_dir(SharedString::from(folder.path().to_string_lossy().into_owned()));
                    }
                }
            }).unwrap();
        }
    });

    let ui_handle = ui.as_weak();
    let inputs = Rc::clone(&image_inputs);
    ui.on_reset_image_state(move || {
        if let Some(ui) = ui_handle.upgrade() {
            inputs.borrow_mut().clear();
            ui.set_image_source_count(0);
            ui.set_image_target_dir(SharedString::from(""));
            let empty_model: Vec<ExtensionToggle> = Vec::new();
            ui.set_image_available_input_extensions(std::rc::Rc::new(slint::VecModel::from(empty_model)).into());
            ui.set_image_status_message(SharedString::from("Ready"));
            ui.set_image_selected_target_format(SharedString::from("png"));
        }
    });
    
    let ui_handle = ui.as_weak();
    ui.on_image_select_all(move || {
        if let Some(ui) = ui_handle.upgrade() {
            let model = ui.get_image_available_input_extensions();
            for i in 0..model.row_count() {
                if let Some(mut item) = model.row_data(i) { item.selected = true; model.set_row_data(i, item); }
            }
        }
    });

    let ui_handle = ui.as_weak();
    ui.on_image_deselect_all(move || {
        if let Some(ui) = ui_handle.upgrade() {
            let model = ui.get_image_available_input_extensions();
            for i in 0..model.row_count() {
                if let Some(mut item) = model.row_data(i) { item.selected = false; model.set_row_data(i, item); }
            }
        }
    });

    let ui_handle = ui.as_weak();
    let inputs = Rc::clone(&image_inputs);
    ui.on_start_image_conversion(move || {
        if let Some(ui) = ui_handle.upgrade() {
            let target_dir = PathBuf::from(ui.get_image_target_dir().to_string());
            let target_format = ui.get_image_selected_target_format().to_string();
            let remove_duplicates = ui.get_remove_duplicates();
            
            let model = ui.get_image_available_input_extensions();
            let mut allowed_input_extensions = Vec::new();
            for i in 0..model.row_count() {
                if let Some(item) = model.row_data(i) {
                    if item.selected { allowed_input_extensions.push(item.ext.to_string()); }
                }
            }

            ui.set_image_status_message(SharedString::from("Converting..."));
            let ui_weak = ui.as_weak();
            let ui_callback_weak = ui.as_weak();
            let inputs_to_process = inputs.borrow().clone();
            
            std::thread::spawn(move || {
                let progress_callback = move |current: usize, total: usize, file_name: &str| {
                    let file_name_owned = file_name.to_string();
                    let _ = slint::invoke_from_event_loop({
                        let weak = ui_callback_weak.clone();
                        move || {
                            if let Some(ui) = weak.upgrade() {
                                let progress = current as f32 / total as f32;
                                ui.set_progress_value(progress);
                                ui.set_status_text(SharedString::from(format!("Converting ({}/{}): {}", current, total, file_name_owned)));
                            }
                        }
                    });
                };

                let result = file_help_converter::batch_process_paths(
                    inputs_to_process,
                    &target_dir,
                    &target_format,
                    &allowed_input_extensions,
                    remove_duplicates,
                    progress_callback
                );

                let is_ok = result.is_ok();
                let (message, errors) = match result {
                    Ok((count, errs)) => (format!("Successfully converted {} files.", count), errs),
                    Err(e) => (format!("Error: {}", e), vec![]),
                };
                let error_log = errors.join("\n");

                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_weak.upgrade() {
                        ui.set_image_status_message(SharedString::from(message));
                        ui.set_converter_error_log(SharedString::from(error_log));
                        if is_ok { ui.invoke_show_image_success_popup(); }
                    }
                });
            });
        }
    });

    ui.run()
}
