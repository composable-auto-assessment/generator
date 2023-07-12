use std::{num::NonZeroU8, path::PathBuf};

use clap::Parser;
use image::Luma;
use qrcode::{EcLevel, QrCode};
use sha3::{
    digest::{ExtendableOutput, Update, XofReader},
    Shake128,
};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// The reference file
    file: PathBuf,
}

/// Exam medatada
#[derive(Copy, Clone, Debug)]
struct Meta {
    /// Page number. Usually continuous from 1 to n
    page: NonZeroU8,
    /// Exam id. Usually consant
    sujet: u8,
}

/// Iterator (over the pages of an exam) for exam metadata
struct MetaIter {
    current: Meta,
    stop: NonZeroU8,
}

/// To write the metadata in the qr code
impl From<Meta> for Vec<u8> {
    fn from(value: Meta) -> Self {
        return vec![value.sujet, value.page.into()];
    }
}
impl ToString for Meta {
    fn to_string(&self) -> String {
        self.sujet.to_string() + "-" + &self.page.to_string()
    }
}

impl Iterator for MetaIter {
    type Item = Meta;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current.page <= self.stop {
            let r = self.current;
            self.current.page = self.current.page.checked_add(1)?;
            Some(r)
        } else {
            None
        }
    }
}
impl IntoIterator for Meta {
    type Item = Meta;
    type IntoIter = MetaIter;

    fn into_iter(self) -> Self::IntoIter {
        MetaIter {
            current: Meta {
                page: NonZeroU8::new(1).unwrap(), //if this fails; may the lord help us.
                sujet: self.sujet,
            },
            stop: self.page,
        }
    }
}

impl Meta {
    fn new(page: u8, exam_id: u8) -> Option<Self> {
        Some(Meta {
            page: NonZeroU8::new(page)?,
            sujet: exam_id,
        })
    }
}

/// Hash size in bytes
const HASH_SIZE: usize = 16;

type Hash = [u8; HASH_SIZE];
type StrResult<T> = Result<T, String>;

/// Hashes a data buffer
fn gen_hash(data: &[u8]) -> Hash {
    let mut hasher = Shake128::default(); // Up to 256 bits
    hasher.update(data);
    let mut reader = hasher.finalize_xof();
    let mut res1 = Hash::default(); // Since SHAKE is of variable d, we have to specify the output size.
    reader.read(&mut res1);
    return res1;
}

/// Writes data as a qr code image.
/// Returns an error when the data is doo long, or writing failed.
fn qr_as_image(data: &[u8], filename: &str) -> StrResult<()> {
    // If we ever write more than 20 bytes we will need to reassess this.
    //  Either change the assert (and have a bigger code) or the EC level
    // https://developers.google.com/chart/infographics/docs/qr_codes?hl=fr#qr-code-details-[optional-reading]
    if data.len() > 20 {
        return Err("Too much data!".to_string());
    }
    // Write qr code
    let code = QrCode::with_error_correction_level(data, EcLevel::Q).map_err(|e| e.to_string())?;

    let image = code
        .render::<Luma<u8>>()
        .quiet_zone(false) // no border
        .build();

    // Save the image.
    image
        .save("tests/data/".to_owned() + filename + ".png")
        .map_err(|e| e.to_string())
}

/*
/// Generate a QR code
fn gen_qr(contents: Vec<&[u8]>, meta: Meta, name: &str) {
    // Generate hash
    let hash: [u8; 16] = gen_hash(&collapse_contents(contents));

    // Create the full data (here, hash + page number)
    let mut data = hash.to_vec();
    data.extend(Vec::from(i));

    // Write qr code
    qr_as_image(&data, &(name.to_owned() + &meta.to_string()));
}
*/

/// Generate a series of QR codes
fn mul_qr(contents: Vec<&[u8]>, meta: Meta, name: &str) -> StrResult<()> {
    // Generate hash
    let hash: [u8; 16] = gen_hash(&collapse_contents(contents));
    for i in meta {
        // Create the full data (here, hash + page number)
        let mut data = hash.to_vec();
        data.extend(Vec::from(i));
        //println!("{:?}", data);
        // Write qr code
        qr_as_image(&data, &(name.to_owned() + &i.to_string()))?;
    }
    Ok(())
}

/// Creates a unique hashable data buffer from multiple data buffers
fn collapse_contents(contents: Vec<&[u8]>) -> Vec<u8> {
    let len = contents.len().to_be_bytes();
    let mut data = contents
        .into_iter()
        .flat_map(|v| {
            // Add opening and closing terminators, ensuring that with contents = [file1,file2] and [file,1file2], the hashes are unique
            let mut a = b"<".to_vec();
            a.extend_from_slice(v);
            a.extend_from_slice(b">");
            a
        })
        .collect::<Vec<u8>>();
    // Add element counter, ensuring that [file1,file2] and [file1<file2>] hash differently
    data.extend_from_slice(&len);
    data
}

fn main() -> StrResult<()> {
    let cmd = Cli::parse();

    mul_qr(
        vec![&std::fs::read(cmd.file).map_err(|e| e.to_string())?],
        Meta::new(2, 0).ok_or("Page count cannot be O!")?,
        "qrcode-",
    )
}
