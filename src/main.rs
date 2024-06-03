use pcap::Device;

// use std::env;
// use std::path::Path;

fn main() {

    // let dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    // println!(
    //     "cargo:rustc-link-search=native={}",
    //     Path::new(&dir).join("lib/x64").display()
    // );
   
    let mut cap = Device::lookup().unwrap().unwrap().open().unwrap();

    while let Ok(packet) = cap.next_packet() {
        println!("received packet! {:?}", packet);
    }

}


