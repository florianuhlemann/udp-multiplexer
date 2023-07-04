//  UDP Multiplexer v0.3.0
//  Written in Rust by Florian Uhlemann

// Known Bugs:
// - None, at the moment

use std::env;
use std::net::{TcpListener, UdpSocket};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::io::{self, Write};


fn handle_multiplexing(udp_port:u16, tcp_port:u16, client_ip_address:std::net::IpAddr, server_ip_address:std::net::IpAddr) {

    // Variable Initalization
    let clients = Arc::new(Mutex::new(Vec::new()));
    let my_data = Arc::new(Mutex::new(0));
    let my_data1 = my_data.clone();
    let my_data2 = my_data.clone();


    // Spawn the TCP socket server thread
    let clients_tcp = clients.clone();
    let tcp_handle = thread::spawn(move || {
        match TcpListener::bind(format!("{}:{}", server_ip_address, tcp_port)) {
            Ok(listener) => {
                thread::sleep(Duration::from_millis(25));
                println!("A TCP server is listening on tcp://{server_ip_address}:{tcp_port}");
                for stream in listener.incoming() {
                    match stream {
                        Ok(stream) => {
                            let mut clients = clients_tcp.lock().unwrap();
                            println!("\rNew client connected: {}                    ", stream.peer_addr().unwrap());
                            io::stdout().flush().unwrap();
                            clients.push(stream.try_clone().unwrap());
                        }
                        Err(e) => {
                            eprintln!("TCP socket error: {}", e);
                        }
                    }
                }

            }
            Err(e) => {
                eprintln!("Error: {e} -> Failed to bind to tcp://{server_ip_address}:{tcp_port}");
                std::process::exit(1);
            }
        }
    });


    // Spawn the UDP listener thread
    let clients_udp = clients.clone();
    let udp_handle = thread::spawn(move || {
        match UdpSocket::bind(format!("{}:{}", client_ip_address, udp_port)) {
            Ok(socket) => {
                thread::sleep(Duration::from_millis(25));
                println!("A UDP client is listening on udp://{client_ip_address}:{udp_port}");
                loop {
                    let mut buf = [0; 8192];
                    let (amt, _) = socket.recv_from(&mut buf).unwrap();
                    let mut my_data_udp = my_data1.lock().unwrap();
                    *my_data_udp += amt;
                    drop(my_data_udp);
                    let mut clients_b = clients_udp.lock().unwrap();
                    clients_b.retain(|mut client| {
                        match client.write_all(&buf[..amt]) {
                            Ok(_) => true,
                            Err(_) => {
                                println!("\rFailed to send data client: it is no longer connected.");
                                // should I remove the missing client now? or is it automatically done?
                                false
                            }
                        }
                    });
                }
            }
            Err(e) => {
                eprintln!("Error: {e} -> Failed to bind to udp://{client_ip_address}:{udp_port}");
                std::process::exit(1);
            }

        }
    });


    // Basic User Interface
    let ui_handle = thread::spawn(move || {
        const START_DELAY : u8 = 250;
        const UPDATE_RATE : u8 = 100;
        let mut last_data : f64 = 0.0;
        thread::sleep(Duration::from_millis(START_DELAY as u64));
        loop {
            let my_data_ui = my_data2.lock().unwrap();
            let data_rate = (*my_data_ui as f64 - last_data) / (UPDATE_RATE as f64 / 1000.0) / 1024.0;
            last_data = *my_data_ui as f64;
            drop(my_data_ui);
            print!("\rReceiving bytes on UDP: {} ({: >8} KB/s)", last_data, data_rate.floor());
            io::stdout().flush().unwrap(); // flush the stdout buffer to immediately print the message
            thread::sleep(Duration::from_millis(UPDATE_RATE as u64));
        }
    });


    // Wait for the threads to finish
    tcp_handle.join().unwrap();
    udp_handle.join().unwrap();
    ui_handle.join().unwrap();
}


fn print_help() {
    println!("Usage:");
    println!(" -h                          Print this help message");
    println!(" -t <port number>            Set TCP port number");
    println!(" -u <port number>            Set UDP port number");
    println!(" -s <IPv4 address>           Set TCP IP address");
    println!(" -c <IPv4 address>           Set UDP IP address");
}


fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_help();
        return;
    }

    let mut tcp_port: Option<u16> = None;
    let mut udp_port: Option<u16> = None;
    let mut client_ip_address: Option<std::net::IpAddr> = None;
    let mut server_ip_address: Option<std::net::IpAddr> = None;

    for i in 1..args.len() {
        match args[i].as_str() {
            "-h" => {
                print_help();
                return;
            }
            "-t" => {
                if i + 1 < args.len() {
                    match args[i + 1].parse::<u16>() {
                        Ok(port) => {
                            if port > 0 {
                                tcp_port = Some(port);
                            } else {
                                println!("Invalid port number. Must be between 1 and 65535");
                                print_help();
                                return;
                            }
                        }
                        Err(_) => {
                            println!("Invalid port number. Must be an integer.");
                            print_help();
                            return;
                        }
                    }
                } else {
                    println!("'-t' flag requires a port number argument.");
                    print_help();
                    return;
                }
            }
            "-u" => {
                if i + 1 < args.len() {
                    match args[i + 1].parse::<u16>() {
                        Ok(port) => {
                            if port > 0 {
                                udp_port = Some(port);
                            } else {
                                println!("Invalid port number. Must be between 1 and 65535");
                                print_help();
                                return;
                            }
                        }
                        Err(_) => {
                            println!("Invalid port number. Must be an integer.");
                            print_help();
                            return;
                        }
                    }
                } else {
                    println!("'-u' flag requires a port number argument.");
                    print_help();
                    return;
                }
            }
            "-s" => {
                if i + 1 < args.len() {
                    match args[i + 1].parse::<std::net::IpAddr>() {
                        Ok(ip) => {
                            if ip.is_ipv4() {
                                server_ip_address = Some(ip);
                            } else {
                                println!("Invalid IP address format. Must be IPv4 address.");
                                print_help();
                                return;
                            }
                        }
                        Err(_) => {
                            println!("Invalid IP address format.");
                            print_help();
                            return;
                        }
                    }
                } else {
                    println!("'-s' flag requires an IP address argument.");
                    print_help();
                    return;
                }
            }

            "-c" => {
                if i + 1 < args.len() {
                    match args[i + 1].parse::<std::net::IpAddr>() {
                        Ok(ip) => {
                            if ip.is_ipv4() {
                                client_ip_address = Some(ip);
                            } else {
                                println!("Invalid IP address format. Must be IPv4 address.");
                                print_help();
                                return;
                            }
                        }
                        Err(_) => {
                            println!("Invalid IP address format.");
                            print_help();
                            return;
                        }
                    }
                } else {
                    println!("'-c' flag requires an IP address argument.");
                    print_help();
                    return;
                }
            }
            _ => {
                // Skip unknown flags and values of known flags
            }
        }
    }

    // Check that both values have been set correctly.
    if let (Some(my_udp_port), Some(my_tcp_port), Some(my_udp_addr), Some(my_tcp_addr)) = (udp_port, tcp_port, client_ip_address, server_ip_address) {
        handle_multiplexing(my_udp_port, my_tcp_port, my_udp_addr, my_tcp_addr);
    } else {
        println!("You need to provide the server TCP port, the client UDP port, the server IP address and the client IP address.");
        print_help();
    }
    

}
