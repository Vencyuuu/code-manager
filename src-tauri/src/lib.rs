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

// 移除 ANSI 转义序列
fn strip_ansi_codes(s: &str) -> String {
    // ANSI 转义序列的正则表达式
    let re = regex::Regex::new(r"\x1B\[[0-9;]*[a-zA-Z]").unwrap();
    re.replace_all(s, "").to_string()
}

#[cfg(windows)]
fn convert_output(bytes: &[u8]) -> String {
    // Try GBK to UTF-8 conversion for Chinese Windows
    let (decoded, _, had_errors) = encoding_rs::GBK.decode(bytes);
    let result = if had_errors {
        // Fallback to lossy UTF-8
        String::from_utf8_lossy(bytes).to_string()
    } else {
        decoded.to_string()
    };
    // 移除 ANSI 转义序列
    strip_ansi_codes(&result)
}

#[cfg(not(windows))]
fn convert_output(bytes: &[u8]) -> String {
    let result = String::from_utf8_lossy(bytes).to_string();
    // 移除 ANSI 转义序列
    strip_ansi_codes(&result)
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
    add_branch: Option<bool>,
    add_env: Option<bool>,
    env_name: Option<String>,
    add_version: Option<bool>,
    version_name: Option<String>,
    group_id: Option<String>,
    last_updated_at: Option<i64>,
    remark: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
struct AppConfig {
    dist_output_path: String,
    auto_refresh_enabled: bool,
    auto_refresh_interval: i32,
}

#[derive(Serialize, Deserialize, Clone)]
struct GitBranch {
    name: String,
    current: bool,
}

#[derive(Serialize, Deserialize, Clone)]
struct Group {
    id: String,
    name: String,
    color: String,
    icon: Option<String>,
    created_at: i64,
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

// 尝试查找 IDE 的完整路径 - 简化版本
fn find_ide_path(ide: &str) -> Option<String> {
    // 根据IDE类型获取搜索配置
    let (exe_names, base_paths): (Vec<&str>, Vec<&str>) = match ide {
        "vscode" => (
            vec!["code.exe"],
            vec![
                r"%LOCALAPPDATA%\Programs\Microsoft VS Code\bin",
                r"%PROGRAMFILES%\Microsoft VS Code\bin",
                r"%PROGRAMFILES(X86)%\Microsoft VS Code\bin",
            ],
        ),
        "cursor" => (
            vec!["Cursor.exe"],
            vec![
                r"%LOCALAPPDATA%\Programs\Cursor",
                r"%APPDATA%\Cursor",
            ],
        ),
        "webstorm" => (
            vec!["webstorm64.exe"],
            vec![
                r"%PROGRAMFILES%\JetBrains\WebStorm\bin",
                r"%PROGRAMFILES(X86)%\JetBrains\WebStorm\bin",
            ],
        ),
        "trae" => (
            vec!["Trae.exe", "Trae CN.exe"],
            vec![
                r"%LOCALAPPDATA%\Programs\Trae CN",
                r"%LOCALAPPDATA%\Programs\Trae",
            ],
        ),
        _ => return None,
    };

    // 1. 先在PATH中查找
    for exe_name in &exe_names {
        let output = Command::new("powershell")
            .args(["-NoProfile", "-WindowStyle", "Hidden", "-Command", &format!("Get-Command '{}' -ErrorAction SilentlyContinue | Select-Object -ExpandProperty Source", exe_name)])
            .creation_flags(0x08000000)
            .output()
            .ok()?;
        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !path.is_empty() {
            return Some(path);
        }
    }

    // 2. 展开环境变量并构建搜索路径
    let mut all_paths: Vec<String> = Vec::new();

    // 添加基本路径
    for p in &base_paths {
        let expanded = Command::new("powershell")
            .args(["-NoProfile", "-WindowStyle", "Hidden", "-Command", &format!("[Environment]::ExpandEnvironmentVariables('{}')", p)])
            .creation_flags(0x08000000)
            .output()
            .ok()?;
        let path = String::from_utf8_lossy(&expanded.stdout).trim().to_string();
        if !path.is_empty() {
            all_paths.push(path);
        }
    }

    // 添加用户目录路径
    if let Ok(userprofile) = std::env::var("USERPROFILE") {
        let user_paths = vec![
            format!(r"{}\AppData\Local\Programs\Microsoft VS Code\bin", userprofile),
            format!(r"{}\AppData\Local\Programs\Cursor", userprofile),
            format!(r"{}\AppData\Local\Programs\Trae CN", userprofile),
            format!(r"{}\AppData\Local\Programs\Trae", userprofile),
            format!(r"{}\AppData\Roaming\Cursor", userprofile),
        ];
        all_paths.extend(user_paths);
    }

    // 3. 在所有路径中搜索exe
    for search_path in &all_paths {
        for exe_name in &exe_names {
            let find_cmd = format!(
                "if (Test-Path '{}') {{ Get-ChildItem -Path '{}' -Filter '{}' -ErrorAction SilentlyContinue | Select-Object -First 1 -ExpandProperty FullName }}",
                search_path, search_path, exe_name
            );
            let output = Command::new("powershell")
                .args(["-NoProfile", "-WindowStyle", "Hidden", "-Command", &find_cmd])
                .creation_flags(0x08000000)
                .output()
                .ok()?;
            let found = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !found.is_empty() {
                return Some(found);
            }
        }
    }

    None
}

#[tauri::command]
fn open_in_vscode(path: String, ide_path: Option<String>) -> Result<(), String> {
    // 使用传入的路径或查找 VSCode 路径
    let exe_path = ide_path.or_else(|| find_ide_path("vscode")).unwrap_or_else(|| "code".to_string());

    // 使用 PowerShell Start-Process 隐藏窗口启动
    let result = Command::new("powershell")
        .args(["-NoProfile", "-WindowStyle", "Hidden", "-Command", &format!("Start-Process -FilePath '{}' -ArgumentList '{}' -WorkingDirectory '{}'", exe_path, &path, &path)])
        .creation_flags(0x08000000)
        .spawn();

    match result {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("启动失败: {}", e)),
    }
}

#[tauri::command]
fn open_in_cursor(path: String, ide_path: Option<String>) -> Result<(), String> {
    // 使用传入的路径或查找 Cursor 路径
    let exe_path = ide_path.or_else(|| find_ide_path("cursor")).unwrap_or_else(|| "cursor".to_string());

    // 使用 PowerShell Start-Process 隐藏窗口启动
    let result = Command::new("powershell")
        .args(["-NoProfile", "-WindowStyle", "Hidden", "-Command", &format!("Start-Process -FilePath '{}' -ArgumentList '{}' -WorkingDirectory '{}'", exe_path, &path, &path)])
        .creation_flags(0x08000000)
        .spawn();

    match result {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("启动失败: {}", e)),
    }
}

