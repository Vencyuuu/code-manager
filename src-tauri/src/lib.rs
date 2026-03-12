use std::process::{Command, Stdio};
use std::sync::Mutex;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::fs;
use rusqlite::Connection;
use tauri::{Manager, Emitter};
use serde::{Deserialize, Serialize};
use chrono::Utc;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
fn convert_output(bytes: &[u8]) -> String {
    // Try GBK to UTF-8 conversion for Chinese Windows
    let (decoded, _, had_errors) = encoding_rs::GBK.decode(bytes);
    if had_errors {
        // Fallback to lossy UTF-8
        String::from_utf8_lossy(bytes).to_string()
    } else {
        decoded.to_string()
    }
}

#[cfg(not(windows))]
fn convert_output(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).to_string()
}

struct AppState {
    db: Mutex<Connection>,
    running_processes: Mutex<HashMap<String, u32>>,
}

#[derive(Serialize, Deserialize, Clone)]
struct Project {
    id: String,
    name: String,
    path: String,
    logo: Option<String>,
    added_at: i64,
    is_git_repo: Option<bool>,
    product_name_template: Option<String>,
    add_timestamp: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone)]
struct GitBranch {
    name: String,
    current: bool,
}

#[derive(Serialize, Deserialize, Clone)]
struct NpmScript {
    name: String,
    script: String,
}

#[derive(Serialize, Deserialize)]
struct PackageJson {
    scripts: std::collections::HashMap<String, String>,
}

#[derive(Serialize, Clone)]
struct ScriptOutput {
    project_id: String,
    output: String,
    is_error: bool,
}

#[tauri::command]
fn open_in_vscode(path: String) -> Result<(), String> {
    let _ = Command::new("cmd")
        .args(["/C", "start", "", "code", &path])
        .spawn();

    Ok(())
}

#[tauri::command]
fn get_git_branches(path: String) -> Result<Vec<GitBranch>, String> {
    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", &format!("cd '{}'; git branch -a", path)])
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err("Not a git repository".to_string());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let branches: Vec<GitBranch> = stdout
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            let name = line.trim().trim_start_matches("* ").to_string();
            let current = line.trim().starts_with('*');
            GitBranch { name, current }
        })
        .collect();

    Ok(branches)
}

#[tauri::command]
fn switch_git_branch(path: String, branch: String) -> Result<(), String> {
    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", &format!("cd '{}'; git checkout {}", path, branch)])
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to switch branch: {}", stderr));
    }

    Ok(())
}

#[tauri::command]
fn get_npm_scripts(path: String) -> Result<Vec<NpmScript>, String> {
    let package_json_path = std::path::Path::new(&path).join("package.json");

    if !package_json_path.exists() {
        return Err("No package.json found".to_string());
    }

    let content = std::fs::read_to_string(&package_json_path)
        .map_err(|e| e.to_string())?;

    let package_json: PackageJson = serde_json::from_str(&content)
        .map_err(|e| e.to_string())?;

    let scripts: Vec<NpmScript> = package_json.scripts
        .iter()
        .map(|(name, script)| NpmScript {
            name: name.clone(),
            script: script.clone(),
        })
        .collect();

    Ok(scripts)
}

#[tauri::command]
fn run_npm_script(app_handle: tauri::AppHandle, state: tauri::State<AppState>, project_id: String, path: String, script_name: String) -> Result<u32, String> {
    let mut processes = state.running_processes.lock().map_err(|e| e.to_string())?;

    if let Some(pid) = processes.get(&project_id) {
        let _ = kill_process(*pid);
    }

    // Emit script started event
    let _ = app_handle.emit("script-started", (&project_id, &script_name));

    // Run with hidden PowerShell
    let mut child = Command::new("powershell")
        .args(["-NoProfile", "-WindowStyle", "Hidden", "-Command", &format!("cd '{}'; chcp 65001 > $null; npm run {}", path, script_name)])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .creation_flags(0x08000000)
        .spawn()
        .map_err(|e: std::io::Error| e.to_string())?;

    let pid = child.id();
    processes.insert(project_id.clone(), pid);

    let app_handle_clone = app_handle.clone();
    let project_id_clone = project_id.clone();

    std::thread::spawn(move || {
        if let Some(stdout) = child.stdout.take() {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                if let Ok(line) = line {
                    // Convert from GBK to UTF-8 for Chinese Windows
                    let converted = convert_output(line.as_bytes());
                    let _ = app_handle_clone.emit("script-output", ScriptOutput {
                        project_id: project_id_clone.clone(),
                        output: converted,
                        is_error: false,
                    });
                }
            }
        }

        let _ = child.wait();

        let _ = app_handle_clone.emit("script-output", ScriptOutput {
            project_id: project_id_clone,
            output: "__SCRIPT_COMPLETED__".to_string(),
            is_error: false,
        });
    });

    Ok(pid)
}

