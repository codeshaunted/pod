use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Unit {
    pub kind: String,
    pub addr_virtual: usize,
    pub raw_size: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Section {
    pub name: String,
    //pub addr_file: u32,
    //pub size_file: u32,
    //pub addr_virtual: u32,
    //pub size_virtual: u32,
    //pub flags: u32,
    pub units: Vec<Unit>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub executable: String,
    pub hash: String,
    pub assembler_path: String,
    pub compiler_path: String,
    pub linker_path: String,
    //pub base_addr_virtual: u64,
    //pub entry: u32,
    //pub subsystem: u16,
    pub sections: Vec<Section>,
}
