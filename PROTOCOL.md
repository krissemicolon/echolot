[] = transmitter (transmitter -> receiver) packet
() = receiver (receiver -> transmitter) packet

transmittingDevice $ echolot transmit paper.pdf
transmittingDevice echolot: Audio Input Device: MacBook Microphone
transmittingDevice echolot: Audio Output Device: MacBook Speakers
transmittingDevice echolot: Readying Packets..
transmittingDevice echolot: Packets are Ready

receivingDevice $ echolot receive # sends out initiation for handshake with transmitter
receivingDevice echolot: Establishing Handshake..

[
Fileinfo
- Preamble

- filename
- filesize
- checksum
  ]

receivingDevice echolot: [Spinner] Receiving Fileinfo
receivingDevice echolot: Pretty Print file info here + where file gets saved
receivingDevice echolot: Do you want to receive paper.pdf (y/n):

(
Confirmation
- Preamble

- true/false
)

-> in case of false confirmation {
transmittingDevice echolot: Receiver does not want to receive paper.pdf!
transmittingDevice echolot: Listening for Receiver's Handshake Initiation
}

transmittingDevice echolot: [=>--------] Transmitting paper.pdf..

[
Filetransmission
- Preamble

- file encoded in Codec
- checksum
  ]

receivingDevice echolot: [========>----] Receiving File paper.pdf
receivingDevice echolot: Checking checksum..
receivingDevice echolot: Checksum verified!
receivingDevice echolot: Writing file..
receivingDevice echolot: Sucessfully received ~/Downloads/paper.pdf
