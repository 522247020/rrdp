use clap::Parser;
use clap::Subcommand;
use colored::*;
use std::process::Command;

mod config;
mod connection;

use config::Config;
use connection::ConnectionBuilder;

mod interactive;

/// RRDP - Rust Remote Desktop Protocol 客户端
/// 一个简单的 xfreerdp3 包装工具
#[derive(Parser)]
#[command(name = "rrdp")]
#[command(about = "xfreerdp3 的简洁命令行包装工具", long_about = None)]
struct Cli {
    /// 显示版本信息
    #[arg(short = 'v', long = "version")]
    version: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// 连接到远程桌面
    Connect {
        /// 服务器地址 (如 192.168.1.100)
        server: String,

        /// 登录用户名
        #[arg(short, long)]
        username: Option<String>,

        /// 登录密码
        #[arg(short, long)]
        password: Option<String>,

        /// 域
        #[arg(short, long)]
        domain: Option<String>,

        /// 远程桌面宽度 (默认: 1920)
        #[arg(long, default_value = "1920")]
        width: u32,

        /// 远程桌面高度 (默认: 1080)
        #[arg(long, default_value = "1080")]
        height: u32,

        /// 全屏模式
        #[arg(short, long)]
        fullscreen: bool,

        /// 启用剪贴板共享
        #[arg(long)]
        clipboard: bool,

        /// 启用驱动器重定向
        #[arg(long)]
        drive: Option<String>,

        /// 启用音频输出
        #[arg(long)]
        audio: bool,

        /// 网络级身份验证 (NLA)
        #[arg(long)]
        nla: bool,

        /// TLS 安全连接
        #[arg(long)]
        tls: bool,

        /// 允许动态调整窗口大小
        #[arg(long)]
        dynamic_resolution: bool,

        /// 桌面缩放百分比 (100-500)
        #[arg(long)]
        scale_desktop: Option<u32>,

        /// 智能缩放以适应窗口
        #[arg(long)]
        smart_sizing: bool,

        /// 附加的 xfreerdp3 参数
        #[arg(last = true)]
        extra_args: Vec<String>,
    },

    /// 列出已保存的连接
    List,

    /// 从已保存列表中选择一个连接（交互模式）
    Select,

    /// 保存连接配置
    Save {
        /// 连接名称
        name: String,

        /// 服务器地址
        #[arg(short, long)]
        server: String,

        /// 登录用户名
        #[arg(short, long)]
        username: Option<String>,

        /// 登录密码
        #[arg(short, long)]
        password: Option<String>,

        /// 域
        #[arg(short, long)]
        domain: Option<String>,

        /// 连接描述
        #[arg(short, long)]
        description: Option<String>,
    },

    /// 加载已保存的配置并连接
    Load {
        /// 连接名称
        name: String,

        /// 密码（如果未保存）
        #[arg(short, long)]
        password: Option<String>,
    },

    /// 删除已保存的连接
    Delete {
        /// 要删除的连接名称
        name: String,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    if cli.version {
        println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    // Check if xfreerdp3 is installed
    if !is_xfreerdp3_installed() {
        eprintln!("{} 未安装 xfreerdp3！", "Error:".red().bold());
        eprintln!("请使用以下命令安装: sudo pacman -S freerdp");
        std::process::exit(1);
    }

    let mut config = Config::load()?;

    match &cli.command {
        None | Some(Commands::Select) => {
            interactive::run(&mut config)?;
        }
        Some(Commands::Connect {
            server,
            username,
            password,
            domain,
            width,
            height,
            fullscreen,
            clipboard,
            drive,
            audio,
            nla,
            tls,
            dynamic_resolution,
            scale_desktop,
            smart_sizing,
            extra_args,
        }) => {
            let mut builder = ConnectionBuilder::new(server);

            if let Some(user) = username {
                builder.username(user);
            }
            if let Some(pass) = password {
                builder.password(pass);
            }
            if let Some(dom) = domain {
                builder.domain(dom);
            }

            builder
                .size(*width, *height)
                .fullscreen(*fullscreen)
                .clipboard(*clipboard)
                .audio(*audio)
                .nla(*nla)
                .tls(*tls)
                .dynamic_resolution(*dynamic_resolution)
                .smart_sizing(*smart_sizing);

            if let Some(drive_path) = drive {
                builder.drive(drive_path);
            }

            if let Some(scale) = scale_desktop {
                builder.scale_desktop(*scale);
            }

            builder.extra_args(extra_args.clone());

            println!("{} 正在连接到 {}...", "🚀".green(), server.cyan());
            builder.connect()?;
        }

        Some(Commands::List) => {
            let mut connections = config.list_connections().to_vec();
            connections.sort_by_cached_key(|conn| conn.name.to_lowercase());
            if connections.is_empty() {
                println!("{} 暂无已保存的连接。", "ℹ️ ".blue());
            } else {
                println!("{} 已保存的连接:", "📋".green());
                for conn in &connections {
                    println!("  {} - {}", conn.name.bold(), conn.server);
                    if let Some(desc) = &conn.description {
                        println!("    描述: {}", desc);
                    }
                    if let Some(user) = &conn.username {
                        println!("    用户: {}", user);
                    }
                    if let Some(domain) = &conn.domain {
                        println!("    域: {}", domain);
                    }
                    println!();
                }
            }
        }

        Some(Commands::Save {
            name,
            server,
            username,
            password,
            domain,
            description,
        }) => {
            let mut updated_config = config.clone();
            updated_config.save_connection(
                name,
                server,
                username.clone(),
                password.clone(),
                domain.clone(),
                description.clone(),
                None,
                None,
                None,
                None,
                None,
                None,
            );
            updated_config.save()?;
            println!("{} 连接 '{}' 保存成功！", "✓".green(), name.bold());
        }

        Some(Commands::Load { name, password }) => match config.get_connection(name) {
            Some(conn) => {
                let mut builder = ConnectionBuilder::new(&conn.server);

                if let Some(user) = &conn.username {
                    builder.username(user);
                }
                if let Some(dom) = &conn.domain {
                    builder.domain(dom);
                }
                if let Some(pass) = password.as_ref().or(conn.password.as_ref()) {
                    builder.password(pass);
                }

                println!("{} 正在加载连接: {}", "🔄".blue(), name.bold());
                builder.connect()?;
            }
            None => {
                eprintln!("{} 连接 '{}' 未找到！", "Error:".red().bold(), name);
            }
        },

        Some(Commands::Delete { name }) => {
            let mut updated_config = config.clone();
            if updated_config.delete_connection(name) {
                updated_config.save()?;
                println!("{} 连接 '{}' 已删除！", "🗑️ ".red(), name.bold());
            } else {
                eprintln!("{} 连接 '{}' 未找到！", "Error:".red().bold(), name);
            }
        }
    }

    Ok(())
}

/// 判断系统中是否存在 xfreerdp3
fn is_xfreerdp3_installed() -> bool {
    Command::new("which")
        .arg("xfreerdp3")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
        || Command::new("which")
            .arg("xfreerdp")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
}
