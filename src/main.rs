use std::{env, fs, io, path::Path};

use rayon::prelude::*;

fn main() {
    env::args().collect::<Vec<_>>().par_iter().for_each(|d| {
        let path = Path::new(d);

        let size = get_size(path);

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
        fs::read_dir(path)?
            .collect::<Vec<_>>()
            .into_par_iter()
            .map(|entry| -> io::Result<u64> {
                let entry = entry?;
                get_size(&entry.path())
            })
            .sum()
    } else {
        Ok(path.metadata()?.len())
    }
}
