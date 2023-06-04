use std::{fs, process::Command, task::Poll};

use clap::Parser;
use serde::Deserialize;

/// Neovim安装路径
#[cfg(windows)]
const NVIM_PATH: &str = "~/AppData/Local";
#[cfg(unix)]
const NVIM_PATH: &str = "~/.config/nvim/";

/// plugin 安装路径
#[cfg(unix)]
const PLUGIN_PATH: &str = "~/.local/share/nvim/";
#[cfg(windows)]
const PLUGIN_PATH: &str = "~/AppData/Local/nvim-data/";

#[cfg(unix)]
const CACHE_PATH: &str = "~/.cache/nvim";
#[cfg(unix)]
const STATE_PATH: &str = "~/.local/state/nvim";

fn main() {
    simple_logger::SimpleLogger::new().env().init().unwrap();

    let opt = Opt::parse();
    let app = App::new();

    let mut task = Vec::new();
    task.push(NeovimPlugin::new(&app.starter, NVIM_PATH));

    app.plugins.iter().for_each(|p| {
        task.push(NeovimPlugin::new(p, PLUGIN_PATH));
    });

    log::info!("总共有{}个插件", task.len());

    match opt {
        Opt::Install | Opt::Update => {
            futures::executor::block_on(async {
                // join_all futures::future:join_all 按顺序运行
                let _ = futures::future::join_all(task).await;
            });
            log::info!("安装结束");
        }
        Opt::Delete => {
            app.delete();
        }
    };
}

/// LazyVim 安装更新脚本
///
/// 使用git下载LazyVim，使用前确保git已安装
#[derive(Debug, Parser, Clone, Copy)]
#[command(author,version,about,long_about=None)]
enum Opt {
    /// 安装
    /// windows 安装到
    /// ~/AppData/Local/nvim
    /// ~/AppData/Local/nvim-data
    /// Linux 安装到
    /// ~/.config/nvim/
    /// ~/.local/share/nvim/lazy/
    Install,
    /// 更新starter和plugins
    Update,
    /// 删除starter和plugins
    Delete,
}

#[derive(Debug, Deserialize)]
struct App {
    starter: String,
    plugins: Vec<String>,
}

impl App {
    fn new() -> Self {
        let config: String =
            fs::read_to_string("./lazyvim.toml").expect("lazyvim.toml config file not exists");
        toml_edit::de::from_str::<Self>(&config).expect("read lazyvim.toml failed")
    }

    fn delete(&self) {
        fs::remove_dir_all(NVIM_PATH).unwrap();
        fs::remove_dir_all(PLUGIN_PATH).unwrap();
        fs::remove_dir_all(CACHE_PATH).unwrap();
        fs::remove_dir_all(STATE_PATH).unwrap();
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct NeovimPlugin {
    git_url: String,
    install_path: String,
}

impl std::future::Future for NeovimPlugin {
    type Output = ();

    fn poll(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let Some(name) = self.plugin_name() else {
            log::error!("{} 插件名称为空,跳过下载",self.git_url);
            return Poll::Ready(());
        };

        // 先删除
        let _ = fs::remove_dir_all(format!("{}/{}", &self.install_path, name));

        let mut git = Command::new("git");
        git.arg("clone")
            .arg(&self.git_url)
            .arg(format!("{}/{}", &self.install_path, name))
            .arg("--depth")
            .arg("1");
        log::info!(
            "开始下载插件: {name} ==> {:?} {:?}",
            git.get_program(),
            git.get_args().collect::<Vec<_>>()
        );
        let output: Result<std::process::Output, std::io::Error> = git.output();
        match output {
            Ok(output) => {
                log::debug!("{output:?}");

                if output.status.success() {
                    log::info!("下载插件{name}成功");
                    Poll::Ready(())
                } else {
                    let stderr = String::from_utf8(output.stderr).unwrap();
                    log::error!("安装插件{name}失败，原因是: {stderr}，稍后重新安装");
                    Poll::Pending
                }
            }
            Err(e) => {
                log::error!("安装插件{name}失败，原因是: {e}，稍后重新安装");
                Poll::Pending
            }
        }
    }
}

impl NeovimPlugin {
    pub fn new(git_url: &str, install_path: &str) -> Self {
        Self {
            git_url: git_url.into(),
            install_path: install_path.into(),
        }
    }

    /// 获取插件名称
    /// 例如
    /// https://github.com/neovim/nvim-lspconfig.git
    /// 插件名称为 nvim-lspconfig
    fn plugin_name(&self) -> Option<String> {
        if let Some((_, name)) = self.git_url.rsplit_once('/') {
            if name.contains(".git") {
                return Some(name.replace(".git", ""));
            }
            Some(name.into())
        } else {
            None
        }
    }
}
