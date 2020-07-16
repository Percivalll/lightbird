use crate::processor;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use std::str;
use std::time;
#[derive(Debug, Default, Copy, Clone)]
struct ProcessStat {
    utime: u64,
    stime: u64,
    cutime: u64,
    cstime: u64,
}
impl ProcessStat {
    fn get_total_with_children(&self) -> u64 {
        self.utime + self.stime + self.cutime + self.cstime
    }
    fn get_total(&self) -> u64 {
        self.utime + self.stime
    }
}
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Process {
    pub processor_usage_with_children: f32,
    pub processor_usage: f32,
    pub rss: String,
    pub pid: String,
    #[serde(skip)]
    last_stat: ProcessStat,
    #[serde(skip)]
    latest_stat: ProcessStat,
    #[serde(skip)]
    last_processor_stat: processor::ProcessStat,
    #[serde(skip)]
    latest_processor_stat: processor::ProcessStat,
}
impl Process {
    pub fn refresh(&mut self) -> Result<(), String> {
        self.last_processor_stat = self.latest_processor_stat;
        self.last_stat = self.latest_stat;
        self.latest_processor_stat = processor::get_total_processor_stat()?;
        self.latest_stat = get_process_stat(self.pid.to_owned())?;
        let total_diff =
            self.latest_processor_stat.get_total() - self.last_processor_stat.get_total();
        let processors = processor::new()?;
        if total_diff == 0 {
            self.processor_usage = 0.0;
            self.processor_usage_with_children = 0.0;
        } else {
            self.processor_usage = processors.len() as f32
                * (self.latest_stat.get_total() - self.last_stat.get_total()) as f32
                / total_diff as f32;
            self.processor_usage_with_children = processors.len() as f32
                * (self.latest_stat.get_total_with_children()
                    - self.last_stat.get_total_with_children()) as f32
                / total_diff as f32;
        }
        return Ok(());
    }
}
pub fn new(pid: &'static str) -> Result<Process, String> {
    let mut process = Process::default();
    let mut file = match File::open(String::from("/proc/") + pid + "/status") {
        Ok(o) => o,
        Err(err) => return Err(err.to_string()),
    };
    let mut content = String::new();
    match file.read_to_string(&mut content) {
        Ok(o) => o,
        Err(err) => return Err(err.to_string()),
    };
    for i in content
        .lines()
        .map(|line| {
            let mut fields = line.split(":");
            let (k, v) = (
                fields.next().unwrap_or("").trim(),
                fields.next().unwrap_or("").trim(),
            );
            (k, v)
        })
        .filter(|(k, v)| match *k {
            "VmRSS" | "Pid" => true,
            _ => false,
        })
    {
        match i.0 {
            "VmRSS" => process.rss = i.1.replace(" kB", ""),
            "Pid" => process.pid = i.1.to_string(),
            _ => {}
        }
    }
    process.latest_stat = get_process_stat(pid.to_owned())?;
    process.latest_processor_stat = processor::get_total_processor_stat()?;
    process.processor_usage = 0.0;
    process.processor_usage_with_children = 0.0;
    Ok(process)
}
fn get_process_stat(pid: String) -> Result<ProcessStat, String> {
    let mut content = String::new();
    let mut file = match File::open(String::from("/proc/") + pid.as_str() + "/stat") {
        Ok(o) => o,
        Err(err) => return Err(err.to_string()),
    };
    match file.read_to_string(&mut content) {
        Ok(o) => o,
        Err(err) => return Err(err.to_string()),
    };
    let fields: Vec<&str> = content.split_ascii_whitespace().collect();
    Ok(ProcessStat {
        utime: fields[13].parse::<u64>().unwrap(),
        stime: fields[14].parse::<u64>().unwrap(),
        cutime: fields[15].parse::<u64>().unwrap(),
        cstime: fields[16].parse::<u64>().unwrap(),
    })
}
#[test]
fn process_test() {
    loop {
        let mut process = new("1300").unwrap();
        std::thread::sleep(std::time::Duration::from_secs(45));
        process.refresh().unwrap();
        assert_ne!(process.processor_usage, 0.0);
        assert_ne!(process.processor_usage_with_children, 0.0);
        println!("{:?}", process.processor_usage)
    }
}
