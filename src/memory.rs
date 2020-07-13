use std::fs::File;
use std::io::Read;
use serde::{Deserialize, Serialize};
#[derive(Default,Debug,Serialize, Deserialize)]
pub struct Memory {
    pub total: i64,
    pub free: i64,
    pub used: i64,
    pub avaliable: i64,
    pub buffer_or_cached: i64,
    pub swap_total: i64,
    pub swap_used: i64,
    pub swap_free: i64,
}

pub fn get_memory() -> Result<Memory, String> {
    let mut file = match File::open("/proc/meminfo") {
        Ok(o) => o,
        Err(err) => return Err(err.to_string()),
    };
    let mut content = String::new();
    match file.read_to_string(&mut content){
        Ok(_)=>{}
        Err(err)=>{return Err(err.to_string())}
    };
    let mut memory=Memory::default();
    for i in content.split("\n") {
        let fields: Vec<&str> = i.split(":").collect();
        match fields[0].trim() {
            "MemTotal" => {
                memory.total = String::from(fields[1].trim())
                    .replace(" kB", "")
                    .parse::<i64>()
                    .unwrap();
            }
            "MemFree" => {
                memory.free = String::from(fields[1].trim())
                    .replace(" kB", "")
                    .parse::<i64>()
                    .unwrap();
            }
            "MemAvailable" => {
                memory.avaliable = String::from(fields[1].trim())
                    .replace(" kB", "")
                    .parse::<i64>()
                    .unwrap();
            }
            "Buffers" => {
                memory.buffer_or_cached += String::from(fields[1].trim())
                    .replace(" kB", "")
                    .parse::<i64>()
                    .unwrap();
            }
            "Cached" => {
                memory.buffer_or_cached += String::from(fields[1].trim())
                    .replace(" kB", "")
                    .parse::<i64>()
                    .unwrap();
            }
            "SReclaimable"=>{
                memory.buffer_or_cached += String::from(fields[1].trim())
                    .replace(" kB", "")
                    .parse::<i64>()
                    .unwrap();
            }
            "SwapTotal" => {
                memory.swap_total = String::from(fields[1].trim())
                    .replace(" kB", "")
                    .parse::<i64>()
                    .unwrap();
            }
            "SwapFree" => {
                memory.swap_free = String::from(fields[1].trim())
                    .replace(" kB", "")
                    .parse::<i64>()
                    .unwrap();
            }
            _ => {}
        }
        memory.used = memory.total - memory.buffer_or_cached - memory.free;
        memory.swap_used = memory.swap_total - memory.swap_free;
    }
    Ok(memory)
}
#[test]
fn get_memory_stat_test(){
    println!("{:?}",get_memory());
}