[] = transmitter (transmitter -> receiver) packet
() = receiver (receiver -> transmitter) packet

transmittingDevice $ echolot transmit paper.pdf # readies packets and listens for initiation
transmittingDevice echolot: Readying Packets..
transmittingDevice echolot: Packets are Ready
transmittingDevice echolot: Listening for Receiver's Handshake Initiation

receivingDevice $ echolot receive # sends out initiation for handshake with transmitter
receivingDevice echolot: Establishing Handshake..

(
Initiation - reserved freq
)

transmittingDevice echolot: Received Initiation!
transmittingDevice echolot: Sending Response..

[
Response
- reserved freq
- size of Fileinfo
]

receivingDevice echolot: Handshake Established!

(
Agreement - reserved freq
)

transmittingDevice echolot: Handshake Established!
transmittingDevice echolot: Transmitting Fileinfo

[
Fileinfo
- filename
- filesize
- checksum
]

receivingDevice echolot: [========>----] Receiving Fileinfo
receivingDevice echolot: Pretty Print file info here + where file gets saved
receivingDevice echolot: Do you want to receive paper.pdf (y/n):

(
Confirmation - true/false
)

-> in case of false confirmation {
transmittingDevice echolot: Receiver does not want to receive paper.pdf!
transmittingDevice echolot: Listening for Receiver's Handshake Initiation
}

transmittingDevice echolot: [=>--------] Transmitting paper.pdf..

[
Filetransmission
- file as base64 string
- checksum
]

receivingDevice echolot: [========>----] Receiving File paper.pdf
receivingDevice echolot: Checking checksum..
receivingDevice echolot: Checksum verified!
receivingDevice echolot: Writing file..
receivingDevice echolot: Sucessfully received ~/Downloads/paper.pdf
