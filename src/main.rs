use std::{
    env,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
};

use clap::Parser;

#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    path: String,
}

fn get_current_working_dir() -> std::io::Result<PathBuf> {
    env::current_dir()
}

fn get_file_path(path: &Path, end_path: &Path, target_names: &Vec<String>) -> Option<PathBuf> {
    for target_name in target_names {
        let target = path.join(target_name);
        if target.exists() {
            return Some(target);
        }
    }
    if path.eq(end_path) {
        return None;
    }
    let new_path = path.parent().unwrap();
    return get_file_path(new_path, end_path, target_names);
}

fn format_file(child: &mut Child) -> std::result::Result<(), std::io::Error> {
    let mut buffer = Vec::new();
    io::stdin().read_to_end(&mut buffer).unwrap();
    child.stdin.take().unwrap().write(&buffer).unwrap();
    let err = child.wait().err();
    if err.is_some() {
        return Err(err.unwrap());
    }
    return Ok(());
}

fn main() {
    let cli = Cli::parse();
    let current_dir = get_current_working_dir().unwrap();
    let home_dir = home::home_dir().unwrap();
    let biome_config = get_file_path(&current_dir, &home_dir, &vec!["biome.json".to_string()]);
    match biome_config {
        Some(path) => {
            let biome_path = path;
            let mut biome_dir = biome_path.clone();
            biome_dir.pop();
            let mut output = Command::new(&biome_dir.join("node_modules/.bin/biome"))
                .args(["format", "--stdin-file-path", &cli.path])
                .stdin(Stdio::piped())
                .stdout(Stdio::inherit())
                .spawn()
                .expect("failed to execute process");

            format_file(&mut output).unwrap();
        }
        None => {
            let prettier_config_names: Vec<String> = [
                ".prettierrc",
                ".prettierrc.json",
                ".prettierrc.cjs",
                ".prettierrc.js",
            ]
            .iter()
            .map(|&s| s.into())
            .collect();
            let prettier_config = get_file_path(&current_dir, &home_dir, &prettier_config_names);
            if prettier_config.is_some() {
                let prettier_path = prettier_config.unwrap();
                let mut prettier_dir = prettier_path.clone();
                prettier_dir.pop();
                let mut output = Command::new(&prettier_dir.join("node_modules/.bin/prettier"))
                    .args(["--stdin-filepath", &cli.path])
                    .stdin(Stdio::piped())
                    .stdout(Stdio::inherit())
                    .spawn()
                    .expect("failed to execute process");

                format_file(&mut output).unwrap();
            }
        }
    }
}
