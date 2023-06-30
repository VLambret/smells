use std::env;
use std::fs;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

fn create_files_in_directory(path: &Path, total_files: usize, width: usize, depth: usize, num_lines: usize) -> io::Result<()> {
    if depth == 0 {
        return Ok(());
    }
    fs::create_dir_all(path)?;

    for i in 0..width {
        for j in 0..(total_files / width) {
            let file_path = path.join(format!("file{}.txt", j));
            let file = fs::File::create(&file_path)?;
            let mut writer = io::BufWriter::new(file);

            for _ in 0..num_lines {
                writer.write_all(b"This is a line.\n")?;
            }
        }
        let dir_path = path.join(format!("dir{}", i));
        create_files_in_directory(&dir_path, total_files, width, depth - 1, num_lines)?;
    }

    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 5 {
        eprintln!("Usage: {} <total_files> <width> <depth> <num_lines>", args[0]);
        return;
    }

    let total_files: usize = match args[1].parse() {
        Ok(val) => val,
        Err(_) => {
            eprintln!("Erreur: total_files doit être un nombre entier");
            return;
        }
    };

    let width: usize = match args[2].parse() {
        Ok(val) => val,
        Err(_) => {
            eprintln!("Erreur: width doit être un nombre entier");
            return;
        }
    };

    let depth: usize = match args[3].parse() {
        Ok(val) => val,
        Err(_) => {
            eprintln!("Erreur: depth doit être un nombre entier");
            return;
        }
    };

    let num_lines: usize = match args[4].parse() {
        Ok(val) => val,
        Err(_) => {
            eprintln!("Erreur: num_lines doit être un nombre entier");
            return;
        }
    };

    let root_path = Path::new("root_directory");
    if let Err(err) = create_files_in_directory(&root_path, total_files, width, depth, num_lines) {
        eprintln!("Erreur lors de la création de l'arborescence de fichiers : {}", err);
    }
}

