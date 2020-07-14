use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use std::str;
use std::time;
#[derive(Debug)]
pub(super) struct ProcessStat {
    user: i64,
    nice: i64,
    system: i64,
    idle: i64,
    iowait: i64,
    irq: i64,
    softirq: i64,
}
impl ProcessStat {
    pub fn get_total(&self) -> i64 {
        self.user + self.nice + self.system + self.idle + self.iowait + self.irq + self.softirq
    }
    pub fn get_work(&self) -> i64 {
        self.user + self.nice + self.system + self.irq + self.softirq
    }
}
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Processor {
    pub index: i32,
    pub vendor_id: String,
    pub name: String,
    pub freq: f32,
    pub cache_size: String,
    pub usage: f32,
}
pub(super) fn get_processor_stat() -> Result<Vec<ProcessStat>, String> {
    let mut stat_file = match File::open("/proc/stat") {
        Ok(o) => o,
        Err(err) => return Err(err.to_string()),
    };
    let mut stat_string = String::new();
    stat_file.read_to_string(&mut stat_string).unwrap();
    let mut stat_vec = Vec::new();
    for i in stat_string.trim().split("\n") {
        let fields: Vec<&str> = i.split_ascii_whitespace().collect();
        if String::from(fields[0]).find("cpu").is_some() && fields[0] != "cpu" {
            stat_vec.push(ProcessStat {
                user: fields[1].parse::<i64>().unwrap(),
                nice: fields[2].parse::<i64>().unwrap(),
                system: fields[3].parse::<i64>().unwrap(),
                idle: fields[4].parse::<i64>().unwrap(),
                iowait: fields[5].parse::<i64>().unwrap(),
                irq: fields[6].parse::<i64>().unwrap(),
                softirq: fields[7].parse::<i64>().unwrap(),
            })
        }
    }
    Ok(stat_vec)
}
pub(super) fn get_total_processor_stat() -> Result<ProcessStat, String> {
    let mut stat_file = match File::open("/proc/stat") {
        Ok(o) => o,
        Err(err) => return Err(err.to_string()),
    };
    let mut stat_string = String::new();
    stat_file.read_to_string(&mut stat_string).unwrap();
    for i in stat_string.trim().split("\n") {
        let fields: Vec<&str> = i.split_ascii_whitespace().collect();
        if String::from(fields[0]).find("cpu").is_some() && fields[0] == "cpu" {
            return Ok(ProcessStat {
                user: fields[1].parse::<i64>().unwrap(),
                nice: fields[2].parse::<i64>().unwrap(),
                system: fields[3].parse::<i64>().unwrap(),
                idle: fields[4].parse::<i64>().unwrap(),
                iowait: fields[5].parse::<i64>().unwrap(),
                irq: fields[6].parse::<i64>().unwrap(),
                softirq: fields[7].parse::<i64>().unwrap(),
            });
        }
    }
    Err("Can't find the cpu line.".to_string())
}
pub fn get_processor(interval:time::Duration) -> Result<Vec<Processor>, String> {
    let start = get_processor_stat()?;
    std::thread::sleep(interval);
    let stop = get_processor_stat()?;
    if start.len() != stop.len() {
        return Err("Process stat numbers isn't the same.".to_string());
    }
    let mut usage = Vec::new();
    for i in 0..start.len() {
        let work_start = start[i].get_work();
        let work_stop = stop[i].get_work();
        let total_stop = stop[i].get_total();
        let total_start = start[i].get_total();
        usage.push((work_stop - work_start) as f32 / (total_stop - total_start) as f32);
    }
    let mut info_file = match File::open("/proc/cpuinfo") {
        Ok(o) => o,
        Err(err) => return Err(err.to_string()),
    };
    let mut info_string = String::new();
    info_file.read_to_string(&mut info_string).unwrap();
    let mut processors = Vec::new();
    for i in info_string.split("\n\n") {
        let mut processor = Processor {
            index: -1,
            vendor_id: "".to_string(),
            name: "".to_string(),
            freq: 0.0,
            cache_size: "".to_string(),
            usage: 0.0,
        };
        for j in i.split("\n") {
            let field: Vec<&str> = j.split(":").collect();
            if field.len() == 2 {
                match field[0].trim() {
                    "processor" => {
                        processor.index = field[1].trim().parse::<i32>().unwrap();
                    }
                    "vendor_id" => {
                        processor.vendor_id = field[1].trim().to_string();
                    }
                    "model name" => {
                        processor.name = field[1].trim().to_string();
                    }
                    "cpu MHz" => {
                        processor.freq = field[1].trim().parse::<f32>().unwrap();
                    }
                    "cache size" => {
                        processor.cache_size = field[1].trim().to_string();
                    }
                    _ => {}
                }
            }
        }
        if processor.index != -1 {
            processors.push(processor)
        }
    }
    if processors.len() != usage.len() {
        return Err("Processor stat len is not equal to processor count.".to_string());
    }
    for i in 0..processors.len() {
        processors[i].usage = usage[i];
    }
    Ok(processors)
}
#[test]
fn get_processor_test() {
    for i in 0..10 {
        get_processor(time::Duration::from_secs(1));
    }
}
