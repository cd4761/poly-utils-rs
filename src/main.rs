pub mod binfile_utils;

use std::fs::File;
use std::io::{self, Read};
use binfile_utils::BinFile;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filename = "/Users/hwangjaeseung/workspace/zkp/poly-util-rust/files/fYK_1_2.zkey";

    let mut file = File::open(filename)?;
    let mut file_data = Vec::new();
    file.read_to_end(&mut file_data)?;

    let bin_file = BinFile::new(&file_data, file_data.len(), "zkey", 1)?;

    let section_data = bin_file.get_section_data(2, 0)?;
    let section_size = bin_file.get_section_size(2, 0)?;

    println!("Section size: {}", section_size);
    println!("section data: {:?}", section_data);
    unsafe {
        for i in 0..section_size {
            println!("{}", *section_data.add(i as usize));
        }
    }

    Ok(())
}
