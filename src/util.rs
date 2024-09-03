use std::{ffi::CStr, fs::File, io::Read};

pub fn str_to_cstr(rusty: &str) -> &CStr {
    CStr::from_bytes_with_nul(rusty.as_bytes()).expect("Failed to convert rustaican str to c str")
}

pub fn str_slice_to_cstr_vec<'a>(rusty: &'a [&'a str]) -> Vec<&'a CStr> {
    rusty.iter().map(|x| str_to_cstr(x)).collect()
}

pub fn read_shader_file(path: &str) -> Vec<u32> {
    let mut file = File::open(path).expect("Failed to open shader file");
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .expect("Failed to read shader file");

    if buffer.len() % 4 != 0 {
        panic!("File size is not a multiple of 4 bytes");
    }

    buffer
        .chunks(4)
        .map(|x| u32::from_le_bytes(x.try_into().unwrap()))
        .collect()
}