#[tauri::command]
fn open_in_webstorm(path: String, ide_path: Option<String>) -> Result<(), String> {
    // 使用传入的路径或查找 WebStorm 路径
    let exe_path = ide_path.or_else(|| find_ide_path("webstorm")).unwrap_or_else(|| "webstorm".to_string());

    // 使用 PowerShell Start-Process 隐藏窗口启动
    let result = Command::new("powershell")
        .args(["-NoProfile", "-WindowStyle", "Hidden", "-Command", &format!("Start-Process -FilePath '{}' -ArgumentList '{}' -WorkingDirectory '{}'", exe_path, &path, &path)])
        .creation_flags(0x08000000)
        .spawn();

    match result {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("启动失败: {}", e)),
    }
}

#[tauri::command]
fn open_in_trae(path: String, ide_path: Option<String>) -> Result<(), String> {
    // 使用传入的路径或查找 Trae 路径
    let exe_path = ide_path.or_else(|| find_ide_path("trae")).unwrap_or_else(|| "Trae".to_string());

    // 使用 PowerShell Start-Process 隐藏窗口启动
    let result = Command::new("powershell")
        .args(["-NoProfile", "-WindowStyle", "Hidden", "-Command", &format!("Start-Process -FilePath '{}' -ArgumentList '{}' -WorkingDirectory '{}'", exe_path, &path, &path)])
        .creation_flags(0x08000000)
        .spawn();

    match result {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("启动失败: {}", e)),
    }
}

