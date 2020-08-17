use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use std::str;
#[derive(Debug, Default, Copy, Clone)]
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
    #[serde(skip)]
    last_stat: ProcessStat,
    #[serde(skip)]
    latest_stat: ProcessStat,
}
pub struct Processors {}
impl Processor {
    pub fn refresh(&mut self) -> Result<(), String> {
        let stats = get_processor_stat()?;
        self.last_stat = self.latest_stat;
        self.latest_stat = stats[self.index as usize];
        Ok(())
    }
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
pub fn refresh_all(processors: &mut Vec<Processor>) -> Result<(), String> {
    let stats = get_processor_stat()?;
    if stats.len() != processors.len() {
        return Err("Stat's num is not equal to processor's num!".to_string());
    }
    for i in 0..processors.len() {
        processors[i].last_stat = processors[i].latest_stat;
        processors[i].latest_stat = stats[i];
        let total_diff =
            processors[i].latest_stat.get_total() - processors[i].last_stat.get_total();
        if total_diff == 0 {
            processors[i].usage = 0.0;
            continue;
        }
        processors[i].usage = (processors[i].latest_stat.get_work()
            - processors[i].last_stat.get_work()) as f32
            / total_diff as f32;
    }
    Ok(())
}
pub fn new() -> Result<Vec<Processor>, String> {
    let stats = get_processor_stat()?;
    let mut file = match File::open("/proc/cpuinfo") {
        Ok(o) => o,
        Err(err) => return Err(err.to_string()),
    };
    let mut content = String::new();
    match file.read_to_string(&mut content) {
        Ok(o) => o,
        Err(err) => return Err(err.to_string()),
    };
    let mut processors: Vec<Processor> = content
        .split("\n\n")
        .map(|part| {
            let fields: Vec<Vec<&str>> =
                part.lines().map(|line| line.split(":").collect()).collect();
            let mut processor = Processor::default();
            processor.index = -1;
            for field in fields {
                if field.len() != 2 {
                    continue;
                }
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
            processor
        })
        .filter(|processor| processor.index != -1)
        .collect();
    if processors.len() != stats.len() {
        return Err("Stat's num is not equal to processor's num!".to_string());
    }
    for i in 0..processors.len() {
        processors[i].latest_stat = stats[i];
        processors[i].usage = 0.0;
    }
    Ok(processors)
}
#[test]
fn processor_test() {
    let mut processors = new().unwrap();
    std::thread::sleep(std::time::Duration::from_secs(1));
    refresh_all(&mut processors).unwrap();
    for i in processors {
        assert_ne!(i.usage, 0.0 as f32);
    }
}
