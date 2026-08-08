# the plan (such as it is)

Written so I stop showing up to my own project like I've never seen it
before. Check boxes as they happen. Update this when reality disagrees with
it, which it will.

---

## Phase 0 — infrastructure

- [x] Debian VM booting, disk grown from a hilarious 5G to a less
      embarrassing 30G
- [x] Rust stable + nightly + `rust-src`, because apparently one toolchain
      was never going to be enough
- [x] `cargo-generate`, `bpf-linker` installed and behaving
- [x] `aya-template` skeleton builds clean
- [x] SSH stopped fighting me, `rsync` actually syncs
- [x] Program loads, attaches (`XdpMode::Skb`, virtio-net doesn't do native),
      logs every packet on `ens3`
- [x] Proved it works with `curl` from a second session, not just vibes

This phase is over. It is not coming back. If something breaks here again I
am allowed to be annoyed about it, briefly, then fix it and move on — this
was never the point.

---

## Phase 1 — read the packet

**Goal:** stop logging "received a packet" like some kind of coward. Read
actual bytes.

- [ ] Bounds-checked Ethernet read (14 bytes), branch on EtherType
- [ ] Bounds-checked IPv4 read, pull protocol + src/dst IP
- [ ] Log real fields, not vibes
- [ ] Bounds-checked UDP read (protocol 17), get ports
- [ ] Bounds-checked TCP read (protocol 6), get ports + flags
- [ ] Actually understand why each check exists, not just cargo-cult it

**Done when:** I can log the 5-tuple of anything crossing `ens3` without the
verifier telling me my pointer arithmetic is a personal attack.

**The actual hard part:** LLVM reordering or eating my bounds checks. If the
verifier rejects something that looks obviously fine, that's the bug —
check placement, not check existence.

---

## Phase 2 — make a decision,

**Goal:** a program that does something, instead of a very expensive packet
counter.

- [ ] Drop packets on one condition (ICMP, a port, whatever)
- [ ] Prove the drop is real, not imagined
- [ ] Add a second condition, branch cleanly instead of building an
      if-chain monument
- [ ] (optional) count drops/passes in a map — first contact with eBPF maps

**Done when:** I can point at the code and say exactly what it kills and
why, and back it up with real traffic.

---

## Phase 3 — maps, or teaching the kernel to remember things

**Goal:** stateful flow tracking. The actual prerequisite for anything past
this point.

- [ ] `HASH` map keyed on 5-tuple, count packets per flow
- [ ] Not embarrass myself on map key alignment/padding
- [ ] Swap to `LRU_HASH`, understand why eviction matters once flows aren't
      bounded
- [ ] Read the map from userspace while the XDP program runs — first real
      kernel-to-userspace state bridge

**Done when:** userspace can see live flow state the kernel side wrote.
This is the actual trick real load balancers use. Small scale, same idea.

---

## Phase 4 — AF_XDP, the deep end I signed up for

**Goal:** get packets into userspace without the kernel's TCP stack ever
finding out.

- [ ] Understand `XDP_REDIRECT` vs PASS/DROP
- [ ] `xsk-rs` basics: UMEM, one socket, one queue
- [ ] Fill/RX ring: a real packet reaches userspace
- [ ] TX/completion ring: something goes back out
- [ ] Internalize the ownership rule: a frame given to fill/tx doesn't get
      reused until it comes back via completion/rx. Break this once, learn
      it forever.

**Done when:** a packet goes NIC → XDP → userspace via AF_XDP → back out,
kernel socket stack never involved.

**Already accepted:** virtio-net means copy-mode, not real zero-copy. Same
mechanics either way. I'm learning the shape, not setting a speed record.

---

## Phase 5 — the trap I already warned myself about

Everything here is optional. Pick at most one. Do not chain them. This is
exactly the rabbit hole that started this whole thing.

- [ ] QUIC long-header parsing, 5-tuple routing only, skip CID for now
- [ ] QUIC short-header CID length problem — needs a real design call
      (QUIC-LB-style encoding, or a lookup)
- [ ] Multi-buffer/frags handling for packets that don't fit in one page
- [ ] The HTTP server. The original ask. Almost certainly its own separate,
      boring, normal project — not bolted onto any of this.

---

## how to use this

Top to bottom. Phase 3 and 4 assume 1 and 2 actually work, not "worked once
and I moved on." If stuck mid-phase, that's the right time to go bother
someone about it — bring the actual verifier error, not "phase 3 is hard."

Leave a one-line note under whatever phase I stop in, every session. Future
me does not remember what past me was thinking, and past me has been wrong
before (see: Phase 0).