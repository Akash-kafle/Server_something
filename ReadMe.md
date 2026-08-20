# quic-steer-rs

I set out to write an HTTP server from scratch.

One `cargo generate` later I am parsing Ethernet frames in a kernel VM that
took four hours to boot because GRUB forgot it existed. This is that project
now. HTTP may return eventually, on parole, if it behaves.

## what this actually is

A from-scratch dive into kernel-bypass packet processing in Rust, using
`aya` (XDP/eBPF) and eventually `xsk-rs` (AF_XDP). The short version: instead
of letting the Linux network stack fully process every packet before an
application ever sees it, XDP lets a program run *inside the kernel*, at the
earliest point a packet arrives, and decide what happens to it — pass it on,
drop it, or redirect it — before most of the usual kernel overhead even
happens. This project is me building that decision-making layer from
scratch, one piece at a time, to actually understand it rather than call a
library that does it for me.

Packets, not requests. Kernel bypass, not `curl`. If you came here looking
for routes and middleware, you are one layer too high — go back down to
layer 4 and try again.

## status (as of this writeup)

**Phase 1 — done.** Bounds-checked parsing of Ethernet, IPv4, IPv6, TCP, and
UDP headers, verified safe by the eBPF verifier before the program is even
allowed to load. Parsed fields stream out to userspace over a ring buffer
in real time — this is provably working against live traffic, not simulated.

**Phase 2 — in progress.** Turning "read the packet" into "decide what
happens to it." Building an ICMP filtering policy based on real
IETF guidance (RFC 4890 for ICMPv6, since IPv4 and IPv6 don't share
a type/code space and need different rules), wiring an actual `XDP_DROP`
decision instead of just logging.

**Phase 3–5 — not started, but scoped.** Stateful flow tracking (eBPF maps
keyed on the 5-tuple), AF_XDP for true kernel bypass into userspace, and
QUIC Connection ID parsing — the last of which sets up the actual problem
I want to talk about (see below).

## the actual engineering problem I'm chasing

QUIC connections are identified by a Connection ID, not the classic
5-tuple (src/dst IP + port). That matters because standard NIC hardware
steers packets to CPU cores by hashing the 5-tuple (RSS) — but QUIC
connections can *migrate* across IPs/ports on purpose (that's a real,
intentional QUIC feature), which means the hardware's steering assumption
breaks exactly when it matters most. Meanwhile, keeping one connection's
processing pinned to one CPU core — ideally one that's NUMA-local to
wherever its state lives in memory — is important for performance, or you
get cross-core cache-line bouncing.

The idea I want to build and am not yet sure how to best evaluate: use
`sched_ext` (a relatively new, pluggable Linux kernel scheduler class) to
keep a QUIC connection's processing pinned to a consistent, NUMA-aware
core based on its Connection ID, instead of leaving it to hardware RSS
hashing that doesn't know QUIC exists. I don't yet know what I don't know
here — hardware constraints, whether this is a solved problem I haven't
found, what's actually measurable on the setup I have access to.

## the stack

- **Rust** — because apparently regular difficulty wasn't enough
- **aya** — pure-Rust eBPF, no libbpf, no C toolchain, no mercy
- **QEMU/KVM** — a Debian netinst VM, currently the only thing standing
  between me and a bricked host machine
- **xsk-rs** — for when I'm ready to make the kernel truly optional
- **sched_ext** — the piece Phase 6 depends on; kernel version prerequisite
  not yet confirmed on my current setup

## faq

**Why not just use a framework?**
Because then I'd learn the framework instead of the thing underneath it.

**Isn't this overkill for an HTTP server?**
Yes. Enormously. That was the point.

**Does it work?**
Phase 1 does, provably. The rest is in progress — see status above.

## roadmap

- [x] XDP program that loads and doesn't get rejected by the verifier
- [x] Bounds-checked header parsing (Ethernet/IPv4/IPv6/TCP/UDP), streamed
      to userspace via ring buffer
- [ ] Real drop decisions (ICMP filtering policy, first real use of Phase 1's
      parsing)
- [ ] Stateful flow tracking via eBPF maps
- [ ] AF_XDP redirect, four rings, zero copies, several existential crises
- [ ] QUIC Connection ID parsing — the actual prerequisite for the idea below
- [ ] Connection ID–aware NUMA steering via `sched_ext` — the open problem
      I don't yet have a full plan for
- [ ] HTTP server — the thing this was originally supposed to be

## a note on the VM

It runs on `virtio-net`, which means zero-copy AF_XDP is a lie I've made
peace with. Everything here is copy-mode. I am learning the mechanics, not
setting a benchmark record. If you're reading this and know how to get real
zero-copy without PCI passthrough, you have my attention and my gratitude.

## license

Do what you want with it. It compiles more of the time now.