#[tauri::command]
fn open_in_wechat(path: String, ide_path: Option<String>) -> Result<(), String> {
    // 使用传入的路径或默认路径
    let exe_path = ide_path.unwrap_or_else(|| "wechat-devtools".to_string());

    // 微信开发者工具使用 --project 参数打开项目
    let result = Command::new("powershell")
        .args(["-NoProfile", "-WindowStyle", "Hidden", "-Command", &format!("Start-Process -FilePath '{}' -ArgumentList '{}' -WorkingDirectory '{}'", exe_path, &path, &path)])
        .creation_flags(0x08000000)
        .spawn();

    match result {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("启动失败: {}", e)),
    }
}

// 检查 IDE 是否可用
#[tauri::command]
fn check_ide_available(ide: String) -> Result<serde_json::Value, String> {
    // 使用 find_ide_path 来查找 IDE
    let ide_path = find_ide_path(&ide);

    if let Some(path) = ide_path {
        return Ok(serde_json::json!({
            "available": true,
            "path": path
        }));
    }

    Ok(serde_json::json!({
        "available": false,
        "path": null
    }))
}

// 内部函数：获取当前分支名
fn get_current_branch(path: &str) -> Result<String, String> {
    let output = Command::new("powershell")
        .args(["-NoProfile", "-WindowStyle", "Hidden", "-Command", &format!("cd '{}'; git branch --show-current", path)])
        .creation_flags(0x08000000)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err("Not a git repository".to_string());
    }

    let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(branch)
}

#[tauri::command]
fn get_git_branches(path: String) -> Result<Vec<GitBranch>, String> {
    let output = Command::new("powershell")
        .args(["-NoProfile", "-WindowStyle", "Hidden", "-Command", &format!("cd '{}'; git branch -a", path)])
        .creation_flags(0x08000000)
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
        .args(["-NoProfile", "-WindowStyle", "Hidden", "-Command", &format!("cd '{}'; git checkout {}", path, branch)])
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to switch branch: {}", stderr));
    }

    Ok(())
}

#[tauri::command]
fn scan_subfolders(path: String) -> Result<Vec<String>, String> {
    let mut projects = Vec::new();

    let entries = std::fs::read_dir(&path)
        .map_err(|e| e.to_string())?;

    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let entry_path = entry.path();

        if entry_path.is_dir() {
            // 检查是否是项目目录（包含 package.json 或 .git）
            let has_package_json = entry_path.join("package.json").exists();
            let has_git = entry_path.join(".git").exists();

            if has_package_json || has_git {
                projects.push(entry_path.to_string_lossy().to_string());
            }
        }
    }

    Ok(projects)
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

        // 注意: running_processes 的清理由前端通过 cleanup_script_state 命令处理

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

// 仅清理脚本状态（不杀死进程），用于脚本正常完成后清理
#[tauri::command]
fn cleanup_script_state(state: tauri::State<AppState>, project_id: String) -> Result<(), String> {
    let mut processes = state.running_processes.lock().map_err(|e| e.to_string())?;
    processes.remove(&project_id);
    Ok(())
}

fn kill_process(pid: u32) -> Result<(), String> {
    #[cfg(windows)]
    {
        // 使用 cmd /c start 隐藏窗口执行 taskkill
        let output = Command::new("cmd")
            .args(["/c", "start", "", "taskkill", "/PID", &pid.to_string(), "/T", "/F"])
            .creation_flags(0x08000000)
            .spawn()
            .map_err(|e| e.to_string())?;

        // 不等待结果，因为 start 命令会立即返回
        let _ = output;
        return Ok(());
    }

    Ok(())
}

#[tauri::command]
fn check_is_git_repo(path: String) -> bool {
    let git_path = std::path::Path::new(&path).join(".git");
    git_path.exists()
}

#[tauri::command]
fn update_project_config(state: tauri::State<AppState>, id: String, product_name_template: Option<String>, add_timestamp: Option<bool>, add_branch: Option<bool>, add_env: Option<bool>, env_name: Option<String>, add_version: Option<bool>, version_name: Option<String>) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    conn.execute(
        "UPDATE projects SET product_name_template = ?1, add_timestamp = ?2, add_branch = ?3, add_env = ?4, env_name = ?5, add_version = ?6, version_name = ?7 WHERE id = ?8",
        (&product_name_template, &add_timestamp, &add_branch, &add_env, &env_name, &add_version, &version_name, &id),
    ).map_err(|e| e.to_string())?;

    Ok(())
}

// 更新项目备注
#[tauri::command]
fn update_project_remark(state: tauri::State<AppState>, id: String, remark: Option<String>) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    conn.execute(
        "UPDATE projects SET remark = ?1 WHERE id = ?2",
        (&remark, &id),
    ).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
