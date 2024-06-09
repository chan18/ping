
extern crate pnet;

use pnet::datalink::{self, ChannelType, Config, NetworkInterface};

use pnet::packet::ethernet::{EtherTypes, EthernetPacket};
use pnet::packet::icmp::{echo_reply, echo_request, IcmpPacket, IcmpTypes};
use pnet::packet::ip::{IpNextHeaderProtocol, IpNextHeaderProtocols};
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::Packet;

use std::{net::IpAddr, time::Duration};



fn handle_ethernet_frame(interface: &NetworkInterface, ethernet: &EthernetPacket) {
    let interface_name = &interface.name[..];
    match ethernet.get_ethertype() {
        EtherTypes::Ipv4 => handle_ipv4_packet(interface_name, ethernet),
        // EtherTypes::Ipv6 => handle_ipv6_packet(interface_name, ethernet),
        // EtherTypes::Arp => handle_arp_packet(interface_name, ethernet),
        _ => println!(
            "[{}]: Unknown packet: {} > {}; ethertype: {:?} length: {}",
            interface_name,
            ethernet.get_source(),
            ethernet.get_destination(),
            ethernet.get_ethertype(),
            ethernet.packet().len()
        ),
    }
}
fn handle_ipv4_packet(interface_name: &str, ethernet: &EthernetPacket) {
    let header = Ipv4Packet::new(ethernet.payload());
    if let Some(header) = header {
        handle_transport_protocol(
            interface_name,
            IpAddr::V4(header.get_source()),
            IpAddr::V4(header.get_destination()),
            header.get_next_level_protocol(),
            header.payload(),
        );
    } else {
        println!("[{}]: Malformed IPv4 Packet", interface_name);
    }
}

fn handle_transport_protocol(
    interface_name: &str,
    source: IpAddr,
    destination: IpAddr,
    protocol: IpNextHeaderProtocol,
    packet: &[u8],
) {
    match protocol {
        IpNextHeaderProtocols::Icmp => {
            handle_icmp_packet(interface_name, source, destination, packet)
        }
        _ => println!(
            "[{}]: Unknown {} packet: {} > {}; protocol: {:?} length: {}",
            interface_name,
            match source {
                IpAddr::V4(..) => "IPv4",
                _ => "IPv6",
            },
            source,
            destination,
            protocol,
            packet.len()
        ),
    }
}

fn handle_icmp_packet(interface_name: &str, source: IpAddr, destination: IpAddr, packet: &[u8]) {
    let icmp_packet = IcmpPacket::new(packet);
    if let Some(icmp_packet) = icmp_packet {
        match icmp_packet.get_icmp_type() {
            IcmpTypes::EchoReply => {
                let echo_reply_packet = echo_reply::EchoReplyPacket::new(packet).unwrap();
                println!(
                    "[{}]: ICMP echo reply {} -> {} (seq={:?}, id={:?})",
                    interface_name,
                    source,
                    destination,
                    echo_reply_packet.get_sequence_number(),
                    echo_reply_packet.get_identifier()
                );
            }
            IcmpTypes::EchoRequest => {
                let echo_request_packet = echo_request::EchoRequestPacket::new(packet).unwrap();
                println!(
                    "[{}]: ICMP echo request {} -> {} (seq={:?}, id={:?})",
                    interface_name,
                    source,
                    destination,
                    echo_request_packet.get_sequence_number(),
                    echo_request_packet.get_identifier()
                );
            }
            _ => println!(
                "[{}]: ICMP packet {} -> {} (type={:?})",
                interface_name,
                source,
                destination,
                icmp_packet.get_icmp_type()
            ),
        }
    } else {
        println!("[{}]: Malformed ICMP Packet", interface_name);
    }
}



pub fn start() {
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
