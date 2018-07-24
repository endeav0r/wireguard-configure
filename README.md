# wireguard-config

`wireguard-config` is a command-line utility to help manage wireguard configurations. It assumes a basic setup with one node acting as a, "Router," and several clients which connect and route traffic between the central router node. It allows you to generate and dump wireguard configurations, and bash scripts which also configure interfaces and routes.

You must have the commandline tool `wg` accessible through your path. This is used to automatically generate private/public wireguard keys.

Configurations are stored in yaml, and can be modified from the command line, or directly in the yaml file.

```
$ wireguard-configure --help
wireguard-configure 0.0.1
Alex Eubanks <endeavor@rainbowsandpwnies.com>
Simple wireguard configuration

USAGE:
    wireguard-configure [FLAGS] <CONFIG> [SUBCOMMAND]

FLAGS:
        --example    Generate an example configuration file
    -h, --help       Prints help information
    -l, --list       List clients in this configuration
    -V, --version    Prints version information

ARGS:
    <CONFIG>    wireguard-configure configuration file

SUBCOMMANDS:
    add-client       Add a client to the configuration
    client-config    Dump client config
    help             Prints this message or the help of the given subcommand(s)
    remove-client    Remove a client from the configuration
    router-config    Dump router config
```

# Example usage:

Generate an example configuration file, run `wireguard-config --example <filename>`

```
Alexs-MacBook-Pro:wireguard-configure endeavor$ target/debug/wireguard-configure --example test.conf
Configuration saved to file
Alexs-MacBook-Pro:wireguard-configure endeavor$ cat test.conf
---
router:
  name: "vpn-router"
  private_key: "ADsIErTzl7FaGDI614/MM6Y4YL+edr6v1ls314Fx4Vc="
  public_key: "560oUL8qMUbEFcQRys3tm/IbO8DPz96Oy6xrVlPuIjk="
  external_address:
    address: vpn.com
    port: 47654
  internal_address: 10.0.0.1
  allowed_ips:
    - 10.0.0.0/24
  persistent_keepalive: ~
clients:
  - name: "client-a"
    private_key: "6AXhGpbF36uRQNK3kt8SIwd1WJSGrfsdEnj89SArfls="
    public_key: "QEtcp4V4c79HH1aCGpZy237k96HU0thzHD66100upTQ="
    external_address: ~
    internal_address: 10.0.1.1
    allowed_ips:
      - 10.0.1.0/24
    persistent_keepalive: 25
  - name: "client-b"
    private_key: "8EzIJ2g/8xq24d5dvLXTJjNhJKyjQ8Yzg0E5mWhKKFs="
    public_key: "TwUOO10hyrzdwGZAZoFS5yfPsaVVnVYEJWTtLMD+d2M="
    external_address: ~
    internal_address: 10.0.2.1
    allowed_ips:
      - 10.0.2.0/24
    persistent_keepalive: 25Alexs-MacBook-Pro:wireguard-configure endeavor$ 
```

We can add another client with the `add-client` subcommand.

```
$ wireguard-configure test.conf add-client --help
wireguard-configure-add-client 
Add a client to the configuration

USAGE:
    wireguard-configure add-client [OPTIONS] --internal-address <INTERNAL_ADDRESS> --name <NAME>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -a, --allowed-ips <ALLOWED_IPS>                    An comma-delimited list of subnets for this client
    -i, --internal-address <INTERNAL_ADDRESS>          Internal address for the new client
    -n, --name <NAME>                                  Name for the new client
    -p, --persitent-keepalive <PERSITENT_KEEPALIVE>    Optional persitent keepalive for the client

$ wireguard-configure test.conf add-client --name test-net -a 10.0.3.0/24 -i 10.0.3.1 -p 25
Client added
$ wireguard-configure test.conf --list
+------------+------------------+-------------+
| Name       | Internal Address | Allowed IPs |
+------------+------------------+-------------+
| vpn-router | 10.0.0.1         | 10.0.0.0/24 |
+------------+------------------+-------------+
| client-a   | 10.0.1.1         | 10.0.1.0/24 |
+------------+------------------+-------------+
| client-b   | 10.0.2.1         | 10.0.2.0/24 |
+------------+------------------+-------------+
| test-net   | 10.0.3.1         | 10.0.3.0/24 |
+------------+------------------+-------------+
```

