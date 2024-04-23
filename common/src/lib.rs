/// 将数据写入buf, 前四字节表示有效数据长度
pub fn data_write_to_buf(buf: &mut [u8], mut data: Vec<u8>) {
    let len = data.len() as u32;
    let len_bytes = unsafe { std::mem::transmute::<u32, [u8; 4]>(len) };
    let mut temp = Vec::from(&len_bytes);
    temp.append(&mut data);
    buf[..temp.len()].copy_from_slice(&temp);
}

/// 从buf中读取有效数据
pub fn data_read_from_buf(buf: &[u8]) -> Vec<u8> {
    let mut len = [0u8; 4];
    len.copy_from_slice(&buf[..4]);
    let len = unsafe { std::mem::transmute::<[u8; 4], u32>(len) };
    let len = len as usize;
    buf[4..len + 4].to_vec()
}

pub fn vf32_to_vu8(vf32: Vec<f32>) -> Vec<u8> {
    let vu8len = vf32.len() * 4;

    let mut ret = Vec::with_capacity(vu8len);
    unsafe { ret.set_len(vu8len) };

    ret.copy_from_slice(unsafe { core::slice::from_raw_parts(vf32.as_ptr() as *const u8, vu8len) });

    ret
}

pub fn vu8_to_vf32(vu8: Vec<u8>) -> Vec<f32> {
    let vf32len = vu8.len() / 4;

    let mut ret = Vec::with_capacity(vf32len);
    unsafe { ret.set_len(vf32len) };

    ret.copy_from_slice(unsafe {
        core::slice::from_raw_parts(vu8.as_ptr() as *const f32, vf32len)
    });

    ret
}


pub mod endpoint_config;
pub mod message;