#[tauri::command]
fn run_custom_script(app_handle: tauri::AppHandle, state: tauri::State<AppState>, project_id: String, path: String, script: String) -> Result<u32, String> {
    let mut processes = state.running_processes.lock().map_err(|e| e.to_string())?;

    if let Some(pid) = processes.get(&project_id) {
        let _ = kill_process(*pid);
    }

    // Emit script started event
    let _ = app_handle.emit("script-started", (&project_id, "custom"));

    // Run with hidden PowerShell
    let mut child = Command::new("powershell")
        .args(["-NoProfile", "-WindowStyle", "Hidden", "-Command", &format!("cd '{}'; chcp 65001 > $null; {}", path, script)])
        .current_dir(&path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .creation_flags(0x08000000)
        .spawn()
        .map_err(|e: std::io::Error| e.to_string())?;

    let pid = child.id();
    processes.insert(project_id.clone(), pid);

    let app_handle_clone = app_handle.clone();
    let project_id_clone = project_id.clone();

    std::thread::spawn(move || {
        if let Some(stdout) = child.stdout.take() {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                if let Ok(line) = line {
                    // Convert from GBK to UTF-8 for Chinese Windows
                    let converted = convert_output(line.as_bytes());
                    let _ = app_handle_clone.emit("script-output", ScriptOutput {
                        project_id: project_id_clone.clone(),
                        output: converted,
                        is_error: false,
                    });
                }
            }
        }

        let _ = child.wait();

        let _ = app_handle_clone.emit("script-output", ScriptOutput {
            project_id: project_id_clone,
            output: "__SCRIPT_COMPLETED__".to_string(),
            is_error: false,
        });
    });

    Ok(pid)
}

#[tauri::command]
fn kill_script(state: tauri::State<AppState>, project_id: String) -> Result<(), String> {
    let mut processes = state.running_processes.lock().map_err(|e| e.to_string())?;

    if let Some(pid) = processes.remove(&project_id) {
        kill_process(pid)?;
    }

    Ok(())
}

fn kill_process(pid: u32) -> Result<(), String> {
    #[cfg(windows)]
    {
        let output = Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/T", "/F"])
            .output()
            .map_err(|e| e.to_string())?;

        if !output.status.success() {
            return Err("Failed to kill process".to_string());
        }
    }

    Ok(())
}

#[tauri::command]
fn check_is_git_repo(path: String) -> bool {
    let git_path = std::path::Path::new(&path).join(".git");
    git_path.exists()
}

#[tauri::command]
fn update_project_config(state: tauri::State<AppState>, id: String, product_name_template: Option<String>, add_timestamp: Option<bool>) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    conn.execute(
        "UPDATE projects SET product_name_template = ?1, add_timestamp = ?2 WHERE id = ?3",
        (&product_name_template, &add_timestamp, &id),
    ).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
fn copy_dist_and_zip(state: tauri::State<AppState>, id: String, path: String) -> Result<String, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    // 获取项目配置
    let mut stmt = conn.prepare("SELECT name, product_name_template, add_timestamp FROM projects WHERE id = ?1")
        .map_err(|e| e.to_string())?;

    let (name, product_name_template, add_timestamp): (String, Option<String>, Option<bool>) = stmt.query_row([&id], |row| {
        Ok((row.get(0)?, row.get(1)?, row.get(2)?))
    }).map_err(|e| e.to_string())?;

    let template = product_name_template.unwrap_or(name.clone());
    let timestamp = if add_timestamp.unwrap_or(false) {
        format!("_{}", Utc::now().format("%Y%m%d_%H%M%S"))
    } else {
        String::new()
    };

    let folder_name = format!("{}{}", template, timestamp);

    // 检查 dist 目录是否存在
    let dist_path = std::path::Path::new(&path).join("dist");
    if !dist_path.exists() {
        return Err("dist 目录不存在".to_string());
    }

    // 创建输出目录
    let output_dir = std::path::Path::new(&path).join("output");
    fs::create_dir_all(&output_dir).map_err(|e| e.to_string())?;

    let target_folder = output_dir.join(&folder_name);
    let zip_path = output_dir.join(format!("{}.zip", folder_name));

    // 复制 dist 到目标文件夹
    copy_dir_all(&dist_path, &target_folder).map_err(|e| e.to_string())?;

    // 创建 zip 文件
    let file = fs::File::create(&zip_path).map_err(|e| e.to_string())?;
    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    add_dir_to_zip(&mut zip, &target_folder, &target_folder, &options).map_err(|e| e.to_string())?;

    zip.finish().map_err(|e| e.to_string())?;

    Ok(zip_path.to_string_lossy().to_string())
}

fn copy_dir_all(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst.join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.join(entry.file_name()))?;
        }
    }
    Ok(())
}

fn add_dir_to_zip(zip: &mut ZipWriter<fs::File>, base_path: &std::path::Path, current_path: &std::path::Path, options: &SimpleFileOptions) -> std::io::Result<()> {
    for entry in fs::read_dir(current_path)? {
        let entry = entry?;
        let path = entry.path();
        let relative_path = path.strip_prefix(base_path.parent().unwrap_or(base_path)).unwrap_or(&path);
        let name = relative_path.to_string_lossy().replace("\\", "/");

        if path.is_dir() {
            zip.add_directory(&name, *options).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
            add_dir_to_zip(zip, base_path, &path, options)?;
        } else {
            zip.start_file(&name, *options).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
            let content = fs::read(&path)?;
            zip.write_all(&content).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        }
    }
    Ok(())
}

