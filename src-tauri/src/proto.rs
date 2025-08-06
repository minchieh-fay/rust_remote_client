use std::io::{Read, Write};

// 这个文件是从server那边拷贝过来的，协议要对上，勿修改

// 消息类型枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MessageType {
    Handshake = 1,      // 握手注册
    HandshakeOK = 2, // 握手响应成功
    Data = 3,           // 数据转发
    Fin = 4,            // 断开连接
}

// 数据消息来源类型
pub const SOURCE_TYPE_CONTROLLER: u8 = 1;    // 控制端
pub const SOURCE_TYPE_CONTROLLED: u8 = 2;    // 被控制端

impl From<u8> for MessageType {
    fn from(value: u8) -> Self {
        match value {
            1 => MessageType::Handshake,
            2 => MessageType::HandshakeOK,
            3 => MessageType::Data,
            4 => MessageType::Fin,
            _ => panic!("未知的消息类型: {}", value),
        }
    }
}

impl From<MessageType> for u8 {
    fn from(value: MessageType) -> Self {
        value as u8
    }
}

// 握手消息结构 - 二进制格式
#[derive(Debug)]
pub struct HandshakeMessage {
    pub id: [u8; 10],  // 10字节ID
}

impl HandshakeMessage {
    pub fn new(id: [u8; 10]) -> Self {
        Self { id }
    }

    // 从字节流读取
    pub fn read_from<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        let mut id = [0u8; 10];
        reader.read_exact(&mut id)?;
        Ok(Self { id })
    }

    // 写入字节流
    pub fn write_to<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(&self.id)?;
        Ok(())
    }
}

// 数据转发消息结构 - 二进制格式
#[derive(Debug)]
pub struct DataMessage {
    pub controlled_id: [u8; 10],  // 被控制端ID 10字节
    pub controller_id: [u8; 10], // 控制端ID 10字节
    pub session_id: [u8; 4],     // 会话ID 4字节 (seq)
    pub source_type: u8,          // 消息来源类型 1字节 (1=控制端, 2=被控制端)
    // 目标addr（ipv4+port）
    pub target_addr: [u8; 6],     // 目标地址 6字节 (ipv4+port)
    pub data_length: u32,         // 数据长度 4字节
    pub data: Vec<u8>,            // 实际数据
}

impl DataMessage {
    pub fn new(controlled_id: [u8; 10], controller_id: [u8; 10], session_id: [u8; 4], source_type: u8, target_addr: [u8; 6], data: Vec<u8>) -> Self {
        Self {
            controlled_id,
            controller_id,
            session_id,
            source_type,
            target_addr,   
            data_length: data.len() as u32, 
            data,
        }
    }

    // 从字节流读取
    pub fn read_from<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        let mut controlled_id = [0u8; 10];
        let mut controller_id = [0u8; 10];
        let mut session_id = [0u8; 4];
        let mut source_type = [0u8; 1];
        let mut target_addr = [0u8; 6];
        let mut length_buf = [0u8; 4];
        
        reader.read_exact(&mut controlled_id)?;
        reader.read_exact(&mut controller_id)?;
        reader.read_exact(&mut session_id)?;
        reader.read_exact(&mut source_type)?;
        reader.read_exact(&mut target_addr)?;
        reader.read_exact(&mut length_buf)?;
        
        let data_length = u32::from_be_bytes(length_buf);
        let mut data = vec![0u8; data_length as usize];
        reader.read_exact(&mut data)?;
        
        Ok(Self {
            controlled_id,
            controller_id,
            session_id,
            source_type: source_type[0],
            target_addr,
            data_length,
            data,
        })
    }

    // 写入字节流
    pub fn write_to<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(&self.controlled_id)?;
        writer.write_all(&self.controller_id)?;
        writer.write_all(&self.session_id)?;
        writer.write_all(&[self.source_type])?;
        writer.write_all(&self.target_addr)?;
        writer.write_all(&self.data_length.to_be_bytes())?;
        writer.write_all(&self.data)?;
        Ok(())
    }
}

// 断开连接消息结构 - 二进制格式
#[derive(Debug)]
pub struct FinMessage {
    pub controlled_id: [u8; 10],  // 被控制端ID 10字节
    pub controller_id: [u8; 10],  // 控制端ID 10字节
    pub session_id: [u8; 4],      // 会话ID 4字节 (seq)
    pub source_type: u8,          // 消息来源类型 1字节 (1=控制端, 2=被控制端)
}

impl FinMessage {
    pub fn new(controlled_id: [u8; 10], controller_id: [u8; 10], session_id: [u8; 4], source_type: u8) -> Self {
        Self { controlled_id, controller_id, session_id, source_type }
    }

