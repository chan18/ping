extern crate pnet;

use pnet::packet::icmp::IcmpTypes;
use pnet::packet::icmp::echo_request::MutableEchoRequestPacket;
use pnet::packet::icmp::IcmpPacket;
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::Packet;
use pnet::transport::{transport_channel, icmp_packet_iter, TransportChannelType::Layer3};
use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::time::Duration;
use std::thread;

/*
    7. application layer - data
    6. presentation layer - data
    5. session layer - data
    4. transport layer - segment, datagram
    3. network layer - packet - ICMP, IP 
    2. data link layer - frame
    1. physical layer - bit, symbol
*/
pub fn icmp() {

    // layer - 3  protocol
    // create a transport channel
    let protocol = Layer3(IpNextHeaderProtocols::Icmp);   

    /* socket text
        let ip = Ipv4Addr::new(255, 255, 255, 255);
        let sockaddr = SocketAddr::V4(SocketAddrV4::new(ip, 0));
        println!("{:?}", ip);
        println!("{:?}", sockaddr);
     */

    let (mut transport_sender, mut transport_receiver) = match transport_channel(1024, protocol) {
        Ok((transport_sender, transport_receiver)) => (transport_sender, transport_receiver),
        Err(e) => {
            eprintln!("An error occurred when creating the transport channel: {}", e);
            return;
        }
    };
    
    // get the desination
    let destination = Ipv4Addr::new(8, 8, 8, 8);

    // echo request buffer
    let mut echo_request_buffer = vec![0; 16];  

    // Create a mutable echo request packet from the buffer
    let mut echo_request_packet = MutableEchoRequestPacket::new(&mut echo_request_buffer).unwrap();

    
    /*
    Checksum

      The checksum is the 16-bit ones's complement of the one's
      complement sum of the ICMP message starting with the ICMP Type.
      For computing the checksum , the checksum field should be zero.
      If the total length is odd, the received data is padded with one
      octet of zeros for computing the checksum.  This checksum may be
      replaced in the future.+
    */
    let packet: IcmpPacket = IcmpPacket::new( &echo_request_packet.packet()).unwrap();        
    let checksum = pnet::packet::icmp::checksum(&packet);
  
    /*
         set the packet with values

         Type
            8 for echo message;
            0 for echo reply message.
        
        If code = 0, a sequence number to aid in matching echos and replies, may be zero.

          0                   1                   2                   3
    0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
   |     Type      |      Code     |          Checksum             |
   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
   |           Identifier          |        Sequence Number        |
   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+

     */
    echo_request_packet.set_icmp_type(IcmpTypes::EchoRequest);
    echo_request_packet.set_sequence_number(1);
    echo_request_packet.set_identifier(1);
    echo_request_packet.set_checksum(checksum);

    println!("{:?}",echo_request_packet);

    if let Err(e) = transport_sender.send_to(echo_request_packet, IpAddr::V4(destination)) {
        eprintln!("Failed to send packet: {}", e);
        return;
    }

    println!("Sent ICMP echo request to {}", destination);

    let mut iter = icmp_packet_iter(&mut transport_receiver);

    loop {
        match iter.next() {
            Ok((packet, addr)) => {
                println!("recived a packet");
                if let Some(ipv4_packet) = Ipv4Packet::new(packet.packet()) {
                    if ipv4_packet.get_next_level_protocol() == IpNextHeaderProtocols::Icmp {
                        if let Some(icmp_packet) = IcmpPacket::new(ipv4_packet.payload()) {
                            if icmp_packet.get_icmp_type() == IcmpTypes::EchoReply {
                                println!("Received ICMP echo reply from {}", addr);
                                break;
                            }
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("An error occurred while receiving packet: {}", e);
            }
        }

        thread::sleep(Duration::from_millis(100));
    }
}

