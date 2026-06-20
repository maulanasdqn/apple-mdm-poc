#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnrollmentState {
    Authenticated,

    Enrolled,

    CheckedOut,
}

impl EnrollmentState {
    pub fn as_str(self) -> &'static str {
        match self {
            EnrollmentState::Authenticated => "authenticated",
            EnrollmentState::Enrolled => "enrolled",
            EnrollmentState::CheckedOut => "checked_out",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "enrolled" => EnrollmentState::Enrolled,
            "checked_out" => EnrollmentState::CheckedOut,
            _ => EnrollmentState::Authenticated,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Device {
    pub udid: String,

    pub push_token: Option<String>,
    pub push_magic: Option<String>,
    pub topic: Option<String>,

    pub unlock_token: Option<String>,
    pub enrollment_state: EnrollmentState,
}

impl Device {
    pub fn authenticated(udid: impl Into<String>) -> Self {
        Self {
            udid: udid.into(),
            push_token: None,
            push_magic: None,
            topic: None,
            unlock_token: None,
            enrollment_state: EnrollmentState::Authenticated,
        }
    }

    pub fn is_pushable(&self) -> bool {
        self.push_token.is_some() && self.push_magic.is_some() && self.topic.is_some()
    }
}