If you just want a single entrypoint into the network, with no subnet, simply leave that option out. This is good for single clients.

```
$ wireguard-configure test.conf add-client --name test-net2 -i 10.0.10.10
Client added
$ wireguard-configure test.conf --list
+------------+------------------+---------------+
| Name       | Internal Address | Allowed IPs   |
+------------+------------------+---------------+
| vpn-router | 10.0.0.1         | 10.0.0.0/24   |
+------------+------------------+---------------+
| client-a   | 10.0.1.1         | 10.0.1.0/24   |
+------------+------------------+---------------+
| client-b   | 10.0.2.1         | 10.0.2.0/24   |
+------------+------------------+---------------+
| test-net   | 10.0.3.1         | 10.0.3.0/24   |
+------------+------------------+---------------+
| test-net2  | 10.0.10.10       | 10.0.10.10/32 |
+------------+------------------+---------------+
```

We can now dump ready-to-go configs.

```
$ wireguard-configure test.conf router-config --linux-script
cat > vpn.conf <<EOF
[Interface]
# name: vpn-router
PrivateKey = ADsIErTzl7FaGDI614/MM6Y4YL+edr6v1ls314Fx4Vc=
ListenPort = 47654
[Peer]
# client-a
PublicKey = QEtcp4V4c79HH1aCGpZy237k96HU0thzHD66100upTQ=
AllowedIPs = 10.0.1.0/24
[Peer]
# client-b
PublicKey = TwUOO10hyrzdwGZAZoFS5yfPsaVVnVYEJWTtLMD+d2M=
AllowedIPs = 10.0.2.0/24
[Peer]
# test-net
PublicKey = bZIZkHc8vKjT9oeuVtEOYMbR0bncK23m1DxVuch8SVo=
AllowedIPs = 10.0.3.0/24
[Peer]
# test-net2
PublicKey = 5VXegPNsoWLXp0sNdy0A2UovRXM0xt3lSL7UmsXtISs=
AllowedIPs = 10.0.10.10/32
EOF
ip link del dev wg0
ip link add dev wg0 type wireguard
ip address add dev wg0 10.0.0.1/32
ip link set up dev wg0
route add 10.0.1.0 255.255.255.0 dev wg0
route add 10.0.2.0 255.255.255.0 dev wg0
route add 10.0.3.0 255.255.255.0 dev wg0
route add 10.0.10.10 255.255.255.255 dev wg0
```

```
$ wireguard-configure test.conf client-config test-net
[Interface]
# name: test-net
PrivateKey = yDLYWiwOjO5OUv+TpGuLlAJWgI3u1+C3x4uG2YUcpH8=
[Peer]
# vpn-router
PublicKey = 560oUL8qMUbEFcQRys3tm/IbO8DPz96Oy6xrVlPuIjk=
Endpoint = vpn.com:47654
AllowedIPs = 10.0.0.0/24
Alexs-MacBook-Pro:wireguard-configure endeavor$ target/debug/wireguard-configure test.conf client-config test-net --linux-script
cat > vpn.conf <<EOF
[Interface]
# name: test-net
PrivateKey = yDLYWiwOjO5OUv+TpGuLlAJWgI3u1+C3x4uG2YUcpH8=
[Peer]
# vpn-router
PublicKey = 560oUL8qMUbEFcQRys3tm/IbO8DPz96Oy6xrVlPuIjk=
Endpoint = vpn.com:47654
AllowedIPs = 10.0.0.0/24
EOF
ip link del dev wg0
ip link add dev wg0 type wireguard
ip address add dev wg0 10.0.0.1/32
ip link set up dev wg0
route add 10.0.1.0 255.255.255.0 dev wg0
route add 10.0.2.0 255.255.255.0 dev wg0
route add 10.0.10.10 255.255.255.255 dev wg0
```