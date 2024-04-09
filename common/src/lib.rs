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
    Result(Info, String),
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub enum Info {
    Close,
    UserNameNotFound,
    UserNameExisted,
    UserList(Vec<String>),
}
