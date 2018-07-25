#![allow(dead_code)]
extern crate clap;
extern crate ipnet;
extern crate prettytable;
extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate serde_yaml;


mod addrport;
mod configuration;
mod endpoint;


use addrport::AddrPort;
use clap::{Arg, App, SubCommand};
use configuration::Configuration;
use endpoint::{EndPoint, Router};
use ipnet::Ipv4Net;
use prettytable::{Table, cell::Cell, row::Row};
use std::net::Ipv4Addr;
use std::path::Path;
use std::process::exit;


fn example_configuration() -> Configuration {
    let router =
        Router::new(
            "vpn-router", "10.0.0.1".parse().unwrap(),
            AddrPort::new("vpn.com", 47654)
        );

    let mut configuration = Configuration::new(router);

    configuration.push_client(
        EndPoint::new("client-a", "10.0.1.1".parse().unwrap())
            .builder_push_allowed_ips("10.0.1.0/24".parse().unwrap())
            .builder_persistent_keepalive(Some(25)));

    configuration.push_client(
        EndPoint::new("client-b", "10.0.2.1".parse().unwrap())
            .builder_push_allowed_ips("10.0.2.0/24".parse().unwrap())
            .builder_persistent_keepalive(Some(25)));

    configuration
}



fn main () {
    let matches =
        App::new("wireguard-configure")
            .version("0.0.1")
            .author("Alex Eubanks <endeavor@rainbowsandpwnies.com>")
            .about("Simple wireguard configuration")
            .arg(Arg::with_name("config")
                .value_name("CONFIG")
                .required(true)
                .help("wireguard-configure configuration file"))
            .arg(Arg::with_name("example")
                .long("example")
                .help("Generate an example configuration file"))
            .arg(Arg::with_name("list")
                .short("l")
                .long("list")
                .conflicts_with("example")
                .help("List clients in this configuration"))
            .subcommand(
                SubCommand::with_name("add-client")
                    .about("Add a client to the configuration")
                .arg(Arg::with_name("name")
                    .short("n")
                    .long("name")
                    .value_name("NAME")
                    .required(true)
                    .help("Name for the new client"))
                .arg(Arg::with_name("internal-address")
                    .short("i")
                    .long("internal-address")
                    .value_name("INTERNAL_ADDRESS")
                    .required(true)
                    .help("Internal address for the new client"))
                .arg(Arg::with_name("persistent-keepalive")
                    .short("p")
                    .long("persitent-keepalive")
                    .value_name("PERSITENT_KEEPALIVE")
                    .help("Optional persitent keepalive for the client"))
                .arg(Arg::with_name("allowed-ips")
                    .short("a")
                    .long("allowed-ips")
                    .value_name("ALLOWED_IPS")
                    .help("An comma-delimited list of subnets for this client")))
            .subcommand(
                SubCommand::with_name("remove-client")
                    .about("Remove a client from the configuration")
                .arg(Arg::with_name("name")
                    .value_name("NAME")
                    .required(true)
                    .help("Name of client to remove")))
            .subcommand(
                SubCommand::with_name("router-config")
                    .about("Dump router config")
                .arg(Arg::with_name("linux-script")
                    .short("l")
                    .long("linux-script")
                    .help("Dump as bash script for linux")))
            .subcommand(
                SubCommand::with_name("client-config")
                    .about("Dump client config")
                .arg(Arg::with_name("name")
                    .value_name("NAME")
                    .required(true)
                    .help("Name of client to dump configuration for"))
                .arg(Arg::with_name("linux-script")
                    .short("l")
                    .long("linux-script")
                    .help("Dump as bash script for linux"))
                .arg(Arg::with_name("osx-script")
                    .short("o")
                    .long("osx-script")
                    .conflicts_with("linux-script")
                    .help("Dump as bash script for Mac OS X")))
            .get_matches();

    let filename = matches.value_of("config").unwrap();

    if matches.is_present("example") {
        let configuration = example_configuration();

        configuration.save(Path::new(filename));

        println!("Configuration saved to file");
    }

    else if matches.is_present("list") {
        let configuration = Configuration::open(Path::new(filename));

        let mut table = Table::new();

        table.add_row(Row::new(vec![
            Cell::new("Name"),
            Cell::new("Internal Address"),
            Cell::new("Allowed IPs")
        ]));

        table.add_row(Row::new(vec![
            Cell::new(configuration.router().name()),
            Cell::new(&format!("{}", configuration.router().internal_address())),
            Cell::new("")
        ]));

        for client in configuration.clients() {
            table.add_row(Row::new(vec![
                Cell::new(client.name()),
                Cell::new(&format!("{}", client.internal_address())),
                Cell::new(
                    &client.allowed_ips().iter()
                        .map(|ip| format!("{}", ip))
                        .collect::<Vec<String>>()
                        .join(","))
            ]));
        }

        table.printstd();
    }

    else if let Some(matches) = matches.subcommand_matches("remove-client") {
        let name = matches.value_of("name").unwrap();

        let mut configuration = Configuration::open(Path::new(filename));
        if !configuration.remove_client_by_name(name) {
            eprintln!("Failed to find and remove client {}", name);
            exit(1);
        }

        configuration.save(Path::new(filename));
        println!("Client {} removed", name);
    }

    else if let Some(matches) = matches.subcommand_matches("add-client") {
        let name = matches.value_of("name").unwrap();

        let mut configuration = Configuration::open(Path::new(filename));
        if configuration.clients().iter().any(|client| client.name() == name) {
            eprintln!("Client {} already exists", name);
            exit(1);
        }

        let internal_address: Ipv4Addr = matches.value_of("internal-address")
            .unwrap()
            .parse()
            .expect("Invalid internal address");

        let mut endpoint = EndPoint::new(name, internal_address);

        if let Some(keepalive) = matches.value_of("persistent-keepalive") {
            let keepalive: usize =
                keepalive.parse().expect("Invalid persistent keepalive");
            endpoint.set_persistent_keepalive(Some(keepalive));
        }

        if let Some(allowed_ips) = matches.value_of("allowed-ips") {
            for allowed_ip in allowed_ips.split(",") {
                let allowed_ip: Ipv4Net = allowed_ip.parse()
                    .expect("Invalid allowed ip");
                endpoint.push_allowed_ip(allowed_ip);
            }
        }

        configuration.push_client(endpoint);

        configuration.save(Path::new(filename));

        println!("Client added");
    }

    else if let Some(matches) = matches.subcommand_matches("router-config") {
        let configuration = Configuration::open(Path::new(filename));

        if matches.is_present("linux-script") {
            println!("cat > vpn.conf <<EOF");
        }

        println!("{}", configuration.router().interface());

        for client in configuration.clients() {
            println!("{}", client.peer());
        }

        if matches.is_present("linux-script") {
            println!("EOF");
            println!("ip link add dev wg0 type wireguard");
            println!("ip address add dev wg0 {}/32",
                configuration.router().internal_address());
            println!("wg setconf wg0 vpn.conf");
            println!("ip link set up dev wg0");

            configuration.clients()
                .iter()
                .flat_map(|client| client.allowed_ips())
                .for_each(|allowed_ip|
                        println!("ip route add {} dev wg0", allowed_ip));
        }
    }

    else if let Some(matches) = matches.subcommand_matches("client-config") {
        let configuration = Configuration::open(Path::new(filename));
        let name = matches.value_of("name").unwrap();

        let client = configuration.client_by_name(name)
            .expect(&format!("Could not find client {}", name));

        if matches.is_present("linux-script") || matches.is_present("osx-script") {
            println!("cat > vpn.conf <<EOF");
        }

        println!("{}", configuration.client_config(name).unwrap());

        if matches.is_present("linux-script") {
            println!("EOF");
            println!("ip link add dev wg0 type wireguard");
            println!("ip address add dev wg0 {}/32",
                client.internal_address());
            println!("wg setconf wg0 vpn.conf");
            println!("ip link set up dev wg0");

            println!("ip route add {} dev wg0",
                configuration.router().internal_address());

            configuration.all_allowed_ips()
                .iter()
                .for_each(|allowed_ip|
                    println!("ip route add {} dev wg0",
                        allowed_ip));
        }
        else if matches.is_present("osx-script") {
            println!("EOF");
            println!("sudo wireguard-go utun9");
            println!("sudo wg setconf utun9 vpn.conf");
            println!("sudo ifconfig utun9 inet {} 255.255.255.255 {}",
                client.internal_address(),
                configuration.router().internal_address());

            configuration.all_allowed_ips()
                .iter()
                .for_each(|allowed_ip|
                    if allowed_ip.prefix_len() == 32 {
                        println!("sudo route add {} -interface utun9",
                            allowed_ip.addr());
                    }
                    else {
                        println!("sudo route add -net {} -interface utun9",
                            allowed_ip);
                    });
        }
    }

    else {
        eprintln!("No command!");
        exit(1)
    }
}