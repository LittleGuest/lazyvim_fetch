use std::{fs, process};

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

/// LazyVim 安装更新脚本
///
/// 使用git下载LazyVim，使用前确保git已安装
#[derive(Debug, Parser, Clone, Copy)]
#[command(author,version,about,long_about=None)]
enum App {
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

fn main() {
    let app = App::parse();
    let config = LazyVim::new();

    match app {
        App::Install => config.install(),
        App::Update => config.update(),
        App::Delete => config.delete(),
    };
}

#[derive(Debug, Deserialize)]
struct LazyVim {
    starter: String,
    plugins: Vec<String>,
}

impl LazyVim {
    fn new() -> Self {
        let config = fs::read_to_string("../lazyvim.toml").unwrap();
        toml_edit::de::from_str::<Self>(&config).unwrap()
    }

    fn install(&self) {
        // starter
        let output = process::Command::new("git")
            .arg("clone")
            .arg(&self.starter)
            .arg(NVIM_PATH)
            .output()
            .unwrap();
        let stdout = String::from_utf8(output.stdout).unwrap();
        if stdout.contains("done") {}

        // plugins
        for ele in self.plugins.iter() {
            let output = process::Command::new("git")
                .arg("clone")
                .arg(ele)
                .arg(PLUGIN_PATH)
                .output()
                .unwrap();
            let stdout = String::from_utf8(output.stdout).unwrap();
            if stdout.contains("done") {}
        }
    }

    fn update(&self) {
        // check nvim path exist
        // check plugins path exist
        todo!()
    }

    fn delete(&self) {
        fs::remove_dir_all(NVIM_PATH).unwrap();
        fs::remove_dir_all(PLUGIN_PATH).unwrap();
    }
}

// impl std::fmt::Display for Config {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         writeln!(f, "starter: {}", self.starter);
//         for ele in self.plugins {
//             writeln!(f, "{}", ele);
//         }
//     }
// }

#[cfg(test)]
mod test {
    #[test]
    fn test_read_dir() {
        let entrys = std::fs::read_dir(".").unwrap();
        for entry in entrys {
            let dir = entry.unwrap();
            println!("{:?}", dir.path());
        }
    }
}
