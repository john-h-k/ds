use std::{
    env, fs, io,
    path::{Path, PathBuf},
};

use rayon::prelude::*;

fn main() {
    //"a".parse::<u32>().unwrap();
    let first = env::args().nth(1).unwrap();
    if Path::new(&first).exists() {
        println!("{} exists!", first);
    }

    env::args().collect::<Vec<_>>().par_iter().for_each(|d| {
        let path = PathBuf::from(d);

        let size = get_size(&path);

        match size {
            Ok(size) => println!("{} {}", human_size(size), d),
            Err(err) => eprintln!("Errored attempting '{}', err = {}", d, err),
        }
    });
}

fn human_size(bytes: u64) -> String {
    let magnitude = (64 - 1 - bytes.leading_zeros()) / 10;

    let suffix = match magnitude {
        0 => "B",
        1 => "K",
        2 => "M",
        3 => "G",
        4 => "T",
        5 => "P",
        _ => "<ginormous!>",
    };

    let adjusted = bytes as f64 / 1024f64.powi(magnitude.try_into().unwrap());

    format!(
        "{:>3.N$}{}",
        adjusted,
        suffix,
        N = if adjusted > 10.0 { 0 } else { 1 },
    )
}

fn get_size(path: &Path) -> io::Result<u64> {
    if path.is_dir() {
        println!("{} is dir", path.to_string_lossy());
        fs::read_dir(path)
            .unwrap()
            .collect::<Vec<_>>()
            .into_par_iter()
            .map(|entry| -> io::Result<u64> {
                let entry = entry.unwrap();
                get_size(&entry.path())
            })
            .sum()
    } else {
        Ok(path.metadata().unwrap().len())
    }
}
