use std::fs::File;
use std::io::{self, Write, Read};
use std::path::Path;
use std::fs;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BinFile;

    fn create_temp_file(data: &[u8], filename: &str) -> io::Result<String> {
        let mut file = File::create(filename)?;
        file.write_all(data)?;
        Ok(filename.to_string())
    }

    fn remove_temp_file(path: &str) -> io::Result<()> {
        fs::remove_file(path)
    }

    #[test]
    fn test_large_file() {
        let mut file_data = vec![b'T', b'E', b'S', b'T'];
        file_data.extend_from_slice(&1u32.to_le_bytes()); // Valid version
        file_data.extend_from_slice(&1u32.to_le_bytes()); // One section
        file_data.extend_from_slice(&1u32.to_le_bytes()); // Section type
        file_data.extend_from_slice(&(1_000_000u64).to_le_bytes()); // Large section size
        file_data.extend(vec![0; 1_000_000]); // Section data

        let filename = "test_file.bin";
        create_temp_file(&file_data, filename).expect("Failed to create temp file");

        let file_data = fs::read(filename).expect("Failed to read temp file");

        let bin_file = BinFile::new(&file_data, file_data.len(), "zkey", 1).unwrap();

        let section_data = bin_file.get_section_data(1, 0).unwrap();
        let section_size = bin_file.get_section_size(1, 0).unwrap();

        assert_eq!(section_size, 1_000_000);

        remove_temp_file(filename).expect("Failed to remove temp file");
    }

    #[test]
    fn test_invalid_data() {
        let mut file_data = vec![b'T', b'E', b'S', b'T'];
        file_data.extend_from_slice(&1u32.to_le_bytes()); // Valid version
        file_data.extend_from_slice(&1u32.to_le_bytes()); // One section
        file_data.extend_from_slice(&1u32.to_le_bytes()); // Section type
        file_data.extend_from_slice(&(4u64).to_le_bytes()); // Section size
        file_data.extend_from_slice(&[1, 2, 3]); // Invalid section data (size mismatch)

        let filename = "invalid_test_file.bin";
        create_temp_file(&file_data, filename).expect("Failed to create temp file");

        let file_data = fs::read(filename).expect("Failed to read temp file");

        let result = BinFile::new(&file_data, file_data.len(), "TEST", 1);
        assert!(result.is_err());

        remove_temp_file(filename).expect("Failed to remove temp file");
    }

    #[test]
    fn test_actual_project_data() {
        let filename = "/Users/hwangjaeseung/workspace/zkp/poly-util-rust/files/fYK_1_2.zkey";
        let file_data = fs::read(filename).expect("Failed to read actual data file");

        let bin_file = BinFile::new(&file_data, file_data.len(), "zkey", 1).unwrap();

        // 실제 데이터 파일의 섹션을 검증합니다.
        let section_data = bin_file.get_section_data(2, 0).unwrap();
        let section_size = bin_file.get_section_size(2, 0).unwrap();

        // 섹션 데이터를 실제 값과 비교합니다.
        assert_eq!(section_size, 1024);
        unsafe {
            assert_eq!(*section_data, 1);
            assert_eq!(*section_data.add(1), 2);
            assert_eq!(*section_data.add(2), 3);
            assert_eq!(*section_data.add(3), 4);
        }
    }
}
