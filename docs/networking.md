# Networking Notes

## UDP

When is it a good time to build a UDP based protocol:

* Speed is paramount
* "Fire and forget" scenarios where lost packets will be self corrected
* Using a reliable protocol on top of UDP

Existing reliable UDP protocols:
* Enet
* UDT
* RakNet
* SCTP

Supposedly it's very difficult to implement a reliable UDP protocol yourself and
is highly encouraged to use an existing solution.

One approach for UDP based game protocols is to have two channels.

1. Low latency fire and forget channel
2. One reliable channel for mission critical state updates

You then run into an issue of timing and ensuring low latency fire and forget
aren't dependent on the mission critical updates coming through.

> Generally, for “fire and forget” scenarios you should consider implementing some kind of intra-packet ‘counter’ field, incremented for each packet sent, so out-of-order packets can be silently discarded on receiving side. As reordering window is usually quite limited, you may usually limit your ‘counter’ field to 2 bytes, though you need to handle wraparounds in this case.

**General UDP Tips**

* Restrict datagrams to ~508 bytes - avoid fragmentation, MTU issues, and possible firewall issues due to fragmentation of IP packets.

## Resources

* [UDP Networking App
    Protocol](http://ithare.com/64-network-dos-and-donts-for-game-engine-developers-part-i-client-side/)
