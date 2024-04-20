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

#[test]
fn f() {
    let v: Vec<u8> = vec![0; 960];
    println!("{}", v.as_slice().len())
}



pub mod message;
pub mod endpoint_config;