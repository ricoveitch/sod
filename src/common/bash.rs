use std::collections::HashSet;
use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process;

fn read_dir(commands: &mut HashSet<String>, dir: &str) {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    for result in entries {
        let entry = match result {
            Ok(result) => result,
            Err(_) => continue,
        };

        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };

        let filename = match entry.file_name().into_string() {
            Ok(name) => name,
            Err(_) => continue,
        };

        let is_executable = metadata.permissions().mode() & 0o111 != 0;
        if metadata.is_file() && is_executable {
            commands.insert(filename);
        }
    }
}

pub fn get_commands() -> HashSet<String> {
    let path_str = match env::var("PATH") {
        Ok(p) => p,
        Err(_) => return HashSet::new(),
    };

    let directories: Vec<&str> = path_str.split(":").collect();
    let mut commands = HashSet::new();

    for directory in directories.iter().rev() {
        read_dir(&mut commands, &directory);
    }

    commands
}

pub fn run_cmd(cmd: &str) -> String {
    let output = process::Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .expect("failed to execute process");

    if let Ok(s) = String::from_utf8(output.stderr) {
        if s.len() > 0 {
            eprintln!("{}", s)
        }
    }

    let out_string = match String::from_utf8(output.stdout) {
        Ok(s) => s,
        Err(_) => "".to_string(),
    };

    out_string
}
