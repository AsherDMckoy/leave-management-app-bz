use super::*;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, sqlx::Type)]
#[sqlx(type_name = "leave_type")]
#[sqlx(rename_all = "lowercase")]
pub enum LeaveType {
    Maternity,
    Paternity,
    Vacation,
    Sick,
    Study,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, sqlx::Type)]
#[sqlx(type_name = "status")]
#[sqlx(rename_all = "lowercase")]
pub enum Status {
    Pending,
    Approved,
    Rejected,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LeaveRequestInput {
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub days: i32,
    pub leave_type: LeaveType,
    pub comments: Option<String>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct LeaveRequestOutput {
    pub id: i32,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub days: i32,
    pub leave_type: LeaveType,
    pub comments: Option<String>,
    pub status: Status,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct MyLeavePreview {
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub days: i32,
    pub leave_type: LeaveType,
    pub status: Status,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct TeamLeavePreview {
    pub name: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub days: i32,
    pub current_status: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LeaveRequestUpdate {
    pub id: i32,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub leave_type: LeaveType,
    pub comments: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Request {
    pub id: i32,
    pub name: String,
    pub job_title: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub days: i32,
    pub leave_type: LeaveType,
    pub comments: Option<String>,
    pub apply_date: NaiveDate,
}

// Struct to hold employee-specific data for Gantt chart
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmployeeGanttData {
    pub name: String,
    pub job_title: String,
    pub leaves: Vec<LeaveGanttData>,
}

// Struct to hold each leave's data for Gantt chart
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaveGanttData {
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub leave_type: LeaveType,
    pub leave_days: Vec<u32>, //Pre computed days of leave
}
impl LeaveRequestOutput {
    pub fn print_start_date(&self) -> String {
        let day = self.start_date.day();
        format!(
            "{}/{}/{}",
            self.start_date.format("%m"),
            day,
            self.start_date.format("%Y")
        )
    }
    pub fn print_end_date(&self) -> String {
        let day = self.end_date.day();
        format!(
            "{}/{}/{}",
            self.start_date.format("%m"),
            day,
            self.start_date.format("%Y")
        )
    }
}
impl LeaveGanttData {
    pub fn start_day(&self) -> u32 {
        self.start_date.day()
    }

    pub fn end_day(&self) -> u32 {
        self.end_date.day()
    }

    pub fn start_month(&self) -> u32 {
        self.start_date.month()
    }

    pub fn end_month(&self) -> u32 {
        self.end_date.month()
    }
}

impl Request {
    pub fn print_start_date(&self) -> String {
        let day = self.start_date.day();
        format!(
            "{}/{}/{}",
            self.start_date.format("%m"),
            day,
            self.start_date.format("%Y")
        )
    }
    pub fn print_end_date(&self) -> String {
        let day = self.end_date.day();
        format!(
            "{}/{}/{}",
            self.end_date.format("%m"),
            day,
            self.end_date.format("%Y")
        )
    }
}

impl fmt::Display for LeaveType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LeaveType::Maternity => write!(f, "Maternity"),
            LeaveType::Paternity => write!(f, "Paternity"),
            LeaveType::Vacation => write!(f, "Vacation"),
            LeaveType::Sick => write!(f, "Sick"),
            LeaveType::Study => write!(f, "Study"),
        }
    }
}

impl FromStr for LeaveType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "maternity" => Ok(LeaveType::Maternity),
            "paternity" => Ok(LeaveType::Paternity),
            "vacation" => Ok(LeaveType::Vacation),
            "sick" => Ok(LeaveType::Sick),
            "study" => Ok(LeaveType::Study),
            _ => Err(format!("Unknown leave type: {}", s)),
        }
    }
}

impl PartialEq<str> for LeaveType {
    fn eq(&self, other: &str) -> bool {
        self.to_string().eq_ignore_ascii_case(other)
    }
}

impl PartialEq<String> for LeaveType {
    fn eq(&self, other: &String) -> bool {
        self == other.as_str()
    }
}

impl PartialEq<LeaveType> for str {
    fn eq(&self, other: &LeaveType) -> bool {
        other == self
    }
}

impl PartialEq<LeaveType> for String {
    fn eq(&self, other: &LeaveType) -> bool {
        other == self
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Status::Pending => write!(f, "Pending"),
            Status::Approved => write!(f, "Approved"),
            Status::Rejected => write!(f, "Rejected"),
        }
    }
}

impl FromStr for Status {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pending" => Ok(Status::Pending),
            "approved" => Ok(Status::Approved),
            "rejected" => Ok(Status::Rejected),
            _ => Err(format!("Unknown status: {}", s)),
        }
    }
}

impl PartialEq<str> for Status {
    fn eq(&self, other: &str) -> bool {
        self.to_string().eq_ignore_ascii_case(other)
    }
}

impl PartialEq<String> for Status {
    fn eq(&self, other: &String) -> bool {
        self == other.as_str()
    }
}

impl PartialEq<Status> for str {
    fn eq(&self, other: &Status) -> bool {
        other == self
    }
}

impl PartialEq<Status> for String {
    fn eq(&self, other: &Status) -> bool {
        other == self
    }
}

impl PartialEq<&str> for Status {
    fn eq(&self, other: &&str) -> bool {
        self.to_string().eq_ignore_ascii_case(*other)
    }
}

impl PartialEq<Status> for &str {
    fn eq(&self, other: &Status) -> bool {
        other == *self
    }
}
