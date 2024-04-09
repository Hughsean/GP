#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub enum Message {
    // 只在建立连接时传输
    Wait(String),
    Call(String),
    QueryUsers,
    // call之后传输
    Video(
        Vec<u8>,
        // from: String,
        // to: String,
    ),
    Audio(
        Vec<f32>,
        // from: String,
        // to: String,
    ),
    Close,
    //服务器传
    Result(Info),
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub enum Info {
    Ok,
    Err,
    Close,
    UserList(Vec<String>),
}

impl Info {
    pub fn is_ok(&self) -> bool {
        match self {
            Info::Ok => true,
            _ => false,
        }
    }
    pub fn is_close(&self) -> bool {
        match self {
            Info::Close => true,
            _ => false,
        }
    }
}

#[test]
fn fun1() {
    let v: Vec<u8> = vec![0; 691200];
    let m = Message::Video(v);
    let m = serde_json::to_string(&m).unwrap();
    println!("{}", m.as_bytes().len())
}

#[test]
fn fun2() {
    let v: Vec<f32> = vec![0.; 960];
    let m = Message::Audio(v);
    let m = serde_json::to_string(&m).unwrap();
    println!("{}", m.as_bytes().len())
}


#[test]
fn f(){
    let v: Vec<u8> = vec![0; 960];
    println!("{}",v.as_slice().len())
    
}

