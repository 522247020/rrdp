use anyhow::{Context, Result};
use colored::*;
use std::process::{Command, Stdio};

pub struct ConnectionBuilder {
    server: String,
    username: Option<String>,
    password: Option<String>,
    domain: Option<String>,
    width: u32,
    height: u32,
    fullscreen: bool,
    clipboard: bool,
    drive: Option<String>,
    audio: bool,
    nla: bool,
    tls: bool,
    extra_args: Vec<String>,
}

impl ConnectionBuilder {
    pub fn new(server: &str) -> Self {
        ConnectionBuilder {
            server: server.to_string(),
            username: None,
            password: None,
            domain: None,
            width: 1920,
            height: 1080,
            fullscreen: false,
            clipboard: false,
            drive: None,
            audio: false,
            nla: false,
            tls: false,
            extra_args: Vec::new(),
        }
    }

    pub fn username(&mut self, username: &str) -> &mut Self {
        self.username = Some(username.to_string());
        self
    }

    pub fn password(&mut self, password: &str) -> &mut Self {
        self.password = Some(password.to_string());
        self
    }

    pub fn domain(&mut self, domain: &str) -> &mut Self {
        self.domain = Some(domain.to_string());
        self
    }

    pub fn size(&mut self, width: u32, height: u32) -> &mut Self {
        self.width = width;
        self.height = height;
        self
    }

    pub fn fullscreen(&mut self, fs: bool) -> &mut Self {
        self.fullscreen = fs;
        self
    }

    pub fn clipboard(&mut self, clipboard: bool) -> &mut Self {
        self.clipboard = clipboard;
        self
    }

    pub fn drive(&mut self, path: &str) -> &mut Self {
        self.drive = Some(path.to_string());
        self
    }

    pub fn audio(&mut self, audio: bool) -> &mut Self {
        self.audio = audio;
        self
    }

    pub fn nla(&mut self, nla: bool) -> &mut Self {
        self.nla = nla;
        self
    }

    pub fn tls(&mut self, tls: bool) -> &mut Self {
        self.tls = tls;
        self
    }

    pub fn extra_args(&mut self, args: Vec<String>) -> &mut Self {
        self.extra_args = args;
        self
    }

    pub fn connect(&self) -> Result<()> {
        let freerdp_binary = self.find_freerdp_binary()?;
        let args = self.build_args();

        println!(
            "{} 执行: {} {}",
            "▶️ ".blue(),
            freerdp_binary.cyan(),
            args.join(" ").yellow()
        );

        let status = Command::new(&freerdp_binary)
            .args(&args)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .context("无法执行 xfreerdp3")?;

        if !status.success() {
            let exit_code = status.code().unwrap_or(-1);
            let hint = freerdp_exit_code_hint(exit_code);
            eprintln!(
                "{} 连接失败，退出码: {}",
                "Error:".red().bold(),
                exit_code
            );
            eprintln!("  → {}", hint.cyan());
            if exit_code == 24 {
                eprintln!(
                    "  {}",
                    "提示: 请检查用户名/密码是否正确，或尝试调整 --nla / --tls 参数".yellow()
                );
            }
        }

        Ok(())
    }

    fn find_freerdp_binary(&self) -> Result<String> {
        // Try xfreerdp3 first, then fall back to xfreerdp
        for binary in &["xfreerdp3", "xfreerdp"] {
            if Command::new("which")
                .arg(binary)
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
            {
                return Ok(binary.to_string());
            }
        }

        anyhow::bail!("系统中未找到 xfreerdp3 或 xfreerdp，请先安装: sudo pacman -S freerdp")
    }

    fn build_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        // Server address
        args.push(format!("/v:{}", self.server));

        // Username
        if let Some(ref user) = self.username {
            args.push(format!("/u:{}", user));
        }

        // Password
        if let Some(ref pass) = self.password {
            args.push(format!("/p:{}", pass));
        }

        // Domain — 跳过空值或 "."，FreeRDP 不接受 /d:.
        if let Some(ref domain) = self.domain {
            let trimmed = domain.trim();
            if !trimmed.is_empty() && trimmed != "." {
                args.push(format!("/d:{}", trimmed));
            }
        }

        // Window size or fullscreen
        if self.fullscreen {
            args.push("/f".to_string());
        } else {
            args.push(format!("/w:{}", self.width));
            args.push(format!("/h:{}", self.height));
        }

        // Clipboard
        if self.clipboard {
            args.push("+clipboard".to_string());
        }

        // Drive redirection
        if let Some(ref drive_path) = self.drive {
            args.push(format!("/drive:home,{}", drive_path));
        }

        // Audio
        if self.audio {
            args.push("/audio-mode:0".to_string());
            args.push("/sound:sys:pulse".to_string());
        }

        // Security
        if self.nla {
            args.push("+nla".to_string());
        }

        if self.tls {
            args.push("+tls".to_string());
        }

        // Network
        args.push("+auto-reconnect".to_string());

        // Additional arguments
        args.extend(self.extra_args.clone());

        args
    }
}

/// FreeRDP 退出码中文释义
fn freerdp_exit_code_hint(code: i32) -> &'static str {
    match code {
        0 => "成功",
        1 => "未指定错误",
        2 => "内存不足",
        3 => "协议错误",
        4 => "参数无效",
        5 => "访问被拒绝",
        6 => "连接被拒绝",
        7 => "会话 ID 不匹配",
        8 => "权限不足",
        9 => "需要身份验证",
        10 => "客户端错误",
        11 => "内部错误",
        12 => "连接超时",
        13 => "DNS 错误",
        14 => "DNS 名称未找到",
        15 => "地址已被使用",
        16 => "连接中断",
        17 => "不支持的操作",
        18 => "TLS 错误 — 请检查证书或 TLS 设置",
        19 => "会话已过期",
        20 => "正在注销",
        21 => "密码已过期",
        22 => "密码需要修改",
        23 => "密码太短",
        24 => "身份验证失败 — 用户名或密码错误",
        25 => "会话已终止",
        26 => "服务器拒绝连接",
        27 => "许可证错误",
        28 => "重定向错误",
        29 => "通道错误",
        30 => "图形错误",
        31 => "音频错误",
        32 => "剪贴板错误",
        33 => "设备错误",
        34 => "代理错误",
        35 => "身份验证方式不匹配",
        _ => "未知错误",
    }
}
