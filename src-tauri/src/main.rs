#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::fs;
use std::path::PathBuf;
use tauri::command;
use serde::{Deserialize, Serialize};

const PUBLISH_FILE: &str = "publish.xml";

// 默认插件配置（公司的 jaidoc-wps）
const DEFAULT_PLUGIN_NAME: &str = "jaidoc-wps";
const DEFAULT_PLUGIN_PATH: &str = "/jaidoc-wps/";

#[derive(Serialize, Clone)]
struct PluginInfo {
    name: String,
    url: String,
    plugin_type: String,
}

fn get_wps_plugin_dir() -> Result<PathBuf, String> {
    let home = dirs::home_dir().ok_or("无法获取用户主目录")?;
    let plugin_dir = home.join("Library/Containers/com.kingsoft.wpsoffice.mac/Data/.kingsoft/wps/jsaddons/");
    Ok(plugin_dir)
}

fn ensure_plugin_dir_exists() -> Result<PathBuf, String> {
    let plugin_dir = get_wps_plugin_dir()?;
    if !plugin_dir.exists() {
        fs::create_dir_all(&plugin_dir).map_err(|e| format!("创建插件目录失败: {}", e))?;
    }
    Ok(plugin_dir)
}

fn get_publish_path() -> Result<PathBuf, String> {
    let plugin_dir = ensure_plugin_dir_exists()?;
    Ok(plugin_dir.join(PUBLISH_FILE))
}

fn create_backup(publish_path: &PathBuf) -> Result<(), String> {
    let backup_path = publish_path.with_extension("xml.bak");
    fs::copy(publish_path, backup_path)
        .map_err(|e| format!("创建备份失败: {}", e))?;
    Ok(())
}

fn generate_plugin_xml(name: &str, url: &str) -> String {
    format!(
        r#"<jspluginonline name="{}" url="{}" type="wps" install="{}"/>"#,
        name, url, url
    )
}

fn create_new_publish_file(publish_path: &PathBuf, name: &str, url: &str) -> Result<(), String> {
    let plugin_xml = generate_plugin_xml(name, url);
    let content = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<jsplugins>
  {}
</jsplugins>"#,
        plugin_xml
    );
    fs::write(publish_path, content)
        .map_err(|e| format!("创建 publish.xml 失败: {}", e))?;
    Ok(())
}

fn update_existing_publish_file(publish_path: &PathBuf, name: &str, url: &str) -> Result<(), String> {
    let content = fs::read_to_string(publish_path)
        .map_err(|e| format!("读取 publish.xml 失败: {}", e))?;
    
    let plugin_xml = generate_plugin_xml(name, url);
    let new_content: String;
    
    if content.contains(&format!(r#"name="{}""#, name)) {
        // 更新现有插件
        let pattern = format!(r#"<jspluginonline[^>]*name="{}"[^>]*/>"#, regex::escape(name));
        let re = regex::Regex::new(&pattern).map_err(|e| format!("正则表达式错误: {}", e))?;
        new_content = re.replace(&content, &plugin_xml).to_string();
    } else if content.contains("</jsplugins>") {
        // 添加新插件
        new_content = content.replace("</jsplugins>", &format!("  {}\n</jsplugins>", plugin_xml));
    } else {
        // 文件格式不正确，重新创建
        return create_new_publish_file(publish_path, name, url);
    }
    
    fs::write(publish_path, new_content)
        .map_err(|e| format!("写入 publish.xml 失败: {}", e))?;
    Ok(())
}

#[command]
fn install_plugin(domain: String, protocol: String, plugin_name: Option<String>, plugin_path: Option<String>) -> Result<String, String> {
    if domain.is_empty() {
        return Err("域名不能为空".to_string());
    }
    
    let protocol = protocol.to_lowercase();
    if protocol != "http" && protocol != "https" {
        return Err("协议必须是 http 或 https".to_string());
    }
    
    let name = plugin_name.unwrap_or_else(|| DEFAULT_PLUGIN_NAME.to_string());
    let path = plugin_path.unwrap_or_else(|| DEFAULT_PLUGIN_PATH.to_string());
    
    // 清理域名：移除可能的协议前缀
    let domain = domain
        .replace("https://", "")
        .replace("http://", "")
        .trim_end_matches('/')
        .to_string();
    
    if domain.is_empty() {
        return Err("域名格式不正确".to_string());
    }
    
    // 确保 path 以 / 开头和结尾
    let path = if path.starts_with('/') { path } else { format!("/{}", path) };
    let path = if path.ends_with('/') { path } else { format!("{}/", path) };
    
    let url = format!("{}://{}{}", protocol, domain, path);
    let publish_path = get_publish_path()?;
    
    if publish_path.exists() {
        create_backup(&publish_path)?;
        update_existing_publish_file(&publish_path, &name, &url)?;
    } else {
        create_new_publish_file(&publish_path, &name, &url)?;
    }
    
    Ok(format!("插件「{}」安装成功", name))
}

#[command]
fn uninstall_plugin(plugin_name: Option<String>) -> Result<String, String> {
    let name = plugin_name.unwrap_or_else(|| DEFAULT_PLUGIN_NAME.to_string());
    let publish_path = get_publish_path()?;
    
    if !publish_path.exists() {
        return Err("插件配置文件不存在，可能尚未安装任何插件".to_string());
    }
    
    let content = fs::read_to_string(&publish_path)
        .map_err(|e| format!("读取 publish.xml 失败: {}", e))?;
    
    if !content.contains(&format!(r#"name="{}""#, name)) {
        return Err(format!("未找到插件「{}」", name));
    }
    
    create_backup(&publish_path)?;
    
    let pattern = format!(r#"\s*<jspluginonline[^>]*name="{}"[^>]*/>\s*"#, regex::escape(&name));
    let re = regex::Regex::new(&pattern).map_err(|e| format!("正则表达式错误: {}", e))?;
    let new_content = re.replace_all(&content, "\n").to_string().replace("\n\n", "\n");
    
    fs::write(&publish_path, new_content)
        .map_err(|e| format!("写入 publish.xml 失败: {}", e))?;
    
    Ok(format!("插件「{}」卸载成功", name))
}

#[command]
fn list_plugins() -> Result<Vec<PluginInfo>, String> {
    let publish_path = get_publish_path()?;
    
    if !publish_path.exists() {
        return Ok(vec![]);
    }
    
    let content = fs::read_to_string(&publish_path)
        .map_err(|e| format!("读取 publish.xml 失败: {}", e))?;
    
    let re = regex::Regex::new(r#"<jspluginonline\s+name="([^"]*)"\s+url="([^"]*)"\s+type="([^"]*)""#)
        .map_err(|e| format!("正则表达式错误: {}", e))?;
    
    let plugins: Vec<PluginInfo> = re.captures_iter(&content)
        .map(|cap| PluginInfo {
            name: cap[1].to_string(),
            url: cap[2].to_string(),
            plugin_type: cap[3].to_string(),
        })
        .collect();
    
    Ok(plugins)
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![install_plugin, uninstall_plugin, list_plugins])
        .run(tauri::generate_context!())
        .expect("运行 Tauri 应用时出错");
}
