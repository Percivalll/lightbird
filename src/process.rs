use std::fs::File;
use std::io::Read;
use std::str;
use crate::processor;
#[derive(Debug)]
pub struct Process{
    pub processor_usage_with_children:f32,
    pub rss:String,
    pub pid:&'static str
}
impl Process{
    pub fn new(pid:&'static str) -> Self{
        Process{
            processor_usage_with_children:get_process_usage(pid),
            rss:get_rss(pid),
            pid:pid
        }
    }
    pub fn current()->Self{
        Process{
            processor_usage_with_children:get_process_usage("self"),
            rss:get_rss("self"),
            pid:"self"
        }
    }
    
}
fn get_rss(pid:&'static str)->String{
    let mut file=File::open(String::from("/proc/")+pid+"/status").unwrap();
    let mut content =String::new();
    file.read_to_string(&mut content).unwrap();
    let lines:Vec<&str> =content.split("\n").collect();
    let fields:Vec<&str> =lines[21].split(":").collect();
    fields[1].trim().to_string()
}
fn get_process_cpu_usage_with_children(pid:&'static str)->i64{
    let mut file=File::open(String::from("/proc/")+pid+"/stat").unwrap();
    let mut content =String::new();
    file.read_to_string(&mut content).unwrap();
    let fields:Vec<&str> =content.split_ascii_whitespace().collect();
    fields[13].parse::<i64>().unwrap()+fields[15].parse::<i64>().unwrap()+fields[14].parse::<i64>().unwrap()+fields[16].parse::<i64>().unwrap()
}
fn get_process_usage(pid:&'static str)->f32{
    let total_start=processor::get_total_processor_stat().unwrap();
    let process_start=get_process_cpu_usage_with_children(pid);
    let processor_num=processor::get_processors().unwrap().len();
    let total_stop=processor::get_total_processor_stat().unwrap();
    let process_stop=get_process_cpu_usage_with_children(pid);
    processor_num as f32*(process_stop-process_start)as f32/(total_stop.get_total()-total_start.get_total())as f32
    
}
