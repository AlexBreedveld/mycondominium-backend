use crate::services::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SmtpEmailPayload {
    pub to: String,
    pub subject: String,
    pub body: String,
}
