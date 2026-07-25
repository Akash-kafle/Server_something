# http_server (kind of)

I set out to write an HTTP server from scratch.

One `cargo generate` later I am parsing Ethernet frames in a kernel VM that
took four hours to boot because GRUB forgot it existed. This is that project
now. HTTP may return eventually, on parole, if it behaves.

## what this actually is

A from-scratch dive into XDP + AF_XDP in Rust, using `aya`. Packets, not
requests. Kernel bypass, not `curl`. If you came here looking for routes and
middleware, you are one layer too high — go back down to layer 4 and try
again.

## status

Currently: Nothing here works yet.

## why

Turns out "build an HTTP server" and "learn kernel-bypass packet processing"
are unrelated tasks that share a directory out of spite. One thing led to
another. I regret nothing. I am mildly concerned about my time management.

## the stack

- **Rust** — because apparently regular difficulty wasn't enough
- **aya** — pure-Rust eBPF, no libbpf, no C toolchain, no mercy
- **QEMU/KVM** — a Debian netinst VM, currently the only thing standing
  between me and a bricked host machine
- **xsk-rs** — for when I'm ready to make the kernel truly optional

## faq (nobody asked, I'm answering anyway)

**Why not just use a framework?**
Because then I'd learn the framework instead of the thing underneath it.

**Isn't this overkill for an HTTP server?**
Yes. Enormously. That was the point.

**Does it work?**
See: status.

## roadmap

- [ ] XDP program that loads and doesn't get rejected by the verifier
- [ ] Actually parse a header without segfaulting the kernel's opinion of me
- [ ] AF_XDP redirect, four rings, zero copies, several existential crises
- [ ] QUIC connection ID routing (ambitious, deeply optional, mostly a trap)
- [ ] HTTP server — the thing this was originally supposed to be

## a note on the VM

It runs on `virtio-net`, which means zero-copy AF_XDP is a lie I've made
peace with. Everything here is copy-mode. I am learning the mechanics, not
setting a benchmark record. If you're reading this and know how to get real
zero-copy without PCI passthrough, you have my attention and my gratitude. Also please tell me how to do it.

## license

Do what you want with it. It probably doesn't compile anyway.