# docs, discussion, reference

Collected links, organized by phase. Start here, expand as the project gets
more complicated. Add to this as you find good stuff — don't let it go
stale.

---

## Phase 1 — parsing, verifier, bounds checks

- **Aya Book** — https://aya-rs.dev/book/
  The actual manual. Read "Getting Started" if you haven't, then whatever
  section matches what you're stuck on.
- **Aya API docs (userspace)** — https://docs.rs/aya
- **Aya API docs (eBPF-side, `aya_ebpf`)** — https://docs.rs/aya-ebpf
- **Aya main repo** — https://github.com/aya-rs/aya
- **awesome-aya** — https://github.com/aya-rs/awesome-aya
  Curated list of real projects built with Aya. Worth skimming for how
  other people structure header parsing.
- **BPF maps overview (kernel docs)** — https://docs.kernel.org/bpf/maps.html
  General map concepts, not Aya-specific — useful once code starts asking
  "what map type do I actually want."

---

## Phase 2/3 — maps, flow tracking, decisions

- **BPF_MAP_TYPE_HASH + LRU variants (kernel docs)** —
  https://docs.kernel.org/bpf/map_hash.html
  Explains LRU eviction mechanics directly from source — read this before
  assuming LRU behaves like a simple "oldest out" cache, it doesn't quite.
- **BPF_MAP_TYPE_LRU_HASH (eBPF Docs)** —
  https://docs.ebpf.io/linux/map-type/BPF_MAP_TYPE_LRU_HASH/
  More digestible walkthrough of the active/inactive/free list mechanism.
- **Katran — Meta's XDP-based L4 load balancer** —
  https://github.com/facebookincubator/katran
  C++, not Rust, but this is the real reference architecture for
  flow-tracking + LRU eviction at production scale. Worth reading the
  connection-tracking code even if you never touch C++ for this project.
- **Meta's Katran announcement post** —
  https://engineering.fb.com/2018/05/22/open-source/open-sourcing-katran-a-scalable-network-load-balancer/
  Higher-level explanation of *why* they built it this way — good context
  before diving into source.

---

## Phase 4 — AF_XDP

- **AF_XDP kernel docs** — https://docs.kernel.org/networking/af_xdp.html
  The actual spec: UMEM, fill/RX/TX/completion rings, zero-copy vs copy
  mode, XSKMAP. Read this before touching xsk-rs — the crate docs assume
  you already know these terms.
- **AF_XDP — eBPF Docs (more digestible overview)** —
  https://docs.ebpf.io/linux/concepts/af_xdp/
- **xsk-rs (the Rust crate you'll actually use)** —
  https://github.com/DouglasGray/xsk-rs
- **xsk-rs API docs** — https://docs.rs/xsk-rs
  Pay attention to the ownership-invariant warning in the crate docs — a
  frame given to fill/tx queue can't be reused until it comes back via
  completion/rx queue. This is the whole ballgame for Phase 4.
- **afxdp-rs (alternative crate, libbpf-based)** —
  https://github.com/aterlo/afxdp-rs
  Different API shape than xsk-rs — worth a skim for comparison, not
  necessarily worth switching to.
- **"Recapitulating AF_XDP" (blog, conceptual overview)** —
  https://hpnpl.net/posts/recapituatling-af-xdp/
  Good plain-English pass before or after the kernel docs, whichever
  order sticks better for you.

---

## Phase 5 — QUIC / stretch goals (only if you actually get here)

- **QUIC-LB draft (expired, but this is the real spec for CID-based
  routing)** — https://datatracker.ietf.org/doc/draft-ietf-quic-load-balancers/
  Note: this draft expired and is no longer active — read it as the
  design reference it is, not as a "currently standardized" thing. Solves
  exactly the short-header CID-length problem from the roadmap.
- **F5's reference implementation of QUIC-LB (C)** —
  https://github.com/F5Networks/quic-lb
- **RFC 9000 — QUIC transport spec** —
  https://www.rfc-editor.org/rfc/rfc9000
  The actual wire format for long/short headers, connection IDs, packet
  types. Ground truth if the QUIC-LB draft gets confusing.

---

## General / always useful

- **Cilium's eBPF guide** — referenced directly by the Aya book's own
  prerequisites section as the recommended eBPF background reading if
  Aya's docs assume too much. Search "Cilium eBPF guide" if the direct
  link goes stale.
- **ebpf.io docs** — https://docs.ebpf.io/
  General eBPF reference, not Rust/Aya-specific — good for "what does
  this kernel concept actually mean" questions that aren't about syntax.

---

## how to use this

Add a link here the moment you find something genuinely useful mid-session
— don't rely on remembering to come back and do it later. If a link goes
stale or turns out to be wrong, delete it, don't just leave it to rot.