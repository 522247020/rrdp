use anyhow::Result;
use colored::*;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Password, Select};

use crate::config::{Config, ConnectionConfig};
use crate::connection::ConnectionBuilder;

/// 交互式连接选择器
/// 每次操作后返回菜单循环，直到用户选择退出
pub fn run(config: &mut Config) -> Result<()> {
    loop {
        let connections: Vec<ConnectionConfig> = config.list_connections().to_vec();

        if connections.is_empty() {
            println!("{} 暂无已保存的连接。", "ℹ️ ".blue());
            create_and_connect(config)?;
            *config = Config::load()?;
            continue;
        }

        match select_connection(&connections) {
            None | Some(SelectionResult::Exit) => {
                println!("{} 再见！", "👋".blue());
                return Ok(());
            }
            Some(SelectionResult::Connect(index)) => {
                connect_to(&connections[index])?;
                println!();
            }
            Some(SelectionResult::EditConnection(index)) => {
                edit_connection(config, &connections[index])?;
                *config = Config::load()?;
            }
            Some(SelectionResult::DeleteConnection(index)) => {
                delete_connection(config, &connections[index])?;
                *config = Config::load()?;
            }
            Some(SelectionResult::CreateNew) => {
                create_and_connect(config)?;
                *config = Config::load()?;
            }
        }
    }
}

enum SelectionResult {
    Connect(usize),
    EditConnection(usize),
    DeleteConnection(usize),
    CreateNew,
    Exit,
}

/// 主菜单 — 连接列表 + 操作选项
/// 按 Esc 或选择「退出」均返回 None
fn select_connection(connections: &[ConnectionConfig]) -> Option<SelectionResult> {
    let theme = ColorfulTheme::default();

    // 构建连接列表项
    let mut items: Vec<String> = connections
        .iter()
        .map(|c| {
            let label = format!("{}  [{}]", c.name, c.server);
            if let Some(desc) = &c.description {
                if !desc.is_empty() {
                    return format!("{} — {}", label, desc);
                }
            }
            label
        })
        .collect();

    let edit_index = items.len();
    let delete_index = items.len() + 1;
    let create_new_index = items.len() + 2;
    let exit_index = items.len() + 3;

    items.push("✏️  编辑连接...".to_string());
    items.push("🗑️  删除连接...".to_string());
    items.push("➕  新建连接...".to_string());
    items.push("❌  退出  [Esc]".to_string());

    let selection = Select::with_theme(&theme)
        .with_prompt("请选择连接 (↑↓ 导航, Enter 确认, Esc 退出)")
        .default(0)
        .items(&items)
        .interact_opt()
        .expect("读取选择失败");

    let selection = selection?; // Esc → None → 退出

    match selection {
        _ if selection == exit_index => Some(SelectionResult::Exit),
        _ if selection == create_new_index => Some(SelectionResult::CreateNew),
        _ if selection == edit_index => {
            select_item_from_list(&connections, "请选择要编辑的连接 (Esc 返回)")
                .map(SelectionResult::EditConnection)
        }
        _ if selection == delete_index => {
            select_item_from_list(&connections, "请选择要删除的连接 (Esc 返回)")
                .map(SelectionResult::DeleteConnection)
        }
        _ => Some(SelectionResult::Connect(selection)),
    }
}

/// 从连接列表中选择一项（用于编辑/删除子菜单）
/// Esc → None → 返回上一级
fn select_item_from_list<'a>(connections: &'a [ConnectionConfig], prompt: &str) -> Option<usize> {
    let theme = ColorfulTheme::default();

    let items: Vec<String> = connections
        .iter()
        .map(|c| {
            let label = format!("{}  [{}]", c.name, c.server);
            if let Some(desc) = &c.description {
                if !desc.is_empty() {
                    return format!("{} — {}", label, desc);
                }
            }
            label
        })
        .collect();

    if items.is_empty() {
        return None;
    }

    Select::with_theme(&theme)
        .with_prompt(prompt)
        .default(0)
        .items(&items)
        .interact_opt()
        .expect("读取选择失败")
}

fn connect_to(conn: &ConnectionConfig) -> Result<()> {
    println!("\n{} 已选择: {}", "🔗".cyan(), conn.name.bold());
    println!("  {} 服务器: {}", "🖥️ ".cyan(), conn.server.cyan());
    if let Some(ref user) = conn.username {
        println!("  {} 用户: {}", "👤".cyan(), user);
    }

    let mut builder = ConnectionBuilder::new(&conn.server);

    if let Some(ref user) = conn.username {
        builder.username(user);
    }
    if let Some(ref domain) = conn.domain {
        builder.domain(domain);
    }
    if let Some(ref password) = conn.password {
        builder.password(password);
    } else {
        let password: String = Password::new()
            .with_prompt("密码 (不需要则留空)")
            .allow_empty_password(true)
            .interact()?;

        if !password.is_empty() {
            builder.password(&password);
        }
    }

    // 应用保存的设置
    if let Some(width) = conn.width {
        if let Some(height) = conn.height {
            builder.size(width, height);
        }
    }
    if let Some(fullscreen) = conn.fullscreen {
        builder.fullscreen(fullscreen);
    }
    if let Some(dr) = conn.dynamic_resolution {
        builder.dynamic_resolution(dr);
    }
    if let Some(scale) = conn.scale_desktop {
        builder.scale_desktop(scale);
    }
    if let Some(ss) = conn.smart_sizing {
        builder.smart_sizing(ss);
    }

    println!("\n{} 正在连接到 {}...", "🚀".green(), conn.server.cyan());
    builder.connect()?;

    Ok(())
}

