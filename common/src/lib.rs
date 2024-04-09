#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub enum Message {
    // 只在建立连接时传输
    Wait(String),
    Call(String),
    QueryUsers,
    // call之后传输
    Video {
        data: Vec<u8>,
        from: String,
        to: String,
    },
    Audio {
        data: Vec<f32>,
        from: String,
        to: String,
    },
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
