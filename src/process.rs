use crate::processor;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use std::str;
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Process {
    pub processor_usage_with_children: f32,
    pub processor_usage: f32,
    pub rss: String,
    pub pid: String,
}
pub fn get_process(pid: &'static str) -> Result<Process, String> {
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
    let processor_usage=get_process_usage(pid)?;
    process.processor_usage=processor_usage.0;
    process.processor_usage_with_children=processor_usage.1;
    Ok(process)
}
fn get_process_cpu_usage_with_children(pid: &'static str) -> Result<(u64, u64), String> {
    let mut file = match File::open(String::from("/proc/") + pid + "/stat") {
        Ok(o) => o,
        Err(err) => return Err(err.to_string()),
    };
    let mut content = String::new();
    match file.read_to_string(&mut content) {
        Ok(_) => {}
        Err(err) => return Err(err.to_string()),
    };

    let fields: Vec<&str> = content.split_ascii_whitespace().collect();
    if fields.len() < 17 {}
    Ok((
        fields[13].parse::<u64>().unwrap() + fields[14].parse::<u64>().unwrap(),
        fields[13].parse::<u64>().unwrap()
            + fields[15].parse::<u64>().unwrap()
            + fields[14].parse::<u64>().unwrap()
            + fields[16].parse::<u64>().unwrap(),
    ))
}
fn get_process_usage(pid: &'static str) -> Result<(f32, f32), String> {
    let total_start = processor::get_total_processor_stat()?;
    let process_start = get_process_cpu_usage_with_children(pid)?;
    let processor_num = processor::get_processor().unwrap().len();
    let total_stop = processor::get_total_processor_stat()?;
    let process_stop = get_process_cpu_usage_with_children(pid)?;
    Ok((
        processor_num as f32 * (process_stop.0 - process_start.0) as f32
            / (total_stop.get_total() - total_start.get_total()) as f32,
        processor_num as f32 * (process_stop.1 - process_start.1) as f32
            / (total_stop.get_total() - total_start.get_total()) as f32,
    ))
}
#[test]
fn get_process_test(){
    println!("{:?}",get_process("1300"));
}