#[tauri::command]
fn get_projects(state: tauri::State<AppState>) -> Result<Vec<Project>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn.prepare("SELECT id, name, path, logo, added_at, is_git_repo, product_name_template, add_timestamp FROM projects ORDER BY added_at DESC")
        .map_err(|e| e.to_string())?;

    let projects = stmt.query_map([], |row| {
        Ok(Project {
            id: row.get(0)?,
            name: row.get(1)?,
            path: row.get(2)?,
            logo: row.get(3)?,
            added_at: row.get(4)?,
            is_git_repo: row.get(5)?,
            product_name_template: row.get(6)?,
            add_timestamp: row.get(7)?,
        })
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();

    Ok(projects)
}

#[tauri::command]
fn add_project(state: tauri::State<AppState>, id: String, name: String, path: String, logo: Option<String>) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let added_at = chrono::Utc::now().timestamp();
    let is_git_repo = check_is_git_repo(path.clone());

    conn.execute(
        "INSERT INTO projects (id, name, path, logo, added_at, is_git_repo) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        (&id, &name, &path, &logo, &added_at, &is_git_repo),
    ).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
fn remove_project(state: tauri::State<AppState>, id: String) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let mut processes = state.running_processes.lock().map_err(|e| e.to_string())?;
    if let Some(pid) = processes.remove(&id) {
        let _ = kill_process(pid);
    }

    conn.execute("DELETE FROM projects WHERE id = ?1", [&id])
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
fn refresh_all_projects(state: tauri::State<AppState>) -> Result<Vec<Project>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    // 获取所有项目
    let mut stmt = conn.prepare("SELECT id, name, path, logo, added_at, is_git_repo, product_name_template, add_timestamp FROM projects")
        .map_err(|e| e.to_string())?;

    let projects: Vec<Project> = stmt.query_map([], |row| {
        Ok(Project {
            id: row.get(0)?,
            name: row.get(1)?,
            path: row.get(2)?,
            logo: row.get(3)?,
            added_at: row.get(4)?,
            is_git_repo: row.get(5)?,
            product_name_template: row.get(6)?,
            add_timestamp: row.get(7)?,
        })
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();

    // 更新每个项目的 git 状态
    for project in &projects {
        let is_git = check_is_git_repo(project.path.clone());

        conn.execute(
            "UPDATE projects SET is_git_repo = ?1 WHERE id = ?2",
            (&is_git, &project.id),
        ).map_err(|e| e.to_string())?;
    }

    // 重新查询获取更新后的数据
    let mut stmt = conn.prepare("SELECT id, name, path, logo, added_at, is_git_repo, product_name_template, add_timestamp FROM projects ORDER BY added_at DESC")
        .map_err(|e| e.to_string())?;

    let updated_projects: Vec<Project> = stmt.query_map([], |row| {
        Ok(Project {
            id: row.get(0)?,
            name: row.get(1)?,
            path: row.get(2)?,
            logo: row.get(3)?,
            added_at: row.get(4)?,
            is_git_repo: row.get(5)?,
            product_name_template: row.get(6)?,
            add_timestamp: row.get(7)?,
        })
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();

    Ok(updated_projects)
}

fn init_db(app_dir: &std::path::Path) -> Result<Connection, rusqlite::Error> {
    let db_path = app_dir.join("projects.db");
    let conn = Connection::open(db_path)?;

    // 创建表（如果已存在则忽略）
    conn.execute(
        "CREATE TABLE IF NOT EXISTS projects (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            path TEXT NOT NULL,
            logo TEXT,
            added_at INTEGER NOT NULL,
            is_git_repo INTEGER,
            product_name_template TEXT,
            add_timestamp INTEGER
        )",
        [],
    )?;

    // 检查并添加 is_git_repo 列（如果不存在）
    let _ = conn.execute(
        "ALTER TABLE projects ADD COLUMN is_git_repo INTEGER",
        [],
    );

    // 检查并添加 product_name_template 列（如果不存在）
    let _ = conn.execute(
        "ALTER TABLE projects ADD COLUMN product_name_template TEXT",
        [],
    );

    // 检查并添加 add_timestamp 列（如果不存在）
    let _ = conn.execute(
        "ALTER TABLE projects ADD COLUMN add_timestamp INTEGER",
        [],
    );

    Ok(conn)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let app_dir = app.path().app_data_dir().expect("Failed to get app data dir");
            std::fs::create_dir_all(&app_dir).expect("Failed to create app data dir");

            let conn = init_db(&app_dir).expect("Failed to initialize database");

            app.manage(AppState {
                db: Mutex::new(conn),
                running_processes: Mutex::new(HashMap::new()),
            });

            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            open_in_vscode,
            get_projects,
            add_project,
            remove_project,
            refresh_all_projects,
            get_git_branches,
            switch_git_branch,
            get_npm_scripts,
            run_npm_script,
            run_custom_script,
            kill_script,
            check_is_git_repo,
            update_project_config,
            copy_dist_and_zip
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}