pub enum Message {
    SetValue {key: String, value: String},
    GetValue {key: String},
    Delete {key: String},
    Ack {success: bool}
}