    // 从字节流读取
    pub fn read_from<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        let mut controlled_id = [0u8; 10];
        let mut controller_id = [0u8; 10];
        let mut session_id = [0u8; 4];
        let mut source_type = [0u8; 1];
        
        reader.read_exact(&mut controlled_id)?;
        reader.read_exact(&mut controller_id)?;
        reader.read_exact(&mut session_id)?;
        reader.read_exact(&mut source_type)?;
        
        Ok(Self { 
            controlled_id, 
            controller_id,
            session_id, 
            source_type: source_type[0] 
        })
    }

    // 写入字节流
    pub fn write_to<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(&self.controlled_id)?;
        writer.write_all(&self.controller_id)?;
        writer.write_all(&self.session_id)?;
        writer.write_all(&[self.source_type])?;
        Ok(())
    }
}

// 消息处理函数
pub fn handle_handshake_message(data: &[u8]) -> std::io::Result<HandshakeMessage> {
    if data.len() < 10 {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "握手消息数据长度不足"));
    }
    
    let mut id = [0u8; 10];
    id.copy_from_slice(&data[..10]);
    Ok(HandshakeMessage::new(id))
}

pub fn handle_data_message(data: &[u8]) -> std::io::Result<DataMessage> {
    if data.len() < 33 { // 10 + 10 + 4 + 1 + 6 + 2 = 33字节最小长度
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "数据消息长度不足"));
    }
    
    let mut controlled_id = [0u8; 10];
    let mut controller_id = [0u8; 10];
    let mut session_id = [0u8; 4];
    let mut source_type = [0u8; 1];
    let mut target_addr = [0u8; 6];
    let mut length_buf = [0u8; 4];
    
    controlled_id.copy_from_slice(&data[..10]);
    controller_id.copy_from_slice(&data[10..20]);
    session_id.copy_from_slice(&data[20..24]);
    source_type.copy_from_slice(&data[24..25]);
    target_addr.copy_from_slice(&data[25..31]);
    length_buf.copy_from_slice(&data[31..35]);
    
    let data_length = u32::from_be_bytes(length_buf);
    if data.len() < 27 + data_length as usize {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "数据长度不匹配"));
    }
    
    let actual_data = data[27..27 + data_length as usize].to_vec();
    
    Ok(DataMessage::new(controlled_id, controller_id, session_id, source_type[0], target_addr, actual_data))
}

pub fn handle_fin_message(data: &[u8]) -> std::io::Result<FinMessage> {
    if data.len() < 25 { // 10 + 10 + 4 + 1 = 25字节
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "断开消息数据长度不足"));
    }
    
    let mut controlled_id = [0u8; 10];
    let mut controller_id = [0u8; 10];
    let mut session_id = [0u8; 4];
    let mut source_type = [0u8; 1];
    
    controlled_id.copy_from_slice(&data[..10]);
    controller_id.copy_from_slice(&data[10..20]);
    session_id.copy_from_slice(&data[20..24]);
    source_type.copy_from_slice(&data[24..25]);
    
    Ok(FinMessage::new(controlled_id, controller_id, session_id, source_type[0]))
}

// 创建握手成功响应（只有消息类型）
pub fn create_handshake_ok_response() -> Vec<u8> {
    vec![MessageType::HandshakeOK.into()]
}

// 创建数据响应
pub fn create_data_response(data: Vec<u8>) -> std::io::Result<Vec<u8>> {
    let mut response = Vec::new();
    response.push(MessageType::Data.into());
    response.extend_from_slice(&data);
    
    Ok(response)
}

// 创建断开连接响应
pub fn create_fin_response(controlled_id: [u8; 10], controller_id: [u8; 10], session_id: [u8; 4], source_type: u8) -> std::io::Result<Vec<u8>> {
    let message = FinMessage::new(controlled_id, controller_id, session_id, source_type);
    
    let mut response = Vec::new();
    response.push(MessageType::Fin.into());
    message.write_to(&mut response)?;
    
    Ok(response)
}

// 工具函数：生成会话ID
pub fn generate_session_id(id: &[u8; 10], seq: u16) -> [u8; 14] {
    let mut session_id = [0u8; 14];
    
    // 复制ID (10字节)
    session_id[..10].copy_from_slice(id);
    
    // 添加序列号 (4字节，大端序)
    let seq_bytes = seq.to_be_bytes();
    session_id[10..14].copy_from_slice(&seq_bytes);
    
    session_id
}

// 工具函数：将字节数组转换为字符串
pub fn bytes_to_string(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).trim_matches('\0').to_string()
}

// 工具函数：将字符串填充到固定长度
pub fn string_to_fixed_bytes(s: &str, length: usize) -> Vec<u8> {
    let mut bytes = s.as_bytes().to_vec();
    bytes.resize(length, 0);
    bytes
}
