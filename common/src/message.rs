#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub enum Message {
    /// 请求在服务器挂起等待
    Wait(String),

    /// 请求呼叫
    Call(String),

    /// 请求服务器可被呼叫用户列表
    QueryUsers,

    /// 请求结束通话
    Close,

    /// 服务器回应请求结果
    Response(Res),
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub enum Res {
    Ok,
    Err,
    Wait,
    Wake,
    UserList(Vec<String>),
}

impl Message {
    pub fn to_vec_u8(&self) -> Vec<u8> {
        let json = serde_json::to_vec(&self).unwrap();
        json
    }
}

use std::fmt::Display;
impl Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Message::Wait(name) => format!("等待被呼叫 name({})", name),
            Message::Call(name) => format!("呼叫 name({})", name),
            Message::QueryUsers => "查询等待列表".into(),
            // Message::FrameSize(a, v) => format!("音频帧字节大小({a}) 视频帧字节大小({v})"),
            Message::Close => "关闭通信".into(),
            Message::Response(_) => "Result".into(),
        };
        f.write_str(&str)
    }
}

impl Res {
    pub fn is_ok(&self) -> bool {
        match self {
            Res::Ok => true,
            _ => false,
        }
    }
}

#[test]
fn f1() {
    let msg = Message::Call("(22)\011".into());
    let v = serde_json::to_string(&msg).unwrap();
    let v = v.as_bytes();
    let mut vv = vec![0u8; v.len() * 2];

    vv[..v.len()].copy_from_slice(v);

    let vv: Vec<_> = vv.into_iter().rev().collect();
    let vv: Vec<_> = vv.into_iter().skip_while(|e| *e == 0).collect();
    let vv: Vec<_> = vv.into_iter().rev().collect();

    let _msg: Message = serde_json::from_slice(&vv).unwrap();

    if let (Message::Call(s1), Message::Call(s2)) = (msg, _msg) {
        println!("{} {} {} {}", s1, s2, s1.len(), s2.len())
    }
}