fn edit_connection(config: &Config, conn: &ConnectionConfig) -> Result<()> {
    println!("\n{} 编辑连接: {}", "✏️ ".yellow(), conn.name.bold());
    println!("{} 直接回车保留当前值，清空可选字段可删除。", "💡".blue());
    println!("{}", "─".repeat(40));

    let name: String = Input::new()
        .with_prompt("连接名称")
        .default(conn.name.clone())
        .interact_text()?;

    let server: String = Input::new()
        .with_prompt("服务器地址")
        .default(conn.server.clone())
        .interact_text()?;

    let current_username = conn.username.clone().unwrap_or_default();
    let username: String = Input::new()
        .with_prompt("用户名 (可选，清空删除)")
        .default(current_username)
        .allow_empty(true)
        .interact_text()?;

    let current_domain = conn.domain.clone().unwrap_or_default();
    let domain: String = Input::new()
        .with_prompt("域名 (可选，清空删除)")
        .default(current_domain)
        .allow_empty(true)
        .interact_text()?;

    let password = if Confirm::new()
        .with_prompt("修改保存的密码?")
        .default(false)
        .show_default(true)
        .interact()?
    {
        let password: String = Password::new()
            .with_prompt("新密码 (留空删除已保存密码)")
            .allow_empty_password(true)
            .with_confirmation("确认密码", "两次密码不匹配")
            .interact()?;

        if password.is_empty() {
            None
        } else {
            Some(password)
        }
    } else {
        conn.password.clone()
    };

    let current_desc = conn.description.clone().unwrap_or_default();
    let description: String = Input::new()
        .with_prompt("描述 (可选，清空删除)")
        .default(current_desc)
        .allow_empty(true)
        .interact_text()?;

    // 显示设置
    println!("\n{} 显示设置", "🖥️ ".cyan());
    println!("{}", "─".repeat(40));

    let current_width = conn.width.map(|w| w.to_string()).unwrap_or_default();
    let width_str: String = Input::new()
        .with_prompt("窗口宽度 (留空使用默认 1920)")
        .default(current_width)
        .allow_empty(true)
        .interact_text()?;

    let current_height = conn.height.map(|h| h.to_string()).unwrap_or_default();
    let height_str: String = Input::new()
        .with_prompt("窗口高度 (留空使用默认 1080)")
        .default(current_height)
        .allow_empty(true)
        .interact_text()?;

    let width = width_str.parse::<u32>().ok();
    let height = height_str.parse::<u32>().ok();

    let current_fullscreen = conn.fullscreen.unwrap_or(false);
    let fullscreen = Confirm::new()
        .with_prompt("全屏模式?")
        .default(current_fullscreen)
        .show_default(true)
        .interact()?;

    let current_dynamic_resolution = conn.dynamic_resolution.unwrap_or(false);
    let dynamic_resolution = Confirm::new()
        .with_prompt("允许动态调整窗口大小?")
        .default(current_dynamic_resolution)
        .show_default(true)
        .interact()?;

    let current_scale = conn
        .scale_desktop
        .map(|s| s.to_string())
        .unwrap_or_default();
    let scale_desktop_str: String = Input::new()
        .with_prompt("桌面缩放百分比 (100-500，留空不缩放)")
        .default(current_scale)
        .allow_empty(true)
        .interact_text()?;

    let scale_desktop = scale_desktop_str
        .parse::<u32>()
        .ok()
        .filter(|&s| s >= 100 && s <= 500);

    let current_smart_sizing = conn.smart_sizing.unwrap_or(false);
    let smart_sizing = if dynamic_resolution {
        false
    } else {
        Confirm::new()
            .with_prompt("智能缩放以适应窗口? (与动态调整互斥)")
            .default(current_smart_sizing)
            .show_default(true)
            .interact()?
    };

    // 保存更新后的配置
    let mut updated_config = config.clone();
    updated_config.save_connection(
        &name,
        &server,
        if username.is_empty() {
            None
        } else {
            Some(username)
        },
        password,
        if domain.is_empty() {
            None
        } else {
            Some(domain)
        },
        if description.is_empty() {
            None
        } else {
            Some(description)
        },
        width,
        height,
        if fullscreen { Some(true) } else { None },
        if dynamic_resolution { Some(true) } else { None },
        scale_desktop,
        if smart_sizing { Some(true) } else { None },
    );
    updated_config.save()?;
    println!("{} 连接 '{}' 更新成功！", "✓".green(), name.bold());

    Ok(())
}

