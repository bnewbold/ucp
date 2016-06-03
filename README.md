
# ucp: UDT copy (very remote file transfer)

This is basically a clone of SSH's scp command that uses the UDT (UDP Data
Transport) over UDP instead of passing data through the TCP SSH connection. SSH
(and TCP) is used for authentication and setup, just like `mosh` does; the
server-side ucp process is not a daemon and is not running continuously.

The motivation for this tool is to get better bandwidth utilization over
high-latency links with mild packet loss, while remaining well-behaved netizen.

Just like `scp`, this program is based on the old `rcp` "protocol" and is very
simple. There is no session resumption, or extra checksums or integrity checks,
support for file metadata beyond old-school UNIX permissions, or anything like
that. There is a bunch of overhead sending small files, so if you have a lot of
those and a high-latency link you should probably `tar` things up first. There
also isn't any compression, so you might want to `gzip` that tarball.

### Status

Circa June 2016, this is very much a work in progress; see TODO. Only
getting/sending single files will ever work.

Running with crypto enabled (the default) slows things to a drag. Without
crypto, the tool is generally at least as fast as any other transfer tool (eg,
faster than scp, bbcp, or UTP).

### Dependencies

You need the Sodium (wrapper for NaCl) and UDT libraries and headers installed.
On Linux, this is, eg, libsodium-dev and libudt-dev. Unfortunately, the version
of libsodium from Debian Jessie won't work, you need the Stretch or newer
version.

Uses sodiumoxide instead of rust-crypto because there aren't online docs for
rust-crypto, and AFAIK Sodium and NaCl have been reviewed and rust-crypto has
not.

Uses rustc-serialize instead of serde, because serde seems more complicated
(both nightly/compiler API and a non-macro API?) and doesn't seem to support
base64.

### Usage

The command must be installed on both the local and remote machines.

To send a local file named `insurance.aes256` to the home directory of `robin`
on `files.the-nsa.org`:

    ucp insurance.aes256 robin@files.the-nsa.org:

To copy a complete directory of files from the directory "~/everything/" on the
remote server to a new folder `/tmp/img/` locally:

    ucp -r robin@files.the-nsa.org:everything /tmp

### rcp Protocol

The `rcp` protocol is described in a
[Jul 09, 2007 blog post by janp](https://blogs.oracle.com/janp/entry/how_the_scp_protocol_works),
which is mirrored as text in the `doc` folder of this repo.

