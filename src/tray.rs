use crate::config::{Config, ConnectionConfig};
use crate::connection::ConnectionBuilder;
use ksni::blocking::TrayMethods;
use ksni::menu::{MenuItem, StandardItem, SubMenu};
use ksni::Tray;
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Default)]
pub struct RrdpTray;

impl Tray for RrdpTray {
    fn id(&self) -> String {
        "rrdp".into()
    }

    fn title(&self) -> String {
        "RRDP 远程桌面".into()
    }

    fn icon_name(&self) -> String {
        "computer".into()
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        let connections = Config::load()
            .map(|config| sorted_connections(&config))
            .unwrap_or_default();
        let mut items = Vec::new();

        if connections.is_empty() {
            items.push(
                StandardItem {
                    label: "暂无已保存的主机".into(),
                    enabled: false,
                    ..Default::default()
                }
                .into(),
            );
        } else {
            let host_items = connections
                .iter()
                .map(|connection| host_menu_item(connection))
                .collect();
            items.push(
                SubMenu {
                    label: "连接主机".into(),
                    submenu: host_items,
                    ..Default::default()
                }
                .into(),
            );
        }

        items.push(MenuItem::Separator);
        items.push(command_item("管理主机...", "select"));
        items.push(command_item("添加主机...", "add"));
        items.push(refresh_item());
        items.push(MenuItem::Separator);
        items.push(
            StandardItem {
                label: "退出托盘".into(),
                icon_name: "application-exit".into(),
                activate: Box::new(|_| std::process::exit(0)),
                ..Default::default()
            }
            .into(),
        );
        items
    }
}

fn sorted_connections(config: &Config) -> Vec<ConnectionConfig> {
    let mut connections = config.list_connections().to_vec();
    connections.sort_by_cached_key(|connection| connection.name.to_lowercase());
    connections
}

fn host_menu_item(connection: &ConnectionConfig) -> MenuItem<RrdpTray> {
    let connection = connection.clone();
    StandardItem {
        label: format!("{}  [{}]", connection.name, connection.server),
        icon_name: "network-server".into(),
        activate: Box::new(move |_| connect_from_tray(connection.clone())),
        ..Default::default()
    }
    .into()
}

fn connect_from_tray(connection: ConnectionConfig) {
    thread::spawn(move || {
        if connection.password.is_none() {
            notify(
                "critical",
                "RRDP 无法连接",
                &format!(
                    "主机“{}”没有保存密码，请先在管理主机中保存密码。",
                    connection.name
                ),
                8000,
            );
            return;
        }

        notify(
            "normal",
            "RRDP 正在连接",
            &format!("正在连接主机“{}”...", connection.name),
            10000,
        );

        let mut child = match ConnectionBuilder::from_config(&connection).spawn_silent() {
            Ok(child) => child,
            Err(error) => {
                notify(
                    "critical",
                    "RRDP 连接失败",
                    &format!("主机“{}”：{}", connection.name, error),
                    10000,
                );
                return;
            }
        };

        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            match child.try_wait() {
                Ok(Some(status)) => {
                    let code = status.code().unwrap_or(-1);
                    let (urgency, summary, message) = exit_notification(&connection.name, code);
                    notify(urgency, summary, &message, 12000);
                    return;
                }
                Ok(None) if Instant::now() >= deadline => {
                    notify(
                        "normal",
                        "RRDP 连接成功",
                        &format!("主机“{}”的远程桌面已启动。", connection.name),
                        4000,
                    );
                    return;
                }
                Ok(None) => thread::sleep(Duration::from_millis(200)),
                Err(error) => {
                    notify("critical", "RRDP 状态检查失败", &error.to_string(), 10000);
                    return;
                }
            }
        }
    });
}

fn notify(urgency: &str, summary: &str, body: &str, timeout_ms: u32) {
    let _ = Command::new("notify-send")
        .args([
            "--app-name=RRDP",
            "--icon=network-server",
            &format!("--urgency={}", urgency),
            &format!("--expire-time={}", timeout_ms),
            summary,
            body,
        ])
        .spawn();
}

fn exit_notification(host: &str, code: i32) -> (&'static str, &'static str, String) {
    if code == 0 {
        return (
            "low",
            "RRDP 远程桌面已关闭",
            format!("已关闭主机“{}”的远程桌面。", host),
        );
    }

    let reason = freerdp_exit_code_hint(code);
    (
        "critical",
        "RRDP 连接已中断",
        format!("主机“{}”：{}", host, reason),
    )
}

