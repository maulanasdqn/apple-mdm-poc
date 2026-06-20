#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScepOperation {
    GetCACert,
    GetCACaps,
    PkiOperation,
    Unknown,
}

impl ScepOperation {
    pub fn from_query(op: &str) -> Self {
        match op {
            "GetCACert" => ScepOperation::GetCACert,
            "GetCACaps" => ScepOperation::GetCACaps,
            "PKIOperation" => ScepOperation::PkiOperation,
            _ => ScepOperation::Unknown,
        }
    }
}

pub fn ca_caps() -> &'static str {
    "POSTPKIOperation\nSHA-256\nAES\n"
}