fn delete_connection(config: &Config, conn: &ConnectionConfig) -> Result<()> {
    println!("\n{} 将要删除连接: {}", "🗑️ ".red(), conn.name.bold());
    println!("  服务器: {}", conn.server.cyan());
    if let Some(ref user) = conn.username {
        println!("  用户: {}", user);
    }

    let confirmed = Confirm::new()
        .with_prompt("确定要删除此连接吗？")
        .default(false)
        .show_default(true)
        .interact()?;

    if confirmed {
        let mut updated_config = config.clone();
        updated_config.delete_connection(&conn.name);
        updated_config.save()?;
        println!("{} 连接 '{}' 已删除！", "✓".green(), conn.name.bold());
    } else {
        println!("{} 已取消删除。", "ℹ️ ".blue());
    }

    Ok(())
}

fn create_and_connect(config: &Config) -> Result<()> {
    println!("\n{} 新建连接", "🔌".green());
    println!("{}", "─".repeat(40));

    let name: String = Input::new().with_prompt("连接名称").interact_text()?;

    let server: String = Input::new()
        .with_prompt("服务器地址 (如 192.168.1.100)")
        .interact_text()?;

    let username: String = Input::new()
        .with_prompt("用户名 (可选)")
        .default(String::new())
        .show_default(false)
        .allow_empty(true)
        .interact_text()?;

    let domain: String = Input::new()
        .with_prompt("域名 (可选)")
        .default(String::new())
        .show_default(false)
        .allow_empty(true)
        .interact_text()?;

    let description: String = Input::new()
        .with_prompt("描述 (可选)")
        .default(String::new())
        .show_default(false)
        .allow_empty(true)
        .interact_text()?;

    let password: String = Password::new()
        .with_prompt("密码 (可选)")
        .allow_empty_password(true)
        .with_confirmation("确认密码", "两次密码不匹配")
        .interact()?;

    // 显示显示设置选项
    println!("\n{} 显示设置", "🖥️ ".cyan());
    println!("{}", "─".repeat(40));

    let width_str: String = Input::new()
        .with_prompt("窗口宽度 (留空使用默认 1920)")
        .default(String::new())
        .show_default(false)
        .allow_empty(true)
        .interact_text()?;

    let height_str: String = Input::new()
        .with_prompt("窗口高度 (留空使用默认 1080)")
        .default(String::new())
        .show_default(false)
        .allow_empty(true)
        .interact_text()?;

    let width = width_str.parse::<u32>().ok();
    let height = height_str.parse::<u32>().ok();

    let fullscreen = Confirm::new()
        .with_prompt("全屏模式?")
        .default(false)
        .show_default(true)
        .interact()?;

    let dynamic_resolution = Confirm::new()
        .with_prompt("允许动态调整窗口大小?")
        .default(true)
        .show_default(true)
        .interact()?;

    let scale_desktop_str: String = Input::new()
        .with_prompt("桌面缩放百分比 (100-500，留空不缩放)")
        .default(String::new())
        .show_default(false)
        .allow_empty(true)
        .interact_text()?;

    let scale_desktop = scale_desktop_str
        .parse::<u32>()
        .ok()
        .filter(|&s| s >= 100 && s <= 500);

    let smart_sizing = if dynamic_resolution {
        false
    } else {
        Confirm::new()
            .with_prompt("智能缩放以适应窗口? (与动态调整互斥)")
            .default(false)
            .show_default(true)
            .interact()?
    };

    // 保存到配置
    let mut updated_config = config.clone();
    updated_config.save_connection(
        &name,
        &server,
        if username.is_empty() {
            None
        } else {
            Some(username.clone())
        },
        if password.is_empty() {
            None
        } else {
            Some(password.clone())
        },
        if domain.is_empty() {
            None
        } else {
            Some(domain)
        },
        if description.is_empty() {
            None
        } else {
            Some(description)
        },
        width,
        height,
        if fullscreen { Some(true) } else { None },
        if dynamic_resolution { Some(true) } else { None },
        scale_desktop,
        if smart_sizing { Some(true) } else { None },
    );
    updated_config.save()?;
    println!("{} 连接 '{}' 已保存！", "✓".green(), name.bold());

    // 构建并连接
    let mut builder = ConnectionBuilder::new(&server);
    if !username.is_empty() {
        builder.username(&username);
    }
    if !password.is_empty() {
        builder.password(&password);
    }

    // 应用显示设置
    if let (Some(w), Some(h)) = (width, height) {
        builder.size(w, h);
    }
    builder.fullscreen(fullscreen);
    builder.dynamic_resolution(dynamic_resolution);
    builder.smart_sizing(smart_sizing);
    if let Some(scale) = scale_desktop {
        builder.scale_desktop(scale);
    }

    println!("{} 正在连接到 {}...", "🚀".green(), server.cyan());
    builder.connect()?;

    Ok(())
}