fn copy_dist_and_zip(state: tauri::State<AppState>, id: String, path: String) -> Result<String, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    // 获取项目配置
    let mut stmt = conn.prepare("SELECT name, product_name_template, add_timestamp, add_branch, add_env, env_name, add_version, version_name FROM projects WHERE id = ?1")
        .map_err(|e| e.to_string())?;

    let (name, product_name_template, add_timestamp, add_branch, add_env, env_name, add_version, version_name): (String, Option<String>, Option<bool>, Option<bool>, Option<bool>, Option<String>, Option<bool>, Option<String>) = stmt.query_row([&id], |row| {
        Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?, row.get(6)?, row.get(7)?))
    }).map_err(|e| e.to_string())?;

    // 获取应用配置
    let mut config_stmt = conn.prepare("SELECT dist_output_path FROM app_config WHERE id = 1")
        .map_err(|e| e.to_string())?;

    let dist_output_path: String = config_stmt.query_row([], |row| row.get(0))
        .map_err(|e| e.to_string())?;

    let template = product_name_template.unwrap_or(name.clone());

    // 构建文件名：template + branch + env + version + timestamp
    let mut parts: Vec<String> = vec![template];

    // 添加分支名
    if add_branch.unwrap_or(false) {
        // 获取当前分支名
        if let Ok(branch) = get_current_branch(&path) {
            parts.push(branch);
        }
    }

    // 添加环境名
    if add_env.unwrap_or(false) {
        if let Some(env) = env_name {
            if !env.is_empty() {
                parts.push(env);
            }
        }
    }

    // 添加版本号
    if add_version.unwrap_or(false) {
        if let Some(version) = version_name {
            if !version.is_empty() {
                parts.push(version);
            }
        }
    }

    // 添加时间戳
    if add_timestamp.unwrap_or(false) {
        parts.push(Utc::now().format("%Y%m%d_%H%M%S").to_string());
    }

    let folder_name = parts.join("_");

    // 检查 dist 目录是否存在
    let dist_path = std::path::Path::new(&path).join("dist");
    if !dist_path.exists() {
        return Err("dist 目录不存在".to_string());
    }

    // 确定输出目录（优先使用配置中的路径，否则使用项目目录下的 output 文件夹）
    let output_dir = if dist_output_path.is_empty() {
        std::path::Path::new(&path).join("output")
    } else {
        std::path::Path::new(&dist_output_path).join(&folder_name)
    };
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
fn get_config(state: tauri::State<AppState>) -> Result<AppConfig, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn.prepare("SELECT dist_output_path, auto_refresh_enabled, auto_refresh_interval FROM app_config WHERE id = 1")
        .map_err(|e| e.to_string())?;

    let config = stmt.query_row([], |row| {
        Ok(AppConfig {
            dist_output_path: row.get(0)?,
            auto_refresh_enabled: row.get::<_, i32>(1)? != 0,
            auto_refresh_interval: row.get(2)?,
        })
    }).map_err(|e| e.to_string())?;

    Ok(config)
}

#[tauri::command]
fn save_config(state: tauri::State<AppState>, dist_output_path: String, auto_refresh_enabled: bool, auto_refresh_interval: String) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let interval = auto_refresh_interval.parse::<i32>().unwrap_or(60);

    conn.execute(
        "UPDATE app_config SET dist_output_path = ?1, auto_refresh_enabled = ?2, auto_refresh_interval = ?3 WHERE id = 1",
        (&dist_output_path, if auto_refresh_enabled { 1 } else { 0 }, &interval),
    ).map_err(|e| e.to_string())?;

    Ok(())
}

// IDE 配置结构体
#[derive(Serialize, Deserialize, Clone)]
struct IdeConfig {
    id: String,
    name: String,
    path: String,
    enabled: bool,
}