fn freerdp_exit_code_hint(code: i32) -> String {
    match code {
        3 => "协议错误".into(),
        4 => "参数无效".into(),
        5 | 6 => "连接被拒绝".into(),
        8 => "权限不足".into(),
        9 => "需要身份验证".into(),
        12 => "连接超时".into(),
        13 | 14 => "DNS 名称无法解析".into(),
        16 => "连接中断".into(),
        18 => "TLS 或证书错误".into(),
        21 | 22 => "密码已过期或需要修改".into(),
        24 => "用户名或密码错误".into(),
        26 => "服务器拒绝连接".into(),
        128..=255 => {
            let signal = code - 128;
            let name = signal_name(signal);
            format!("进程收到 {}（信号 {}）后结束", name, signal)
        }
        _ => format!("FreeRDP 异常退出（代码 {}）", code),
    }
}

fn signal_name(signal: i32) -> &'static str {
    match signal {
        1 => "SIGHUP",
        2 => "SIGINT",
        6 => "SIGABRT",
        9 => "SIGKILL",
        11 => "SIGSEGV",
        13 => "SIGPIPE",
        15 => "SIGTERM",
        _ => "未知信号",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_exit_is_reported_as_closed() {
        let (urgency, summary, body) = exit_notification("conf", 0);
        assert_eq!(urgency, "low");
        assert_eq!(summary, "RRDP 远程桌面已关闭");
        assert_eq!(body, "已关闭主机“conf”的远程桌面。");
    }

    #[test]
    fn signal_exit_is_human_readable() {
        assert_eq!(
            freerdp_exit_code_hint(141),
            "进程收到 SIGPIPE（信号 13）后结束"
        );
    }
}

fn refresh_item() -> MenuItem<RrdpTray> {
    StandardItem {
        label: "主机列表打开菜单时自动刷新".into(),
        enabled: false,
        ..Default::default()
    }
    .into()
}

fn command_item(label: &str, command: &str) -> MenuItem<RrdpTray> {
    let label = label.to_string();
    let command = command.to_string();
    StandardItem {
        label,
        activate: Box::new(move |_| launch_in_terminal(&[&command])),
        ..Default::default()
    }
    .into()
}

fn launch_rrdp(args: &[&str]) {
    let Ok(executable) = std::env::current_exe() else {
        return;
    };
    let _ = Command::new(executable).args(args).spawn();
}

fn launch_in_terminal(args: &[&str]) {
    let Ok(executable) = std::env::current_exe() else {
        return;
    };

    // footclient only connects to an existing foot server. Use the standalone
    // foot binary for tray actions so this also works from a graphical login.
    let configured_terminal = std::env::var("TERMINAL").ok();
    let configured_terminal = configured_terminal
        .as_deref()
        .filter(|terminal| !terminal.ends_with("/footclient") && *terminal != "footclient");

    let mut candidates = Vec::new();
    if configured_terminal.is_some() {
        candidates.push((configured_terminal.unwrap().to_string(), "direct"));
    }
    candidates.extend([
        ("foot".to_string(), "direct"),
        ("x-terminal-emulator".to_string(), "execute"),
        ("gnome-terminal".to_string(), "separator"),
        ("konsole".to_string(), "execute"),
        ("alacritty".to_string(), "execute"),
    ]);

    for (terminal, mode) in candidates {
        if !is_available(&terminal) {
            continue;
        }

        let mut terminal_args = Vec::new();
        match mode {
            "direct" => {}
            "execute" => terminal_args.push("-e".to_string()),
            "separator" => terminal_args.push("--".to_string()),
            _ => continue,
        }
        terminal_args.push(executable.to_string_lossy().into_owned());
        terminal_args.extend(args.iter().map(|arg| (*arg).to_string()));

        if Command::new(&terminal).args(terminal_args).spawn().is_ok() {
            return;
        }
    }

    // A graphical session without a known terminal can still use the command.
    launch_rrdp(args);
}

fn is_available(binary: &str) -> bool {
    Command::new("which")
        .arg(binary)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

pub fn start_background() -> anyhow::Result<()> {
    let executable = std::env::current_exe()?;
    Command::new(executable)
        .arg("tray-worker")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()?;
    Ok(())
}

pub fn run() -> anyhow::Result<()> {
    let _handle = RrdpTray::default()
        .spawn()
        .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    loop {
        std::thread::park();
    }
}
