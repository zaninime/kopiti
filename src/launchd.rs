use std::error;
use std::io;
use std::process;

pub trait ServiceScraper {
    fn list_running(&self) -> Result<Vec<ServiceInfo>, Box<dyn error::Error>>;
}

pub trait ServiceKiller {
    fn kill_by_label(&self, label: &str) -> io::Result<process::ExitStatus>;
}

#[derive(Debug, Clone)]
pub struct ServiceInfo {
    pub label: String,
    pub status: i32,
    pub pid: Option<u32>,
}

pub struct ServiceRepository {
    pub as_root: bool,
}

impl ServiceRepository {
    pub fn new(as_root: bool) -> ServiceRepository {
        ServiceRepository { as_root }
    }

    fn run_launchctl(&self, args: Vec<&str>) -> process::Command {
        let mut base_cmd = if self.as_root {
            let mut cmd = process::Command::new("sudo");
            cmd.arg("launchctl");
            cmd
        } else {
            process::Command::new("launchctl")
        };

        base_cmd.args(args);
        base_cmd
    }
}

impl ServiceScraper for ServiceRepository {
    fn list_running(&self) -> Result<Vec<ServiceInfo>, Box<dyn error::Error>> {
        let result = self.run_launchctl(vec!["list"]).output()?;

        parse_launchctl_list(result.stdout)
    }
}

impl ServiceKiller for ServiceRepository {
    fn kill_by_label(&self, label: &str) -> io::Result<process::ExitStatus> {
        self.run_launchctl(vec!["remove", label]).status()
    }
}

fn parse_launchctl_list<'a>(stdout: Vec<u8>) -> Result<Vec<ServiceInfo>, Box<dyn error::Error>> {
    let text_out = String::from_utf8(stdout)?;

    let mut text_processes = text_out.lines();

    // skip header
    text_processes.next();

    text_processes
        .map(|process_text| {
            let fields: Vec<_> = process_text.splitn(3, '\t').collect();
            let pid_text = fields[0];
            let status_text = fields[1];
            let label = fields[2].to_string();

            let pid = match pid_text {
                "-" => None,
                x => {
                    let num: u32 = x.parse()?;
                    Some(num)
                }
            };

            let status: i32 = status_text.parse()?;

            Ok(ServiceInfo { label, status, pid })
        })
        .collect()
}
