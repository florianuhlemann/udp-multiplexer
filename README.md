# UDP Multiplexer

This is a simple tool that allows you to listen to any incoming UDP packages (bind on a single IP address and port) and provide a TCP server for many clients to connect to, and listen to the same data packages.

> ## Usage:
> + -h                          Print this help message
> + -t <port number>            Set TCP port number
> + -u <port number>            Set UDP port number
> + -s <IPv4 address>           Set TCP IP address
> + -c <IPv4 address>           Set UDP IP address

> Example: ./udp-multiplexer -t 2000 -u 2001 -s 127.0.0.1 -c 192.168.178.1

If you find any issues with this application, please feel free to reach out to me and / or submit an issue note.