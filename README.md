# Distributed file system

A distributed p2p file system. Unlike BitTorrent, doesn't have a tracker server that tracks all peers in the network,
so every client in the system is equal.

## Links

- Torrent file format wikipedia: https://en.wikipedia.org/wiki/Torrent_file#:~:text=Torrent%20files%20are%20normally%20named,torrent%20.&text=A%20torrent%20file%20acts%20like,use%20of%20a%20BitTorrent%20client
- Torrent terms: https://en.wikipedia.org/wiki/Glossary_of_BitTorrent_terms#:~:text=A%20seed%20refers%20to%20a,other%20peers%20to%20download%20from.

## Taxonomy

**Peer** - a client that can connect into the network.

***.rfs file** (stands for rostyslav file system or rust file system) - a file that contains meta information about the state of the network at the time it was created. To get 
the latest info, new peer should connect to one of the peers mentioned in the file, and retrieve latest actual info
about state of the network. 
If no peers mentioned in the file are accessible, the file gets invalidated.

**Sharing** - a process of taking a file from local file system, splitting it into parts, and sending it into the peers in 
network. Before sending the exact file data, the peer sends a share request with information that contains the need size
that peer should have. Based on that, the accepting peers can either accept or reject the share request.

**Downloading** - a process of downloading a file from the network using metadata specified in the *.rfs file.

## System vision
System will have the peer business-logic component and UI component.

Business logic component will handle all the logic described in taxonomy.

UI will be built with one of the libraries (possibly https://www.egui.rs/). It should have 2 views. The first view
represent the state of the system (represent the peers and possibly the ping values for them, some other info about the 
peers). The second view should represent the view of downloaded files (akin to µtorrent)

## UI
1. Represent a list of metafiles present in the system with some info (name, size, downloaded or not, etc.)
2. Each of the file items can be opened and the info about available peers(seeds) is shown to the users with some refresh interval

### UI Wireframe
https://excalidraw.com/#json=qQRuctZkgNcJfE6hOZb3a,qpcrJ17YkC2SLbDHhUGqEQ