// 获取所有 IDE 配置
#[tauri::command]
fn get_ide_configs(state: tauri::State<AppState>) -> Result<Vec<IdeConfig>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn.prepare("SELECT id, name, path, enabled FROM ide_config")
        .map_err(|e| e.to_string())?;

    let configs = stmt.query_map([], |row| {
        Ok(IdeConfig {
            id: row.get(0)?,
            name: row.get(1)?,
            path: row.get(2)?,
            enabled: row.get::<_, i32>(3)? != 0,
        })
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();

    Ok(configs)
}

// 保存 IDE 配置
#[tauri::command]
fn save_ide_config(state: tauri::State<AppState>, id: String, name: String, path: String, enabled: bool) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT OR REPLACE INTO ide_config (id, name, path, enabled) VALUES (?1, ?2, ?3, ?4)",
        (&id, &name, &path, if enabled { 1 } else { 0 }),
    ).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
fn get_projects(state: tauri::State<AppState>) -> Result<Vec<Project>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn.prepare("SELECT id, name, path, logo, added_at, is_git_repo, product_name_template, add_timestamp, add_branch, add_env, env_name, add_version, version_name, group_id, last_updated_at, remark FROM projects ORDER BY added_at DESC")
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
            add_branch: row.get(8)?,
            add_env: row.get(9)?,
            env_name: row.get(10)?,
            add_version: row.get(11)?,
            version_name: row.get(12)?,
            group_id: row.get(13)?,
            last_updated_at: row.get(14)?,
            remark: row.get(15)?,
        })
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();

    Ok(projects)
}

#[tauri::command]
fn add_project(state: tauri::State<AppState>, id: String, name: String, path: String, logo: Option<String>, group_id: Option<String>) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let added_at = chrono::Utc::now().timestamp();
    let is_git_repo = check_is_git_repo(path.clone());

    conn.execute(
        "INSERT INTO projects (id, name, path, logo, added_at, is_git_repo, last_updated_at, group_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        (&id, &name, &path, &logo, &added_at, &is_git_repo, &added_at, &group_id),
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
    let mut stmt = conn.prepare("SELECT id, name, path, logo, added_at, is_git_repo, product_name_template, add_timestamp, add_branch, add_env, env_name, add_version, version_name, group_id, last_updated_at, remark FROM projects")
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
            add_branch: row.get(8)?,
            add_env: row.get(9)?,
            env_name: row.get(10)?,
            add_version: row.get(11)?,
            version_name: row.get(12)?,
            group_id: row.get(13)?,
            last_updated_at: row.get(14)?,
            remark: row.get(15)?,
        })
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();

    let now = chrono::Utc::now().timestamp();

    // 更新每个项目的 git 状态和更新时间
    for project in &projects {
        let is_git = check_is_git_repo(project.path.clone());

        conn.execute(
            "UPDATE projects SET is_git_repo = ?1, last_updated_at = ?2 WHERE id = ?3",
            (&is_git, &now, &project.id),
        ).map_err(|e| e.to_string())?;
    }

    // 重新查询获取更新后的数据
    let mut stmt = conn.prepare("SELECT id, name, path, logo, added_at, is_git_repo, product_name_template, add_timestamp, add_branch, add_env, env_name, add_version, version_name, group_id, last_updated_at, remark FROM projects ORDER BY added_at DESC")
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
            add_branch: row.get(8)?,
            add_env: row.get(9)?,
            env_name: row.get(10)?,
            add_version: row.get(11)?,
            version_name: row.get(12)?,
            group_id: row.get(13)?,
            last_updated_at: row.get(14)?,
            remark: row.get(15)?,
        })
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();

    Ok(updated_projects)
}

// 获取所有分组
#[tauri::command]
fn get_groups(state: tauri::State<AppState>) -> Result<Vec<Group>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn.prepare("SELECT id, name, color, icon, created_at FROM groups ORDER BY created_at DESC")
        .map_err(|e| e.to_string())?;

    let groups = stmt.query_map([], |row| {
        Ok(Group {
            id: row.get(0)?,
            name: row.get(1)?,
            color: row.get(2)?,
            icon: row.get(3)?,
            created_at: row.get(4)?,
        })
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();

    Ok(groups)
}

// 创建分组
#[tauri::command]
fn create_group(state: tauri::State<AppState>, id: String, name: String, color: String, icon: Option<String>) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let created_at = chrono::Utc::now().timestamp();

    conn.execute(
        "INSERT INTO groups (id, name, color, icon, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        (&id, &name, &color, &icon, &created_at),
    ).map_err(|e| e.to_string())?;

    Ok(())
}

