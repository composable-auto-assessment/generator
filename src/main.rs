use std::path::PathBuf;

use sha3::{Shake128, digest::{Update, ExtendableOutput, XofReader}};
use qrcode::{QrCode, EcLevel};
use image::Luma;
use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    // The reference file
    file: PathBuf,
}

const HASH_SIZE: usize = 16;

type Hash = [u8; HASH_SIZE];

fn gen_hash(data: &[u8]) -> Hash {
    let mut hasher = Shake128::default(); // Up to 256 bits
    hasher.update(data);
    let mut reader = hasher.finalize_xof();
    let mut res1 = Hash::default(); // Since SHAKE is of variable d, we have to specify the output size.
    reader.read(&mut res1);
    return res1;
}

fn qr_as_image(data: &[u8], filename: &str) {
    // Write qr code
    let code = QrCode::with_error_correction_level(data, EcLevel::Q).unwrap();
        
    let image = code.render::<Luma<u8>>()
        .quiet_zone(false) // no border
        .build();

    // Save the image.
    image.save("tests/data/".to_owned() + filename + ".png").unwrap();
}

fn gen_qr(contents: Vec<&[u8]>, page: u8, name: &str) {
    // Generate hash
    let hash = gen_hash(
        &contents
            .into_iter()
            .flat_map(|v| v.to_owned()) // One may want to do this differently, here, [file1f, ile2] and [file1, file2] would output the same result.
            .collect::<Vec<u8>>()
    );

    // Create the full data (here, hash + page number)
    let mut data = hash.to_vec();
    data.push(page);

    // Write qr code
    qr_as_image(&data, &(name.to_owned() + &page.to_string()));
}

fn mul_qr(contents: Vec<&[u8]>, pages: u8, name: &str) {
    // Generate hash
    let hash = gen_hash(
        &contents
            .into_iter()
            .flat_map(|v| v.to_owned()) // One may want to do this differently, here, [file1f, ile2] and [file1, file2] would output the same result.
            .collect::<Vec<u8>>());
    for i in 1..pages+1 {
        // Create the full data (here, hash + page number)
        let mut data = hash.to_vec();
        data.push(i);

        // Write qr code
        qr_as_image(&data, &(name.to_owned() + &i.to_string()));
    }
}

fn main() {
    let cmd = Cli::parse();

    mul_qr(vec![&std::fs::read(cmd.file).unwrap()], 2, "qrcode-");
}
