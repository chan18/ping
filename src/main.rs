
extern crate pnet;

use pnet::{datalink::{self, ChannelType, Config, NetworkInterface}, packet::ethernet::EthernetPacket};

use std::time::Duration;

//mod ping;



fn main() {
    use pnet::datalink::Channel::Ethernet;

    let iface_name = "\\Device\\NPF_{5DE9F202-6019-45AC-8857-4DBE33D2DED1}";

    let interface_names_match = |iface: &NetworkInterface| iface.name == iface_name;

    // Find the network interface with the provided name
    let interfaces = datalink::interfaces();

    for interface in datalink::interfaces() {
        println!("Name: {}, Description: {:?}, mac: {:?}, ips: {:?}", interface.name, interface.description, interface.mac, interface.ips);
    }
           
    let interface = interfaces
                                        .into_iter()
                                        .filter(interface_names_match)
                                        .next()
                                        .unwrap_or_else(|| panic!("No such network interface: {}", iface_name));
                             

     println!("Found interface: {}", interface.name);

    let datalink_channel_config =  Config {
        write_buffer_size: 4096,
        read_buffer_size: 4096,
        read_timeout:  Some(Duration::from_secs(1)),
        write_timeout: Some(Duration::from_secs(1)),
        channel_type: ChannelType::Layer2,
        bpf_fd_attempts: 1000,
        linux_fanout: None,
        promiscuous: true,
        socket_fd: None,
    };

    // Create a channel to receive on
    let (_, mut receiver_datalinklayer) = 
    match datalink::channel(&interface, datalink_channel_config) {
        Ok(Ethernet(tx, receiver_datalinklayer)) => (tx, receiver_datalinklayer),
        Ok(_) => panic!("packetdump: unhandled channel type"),
        Err(e) => panic!("packetdump: unable to create channel: {}", e),
    };

    println!("Channel created successfully. Starting packet capture...");

    loop {
        println!("Waiting for packet...");
        match receiver_datalinklayer.next() {
            Ok(packet) => {
                println!("Packet received!");
                if let Some(ethernet_packet) = EthernetPacket::new(packet) {
                    handle_ethernet_frame(&interface, &ethernet_packet);
                } else {
                    eprintln!("Failed to parse Ethernet packet.");
                }
            },
            Err(e) => {
                panic!("packetdump: unable to receive packet: {}", e)
            },
        }

        println!("Capturing..");

        // To avoid tight loop, add a sleep
        std::thread::sleep(Duration::from_millis(100));
    }
}
