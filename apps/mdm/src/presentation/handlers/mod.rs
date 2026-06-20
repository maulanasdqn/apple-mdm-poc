pub mod admin;
pub mod checkin;
pub mod command;
pub mod enroll;
pub mod health;
pub mod scep;

pub mod content_type {
    pub const ENROLL_PROFILE: &str = "application/x-apple-aspen-config";
    pub const MDM_COMMAND: &str = "application/x-apple-aspen-mdm";
    pub const CA_CERT: &str = "application/x-x509-ca-cert";
}
