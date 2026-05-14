#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::fs;
use std::path::PathBuf;
use tauri::command;
use serde::Deserialize;

const PLUGIN_NAME: &str = "jaidoc-wps";
const PUBLISH_FILE: &str = "publish.xml";

#[derive(Deserialize)]
struct InstallRequest {
    domain: String,
    protocol: String,
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

fn generate_plugin_xml(domain: &str, protocol: &str) -> String {
    let url = format!("{}://{}/jaidoc-wps/", protocol, domain);
    format!(
        r#"<jspluginonline name="{}" url="{}" type="wps" install="{}"/>"#,
        PLUGIN_NAME, url, url
    )
}

fn create_new_publish_file(publish_path: &PathBuf, domain: &str, protocol: &str) -> Result<(), String> {
    let plugin_xml = generate_plugin_xml(domain, protocol);
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

fn update_existing_publish_file(publish_path: &PathBuf, domain: &str, protocol: &str) -> Result<(), String> {
    let content = fs::read_to_string(publish_path)
        .map_err(|e| format!("读取 publish.xml 失败: {}", e))?;
    
    let plugin_xml = generate_plugin_xml(domain, protocol);
    let new_content: String;
    
    if content.contains(&format!(r#"name="{}""#, PLUGIN_NAME)) {
        // 更新现有插件
        let pattern = format!(r#"<jspluginonline[^>]*name="{}"[^>]*/>"#, PLUGIN_NAME);
        let re = regex::Regex::new(&pattern).map_err(|e| format!("正则表达式错误: {}", e))?;
        new_content = re.replace(&content, &plugin_xml).to_string();
    } else if content.contains("</jsplugins>") {
        // 添加新插件
        new_content = content.replace("</jsplugins>", &format!("  {}\n</jsplugins>", plugin_xml));
    } else {
        // 文件格式不正确，重新创建
        return create_new_publish_file(publish_path, domain, protocol);
    }
    
    fs::write(publish_path, new_content)
        .map_err(|e| format!("写入 publish.xml 失败: {}", e))?;
    Ok(())
}

#[command]
fn install_plugin(domain: String, protocol: String) -> Result<String, String> {
    // 验证域名格式
    if domain.is_empty() {
        return Err("域名不能为空".to_string());
    }
    
    // 验证协议
    let protocol = protocol.to_lowercase();
    if protocol != "http" && protocol != "https" {
        return Err("协议必须是 http 或 https".to_string());
    }
    
    // 移除可能的协议前缀和路径
    let domain = domain
        .replace("https://", "")
        .replace("http://", "")
        .replace("/jaidoc-wps/", "")
        .trim_end_matches('/')
        .to_string();
    
    if domain.is_empty() {
        return Err("域名格式不正确".to_string());
    }
    
    let publish_path = get_publish_path()?;
    
    if publish_path.exists() {
        // 备份原文件
        create_backup(&publish_path)?;
        // 更新现有文件
        update_existing_publish_file(&publish_path, &domain, &protocol)?;
        Ok(format!("✓ 插件安装成功！\n协议: {}\n域名: {}\n配置文件: {:?}", protocol, domain, publish_path))
    } else {
        // 创建新文件
        create_new_publish_file(&publish_path, &domain, &protocol)?;
        Ok(format!("✓ 插件安装成功！\n协议: {}\n域名: {}\n配置文件: {:?}", protocol, domain, publish_path))
    }
}

#[command]
fn uninstall_plugin() -> Result<String, String> {
    let publish_path = get_publish_path()?;
    
    if !publish_path.exists() {
        return Err("插件配置文件不存在，可能尚未安装".to_string());
    }
    
    let content = fs::read_to_string(&publish_path)
        .map_err(|e| format!("读取 publish.xml 失败: {}", e))?;
    
    if !content.contains(&format!(r#"name="{}""#, PLUGIN_NAME)) {
        return Err("未找到 jaidoc-wps 插件".to_string());
    }
    
    // 备份原文件
    create_backup(&publish_path)?;
    
    // 移除插件配置
    let pattern = format!(r#"\s*<jspluginonline[^>]*name="{}"[^>]*/>\s*"#, PLUGIN_NAME);
    let re = regex::Regex::new(&pattern).map_err(|e| format!("正则表达式错误: {}", e))?;
    let new_content = re.replace_all(&content, "\n").to_string();
    
    // 清理多余的空行
    let new_content = new_content.replace("\n\n", "\n");
    
    fs::write(&publish_path, new_content)
        .map_err(|e| format!("写入 publish.xml 失败: {}", e))?;
    
    Ok(format!("✓ 插件卸载成功！\n配置文件: {:?}", publish_path))
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![install_plugin, uninstall_plugin])
        .run(tauri::generate_context!())
        .expect("运行 Tauri 应用时出错");
}
