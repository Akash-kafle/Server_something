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

- [x] Bounds-checked Ethernet read (14 bytes), branch on EtherType
- [x] Bounds-checked IPv4 read, pull protocol + src/dst IP
- [x] Log real fields, not vibes
- [x] Bounds-checked UDP read (protocol 17), get ports
- [x] Bounds-checked TCP read (protocol 6), get ports + flags
- [x] Actually understand why each check exists, not just cargo-cult it

**Done when:** I can log the 5-tuple of anything crossing `ens3` without the
verifier telling me my pointer arithmetic is a personal attack.

**The actual hard part:** LLVM reordering or eating my bounds checks. If the
verifier rejects something that looks obviously fine, that's the bug —
check placement, not check existence.

---

## Phase 2 — make a decision

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

## Phase 5 — QUIC, the trap I already warned myself about

Pick this up once 1-4 are solid. This one has a real endpoint now (see
Phase 6), so it's less of a pure trap than it used to be — but still don't
start it early just because it sounds cooler than bounds-checking IP
headers.

- [ ] QUIC long-header parsing, 5-tuple routing only, skip CID for now
- [ ] QUIC short-header CID length problem — needs a real design call
      (QUIC-LB-style encoding, or a lookup)
- [ ] Multi-buffer/frags handling for packets that don't fit in one page
- [ ] Parse and log the actual Connection ID from both header forms —
      this is the thing Phase 6 needs to exist first

**Prerequisite reading before starting:** RFC 8999 (short version of the
header-form split) before RFC 9000 (the whole spec). See `DOCS.md`.

**The HTTP server. The original ask.** Almost certainly its own separate,
boring, normal project — not bolted onto any of this. Not forgotten,
just not this.

---

## Phase 6 — QUIC connection-aware NUMA steering with sched_ext

**The actual engineering problem, stated properly:** QUIC connections are
identified by Connection ID, not the classic 5-tuple, so hardware RSS
hashing can't reliably keep one connection's packets landing on the same
core — especially across connection migration, which is the whole point of
QUIC. Meanwhile connection-processing state ideally stays pinned to one
NUMA-local core the entire life of the connection, or you get exactly the
cross-core cache-line bouncing the work-stealing scheduler project already
had to solve, just one layer up. `sched_ext` is the piece that lets "keep
this connection's processing on this core, NUMA-aware" be a real,
swappable kernel scheduling policy instead of a `taskset` and a prayer.

This is not a weekend add-on. Treat it as its own project phase with its
own prerequisites, not a stretch goal bolted onto Phase 5.

**Prerequisites — check these before writing any code, not after:**

- [ ] `uname -r` on the VM — need 6.12+ for `sched_ext`
      (`CONFIG_SCHED_CLASS_EXT=y`). Debian 12 bookworm ships 6.1 by default.
      Almost certainly need backports or a newer kernel entirely. Check this
      FIRST, before sinking time into anything else in this phase.   
- [ ] `CONFIG_DEBUG_INFO_BTF=y` — without it, CO-RE relocations fail and
      every BPF scheduler refuses to load with an unhelpful error. Confirm
      before assuming a kernel bump alone is enough.   
- [ ] Decide up front: are you building the *mechanism* and reasoning about
      it correctly (fine in a single-node VM), or trying to actually
      *measure* a NUMA win (needs real multi-socket hardware — check what
      your host actually has before promising yourself a benchmark)   
- [ ] If staying in QEMU: `-numa node,...` can fake multiple vNUMA nodes for
      correctness testing, but won't produce meaningful latency numbers on
      single-socket consumer hardware. Know which one you're testing for.

**Build order, once prerequisites are actually satisfied:**

- [ ] Confirm CID parsing from Phase 5 works reliably first — this whole
      phase is built on trusting that extraction   
- [ ] XDP/CPUMAP redirect: steer packets to a specific CPU based on parsed
      CID instead of the NIC's hardware RSS hash   
- [ ] AF_XDP socket processing pinned per-connection (builds on Phase 4)   
- [ ] Read `github.com/sched-ext/scx` reference schedulers (`scx_simple`
      first, it's the one to actually understand, not skim) before writing
      a custom one   
- [ ] Write a minimal `sched_ext` policy: keep a given connection's
      processing task on its assigned core, NUMA-aware, don't let the
      default scheduler migrate it away   
- [ ] Only after the minimal version works: try to actually measure
      something — cache misses, cross-node memory access, whatever your
      setup can honestly show   

**Done when:** you can point at a specific connection, show which core its
packets get steered to, and show the scheduler keeping the processing
pinned there instead of the default scheduler bouncing it around.

**Honest framing:** this is the kind of problem people write blog posts or
papers about, not a checkbox. Don't rush it because Phase 5 finally got
QUIC parsing working — the kernel-version prerequisite alone might eat a
session on its own.

---

## how to use this

Top to bottom. Phase 3 and 4 assume 1 and 2 actually work, not "worked once
and I moved on." Phase 6 assumes Phase 5's CID parsing is solid, and has
its own hard prerequisite (kernel version) that has nothing to do with
skill and everything to do with checking `uname -r` before getting
attached to an idea. If stuck mid-phase, that's the right time to go
bother someone about it — bring the actual verifier error, not "phase 3
is hard."

Leave a one-line note under whatever phase I stop in, every session. Future
me does not remember what past me was thinking, and past me has been wrong
before (see: Phase 0).