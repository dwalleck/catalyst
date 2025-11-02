# Rust Lessons Learned

**This document has been restructured for better navigation and usability.**

## 📍 New Location

The content has been split into multiple focused documents in `docs/rust-lessons/`:

**👉 [Start here: Navigation Guide →](docs/rust-lessons/index.md)**

---

## 🚀 Quick Links

### For Quick Lookup

- **[Quick Reference Checklist](docs/rust-lessons/quick-reference.md)** - All 20 lessons in scannable format (~400 lines)
  - Perfect for code review
  - Can be scanned in under 2 minutes
  - Each lesson links to detailed guide

### For Learning

- **[Learning Paths](docs/rust-lessons/index.md#-learning-paths)** - Beginner → Intermediate → Advanced
- **[Topic Index](docs/rust-lessons/index.md#-topic-index-alphabetical)** - Find specific topics quickly

### Deep-Dive Guides

| Guide | Topics | Lines |
|-------|--------|-------|
| **[Error Handling](docs/rust-lessons/error-handling-deep-dive.md)** | Option/Result patterns, expect vs unwrap, Path footguns | ~600 |
| **[File I/O Safety](docs/rust-lessons/file-io-deep-dive.md)** | Atomic writes, tempfile, parent dirs, testing | ~500 |
| **[Type Safety](docs/rust-lessons/type-safety-deep-dive.md)** | Constants → Enums progression, validation, suggestions | ~650 |
| **[Performance](docs/rust-lessons/performance-deep-dive.md)** | Loop optimizations, zero-copy abstractions | ~450 |
| **[Common Footguns](docs/rust-lessons/common-footguns.md)** | Path operations, TOCTOU races, borrow checker | ~400 |
| **[Fundamentals](docs/rust-lessons/fundamentals-deep-dive.md)** | Imports, tracing, CLI UX, duplicated logic | ~450 |

---

## 📦 What Changed

**Old structure:** Single 3012-line file
**New structure:** 8 focused documents with cross-references

**Benefits:**

- ✅ Faster lookup (scannable checklist)
- ✅ Better learning (clear progression)
- ✅ Less redundancy (~900 lines saved through consolidation)
- ✅ Easier maintenance (related topics together)

---

## 📚 Old Version

The previous monolithic version (v1.6) is archived at:

- **[RUST_LESSONS_LEARNED-v1.6-archive.md](RUST_LESSONS_LEARNED-v1.6-archive.md)**

---

## 🎯 Where to Go Next

**New to this guide?**
→ [Quick Reference Checklist](docs/rust-lessons/quick-reference.md)

**Need detailed examples?**
→ [Navigation Guide](docs/rust-lessons/index.md)

**Looking for specific topic?**
→ [Topic Index](docs/rust-lessons/index.md#-topic-index-alphabetical)

**Want to add a new lesson?**
→ [Contributing Guide](docs/rust-lessons/CONTRIBUTING.md)

---

**Version:** 2.0 (Restructured)
**Last Updated:** 2025-11-01
**Maintainer:** Catalyst Project Team