// 更新分组
#[tauri::command]
fn update_group(state: tauri::State<AppState>, id: String, name: String, color: String, icon: Option<String>) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    conn.execute(
        "UPDATE groups SET name = ?1, color = ?2, icon = ?3 WHERE id = ?4",
        (&name, &color, &icon, &id),
    ).map_err(|e| e.to_string())?;

    Ok(())
}

// 删除分组
#[tauri::command]
fn delete_group(state: tauri::State<AppState>, id: String) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    // 先解除该项目与分组的关联
    conn.execute(
        "UPDATE projects SET group_id = NULL WHERE group_id = ?1",
        [&id],
    ).map_err(|e| e.to_string())?;

    // 删除分组
    conn.execute(
        "DELETE FROM groups WHERE id = ?1",
        [&id],
    ).map_err(|e| e.to_string())?;

    Ok(())
}

// 绑定项目到分组
#[tauri::command]
fn bind_project_to_group(state: tauri::State<AppState>, project_id: String, group_id: Option<String>) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    conn.execute(
        "UPDATE projects SET group_id = ?1 WHERE id = ?2",
        (&group_id, &project_id),
    ).map_err(|e| e.to_string())?;

    Ok(())
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
            add_timestamp INTEGER,
            last_updated_at INTEGER
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

    // 检查并添加 add_branch 列（如果不存在）
    let _ = conn.execute(
        "ALTER TABLE projects ADD COLUMN add_branch INTEGER",
        [],
    );

    // 检查并添加 env_name 列（如果不存在）
    let _ = conn.execute(
        "ALTER TABLE projects ADD COLUMN env_name TEXT",
        [],
    );

    // 检查并添加 add_env 列（如果不存在）
    let _ = conn.execute(
        "ALTER TABLE projects ADD COLUMN add_env INTEGER",
        [],
    );

    // 检查并添加 env_name 列（如果不存在）
    let _ = conn.execute(
        "ALTER TABLE projects ADD COLUMN env_name TEXT",
        [],
    );

    // 检查并添加 add_version 列（如果不存在）
    let _ = conn.execute(
        "ALTER TABLE projects ADD COLUMN add_version INTEGER",
        [],
    );

    // 检查并添加 version_name 列（如果不存在）
    let _ = conn.execute(
        "ALTER TABLE projects ADD COLUMN version_name TEXT",
        [],
    );

    // 检查并添加 group_id 列（如果不存在）
    let _ = conn.execute(
        "ALTER TABLE projects ADD COLUMN group_id TEXT",
        [],
    );

    // 检查并添加 last_updated_at 列（如果不存在）
    let _ = conn.execute(
        "ALTER TABLE projects ADD COLUMN last_updated_at INTEGER",
        [],
    );

    // 检查并添加 remark 列（如果不存在）
    let _ = conn.execute(
        "ALTER TABLE projects ADD COLUMN remark TEXT",
        [],
    );

    // 创建配置表（如果已存在则忽略）
    conn.execute(
        "CREATE TABLE IF NOT EXISTS app_config (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            dist_output_path TEXT DEFAULT '',
            auto_refresh_enabled INTEGER DEFAULT 0,
            auto_refresh_interval INTEGER DEFAULT 60
        )",
        [],
    )?;

    // 插入默认配置（如果不存在）
    conn.execute(
        "INSERT OR IGNORE INTO app_config (id, dist_output_path, auto_refresh_enabled, auto_refresh_interval) VALUES (1, '', 0, 60)",
        [],
    )?;

    // 创建分组表（如果已存在则忽略）
    conn.execute(
        "CREATE TABLE IF NOT EXISTS groups (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            color TEXT NOT NULL,
            icon TEXT,
            created_at INTEGER NOT NULL
        )",
        [],
    )?;

    // 创建 IDE 配置表（如果已存在则忽略）
    conn.execute(
        "CREATE TABLE IF NOT EXISTS ide_config (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            path TEXT NOT NULL,
            enabled INTEGER DEFAULT 1
        )",
        [],
    )?;

    Ok(conn)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
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
            open_in_cursor,
            open_in_webstorm,
            open_in_trae,
            open_in_wechat,
            scan_subfolders,
            check_ide_available,
            get_ide_configs,
            save_ide_config,
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
            cleanup_script_state,
            check_is_git_repo,
            update_project_config,
            update_project_remark,
            copy_dist_and_zip,
            get_config,
            save_config,
            get_groups,
            create_group,
            update_group,
            delete_group,
            bind_project_to_group
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}