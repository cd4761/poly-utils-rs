use std::collections::HashMap;
use std::convert::TryInto;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct InvalidFileType {
    expected: String,
    found: String,
}

impl fmt::Display for InvalidFileType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid file type. Expected: {}, Found: {}", self.expected, self.found)
    }
}

impl Error for InvalidFileType {}

#[derive(Debug, Clone)]
pub struct InvalidVersion {
    max_version: u32,
    found_version: u32,
}

impl fmt::Display for InvalidVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid version. It should be <= {}, but found {}", self.max_version, self.found_version)
    }
}

impl Error for InvalidVersion {}

#[derive(Debug, Clone)]
pub struct RangeError {
    details: String,
}

impl RangeError {
    fn new(msg: &str) -> RangeError {
        RangeError {
            details: msg.to_string(),
        }
    }
}

impl fmt::Display for RangeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for RangeError {}

#[derive(Debug, Clone)]
pub struct Section {
    start: *const u8,
    size: u64,
}

pub struct BinFile {
    size: usize,
    addr: Vec<u8>,
    file_type: String,
    pos: usize,
    version: u32,
    sections: HashMap<u32, Vec<Section>>,
    reading_section: Option<Section>,
}

impl BinFile {
    pub fn new(file_data: &[u8], file_size: usize, expected_type: &str, max_version: u32) -> Result<Self, Box<dyn Error>> {
        let mut addr = Vec::with_capacity(file_size);
        addr.extend_from_slice(file_data);

        let file_type = String::from_utf8(addr[0..4].to_vec())?;
        let mut pos = 4;

        if file_type != expected_type {
            return Err(Box::new(InvalidFileType { expected: expected_type.to_string(), found: file_type }));
        }

        let version = Self::read_u32_le(&addr, &mut pos);
        if version > max_version {
            return Err(Box::new(InvalidVersion { max_version, found_version: version }));
        }

        let n_sections = Self::read_u32_le(&addr, &mut pos);
        let mut sections = HashMap::new();

        for _ in 0..n_sections {
            let s_type = Self::read_u32_le(&addr, &mut pos);
            let s_size = Self::read_u64_le(&addr, &mut pos);

            sections.entry(s_type).or_insert_with(Vec::new).push(Section { start: addr[pos..].as_ptr(), size: s_size });
            pos += s_size as usize;
        }

        Ok(BinFile {
            size: file_size,
            addr,
            file_type,
            pos: 0,
            version,
            sections,
            reading_section: None,
        })
    }

    pub fn start_read_section(&mut self, section_id: u32, section_pos: u32) -> Result<(), Box<dyn Error>> {
        if !self.sections.contains_key(&section_id) {
            return Err(Box::new(RangeError::new(&format!("Section does not exist: {}", section_id))));
        }

        if section_pos as usize >= self.sections[&section_id].len() {
            return Err(Box::new(RangeError::new(&format!("Section pos too big. There are {} and it's trying to access section: {}", self.sections[&section_id].len(), section_pos))));
        }

        if self.reading_section.is_some() {
            return Err(Box::new(RangeError::new("Already reading a section")));
        }

        let section = self.sections[&section_id][section_pos as usize].clone();
        self.pos = (section.start as usize) - (self.addr.as_ptr() as usize);
        self.reading_section = Some(section);

        Ok(())
    }

    pub fn end_read_section(&mut self, check: bool) -> Result<(), Box<dyn Error>> {
        if check {
            if (self.addr.as_ptr() as usize + self.pos) - (self.reading_section.as_ref().unwrap().start as usize) != self.reading_section.as_ref().unwrap().size as usize {
                return Err(Box::new(RangeError::new("Invalid section size")));
            }
        }
        self.reading_section = None;

        Ok(())
    }

    pub fn get_section_data(&self, section_id: u32, section_pos: u32) -> Result<*const u8, Box<dyn Error>> {
        if !self.sections.contains_key(&section_id) {
            return Err(Box::new(RangeError::new(&format!("Section does not exist: {}", section_id))));
        }

        if section_pos as usize >= self.sections[&section_id].len() {
            return Err(Box::new(RangeError::new(&format!("Section pos too big. There are {} and it's trying to access section: {}", self.sections[&section_id].len(), section_pos))));
        }

        Ok(self.sections[&section_id][section_pos as usize].start)
    }

    pub fn get_section_size(&self, section_id: u32, section_pos: u32) -> Result<u64, Box<dyn Error>> {
        if !self.sections.contains_key(&section_id) {
            return Err(Box::new(RangeError::new(&format!("Section does not exist: {}", section_id))));
        }

        if section_pos as usize >= self.sections[&section_id].len() {
            return Err(Box::new(RangeError::new(&format!("Section pos too big. There are {} and it's trying to access section: {}", self.sections[&section_id].len(), section_pos))));
        }

        Ok(self.sections[&section_id][section_pos as usize].size)
    }

    fn read_u32_le(data: &[u8], pos: &mut usize) -> u32 {
        let result = u32::from_le_bytes(data[*pos..*pos + 4].try_into().unwrap());
        *pos += 4;
        result
    }

    fn read_u64_le(data: &[u8], pos: &mut usize) -> u64 {
        let result = u64::from_le_bytes(data[*pos..*pos + 8].try_into().unwrap());
        *pos += 8;
        result
    }

    pub fn read(&mut self, len: u64) -> Result<*const u8, Box<dyn Error>> {
        let start = self.addr[self.pos..].as_ptr();
        self.pos += len as usize;
        Ok(start)
    }
}
