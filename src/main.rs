use std::fs;
use std::io::Write;
use HockeySim::sim_engine;
use HockeySim::savestate::savedata::SaveInfo;
fn main() {


    
let mut info = SaveInfo::new();

sim_engine::begin_sim("Test_Sim")



}


fn write_to_file(path: &str, content: &str) {
    fs::write(path, content).expect("Failed to write file");
}
