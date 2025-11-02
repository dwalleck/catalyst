# Rust Lessons Learned - Navigation Guide

This directory contains Rust best practices and common mistakes discovered during code reviews, organized for easy navigation and learning.

## 🚀 Quick Start

**New to this guide?** Start with the [Quick Reference Checklist](quick-reference.md)

**Need detailed examples?** Jump to the relevant deep-dive guide below

**Made a specific mistake?** Check [Common Footguns](common-footguns.md)

---

## 📚 Documentation Structure

### Quick Reference (Start Here)

**[quick-reference.md](quick-reference.md)** - Scannable checklist of all rules

- ~400 lines covering 20 lessons
- Each lesson: Rule + Quick check + 1 example + Link to deep dive
- Perfect for code review or quick lookup
- Can be scanned in under 2 minutes

### Deep-Dive Guides (Comprehensive Learning)

| Guide | Topics Covered | Lines | Skill Level |
|-------|---------------|-------|-------------|
| **[Error Handling](error-handling-deep-dive.md)** | Option/Result patterns, expect vs unwrap, Path footguns | ~600 | Beginner/Intermediate |
| **[File I/O Safety](file-io-deep-dive.md)** | Atomic writes, tempfile, parent dirs, testing | ~500 | Intermediate |
| **[Type Safety](type-safety-deep-dive.md)** | Constants → Enums progression, validation, suggestions | ~650 | Intermediate |
| **[Performance](performance-deep-dive.md)** | Loop optimizations, zero-copy abstractions, profiling | ~450 | Intermediate/Advanced |
| **[Common Footguns](common-footguns.md)** | Path operations, TOCTOU races, borrow checker | ~400 | Mixed |
| **[Fundamentals](fundamentals-deep-dive.md)** | Imports, tracing, CLI UX, duplicated logic | ~450 | Beginner |

---

## 🎯 Learning Paths

### Path 1: Beginner (First PRs)

Recommended reading order for new Rust developers:

1. [Fundamentals Deep Dive](fundamentals-deep-dive.md)
   - Imports and code organization
   - Tracing subscribers
   - Avoiding duplicated logic

2. [Error Handling Deep Dive](error-handling-deep-dive.md) (Sections 1-2)
   - Option handling basics
   - When to use expect vs unwrap

3. [Quick Reference](quick-reference.md)
   - Scan all rules to build awareness

**Goal:** Avoid the most common beginner mistakes

---

### Path 2: Intermediate (Production Code)

For developers writing production-quality Rust:

1. [Error Handling Deep Dive](error-handling-deep-dive.md) (Complete)
   - All Option/Result patterns
   - Path operation footguns

2. [File I/O Safety Deep Dive](file-io-deep-dive.md)
   - Atomic writes
   - Safe file operations
   - Testing file I/O

3. [Type Safety Deep Dive](type-safety-deep-dive.md)
   - Constants → Enums progression
   - Validation patterns
   - User-friendly errors

4. [Common Footguns](common-footguns.md)
   - TOCTOU races
   - Borrow checker patterns

**Goal:** Write robust, safe production code

---

### Path 3: Advanced (Performance & Safety)

For optimizing critical code paths:

1. [Performance Deep Dive](performance-deep-dive.md)
   - Loop optimizations
   - Zero-copy abstractions
   - Profiling techniques

2. [Common Footguns](common-footguns.md)
   - Borrow checker with collections
   - Advanced safety patterns

3. Review all deep-dives for edge cases

**Goal:** Maximize performance while maintaining safety

---

## 🔍 Topic Index (Alphabetical)

Quick lookup by topic:

| Topic | Location |
|-------|----------|
| **Atomic File Writes** | [File I/O Deep Dive](file-io-deep-dive.md#atomic-file-writes) |
| **Borrow Checker (HashSet)** | [Common Footguns](common-footguns.md#borrow-checker-with-hashset) |
| **CLI User Feedback** | [Fundamentals Deep Dive](fundamentals-deep-dive.md#cli-user-feedback) |
| **Constants for Validation** | [Type Safety Deep Dive](type-safety-deep-dive.md#using-constants) |
| **"Did You Mean" Suggestions** | [Type Safety Deep Dive](type-safety-deep-dive.md#did-you-mean-suggestions) |
| **Duplicated Logic** | [Fundamentals Deep Dive](fundamentals-deep-dive.md#duplicated-logic) |
| **Enums vs Strings** | [Type Safety Deep Dive](type-safety-deep-dive.md#enums-vs-strings) |
| **Error Handling (Result)** | [Error Handling Deep Dive](error-handling-deep-dive.md#result-handling) |
| **expect() vs unwrap()** | [Error Handling Deep Dive](error-handling-deep-dive.md#expect-vs-unwrap) |
| **File I/O Testing** | [File I/O Deep Dive](file-io-deep-dive.md#testing-file-io) |
| **Immediate Validation** | [Type Safety Deep Dive](type-safety-deep-dive.md#immediate-validation) |
| **Imports (Redundant)** | [Fundamentals Deep Dive](fundamentals-deep-dive.md#redundant-imports) |
| **Loop Optimizations** | [Performance Deep Dive](performance-deep-dive.md#loop-optimizations) |
| **NamedTempFile** | [File I/O Deep Dive](file-io-deep-dive.md#namedtempfile) |
| **Option Handling** | [Error Handling Deep Dive](error-handling-deep-dive.md#option-types) |
| **Parent Directory Creation** | [File I/O Deep Dive](file-io-deep-dive.md#parent-directories) |
| **Path Operations** | [Common Footguns](common-footguns.md#path-operations) |
| **Performance Profiling** | [Performance Deep Dive](performance-deep-dive.md#profiling) |
| **TOCTOU Races** | [Common Footguns](common-footguns.md#toctou-races) |
| **Tracing Subscribers** | [Fundamentals Deep Dive](fundamentals-deep-dive.md#tracing-subscribers) |
| **TTY Detection** | [Fundamentals Deep Dive](fundamentals-deep-dive.md#tty-detection) |
| **Validation Patterns** | [Type Safety Deep Dive](type-safety-deep-dive.md) |
| **Zero-Copy Abstractions** | [Performance Deep Dive](performance-deep-dive.md#zero-copy) |

---

## 🏗️ Structure & Organization

### How Content is Organized

Each **deep-dive guide** follows this structure:

1. **Overview** - What this guide covers
2. **Sections** - Detailed lessons with examples
3. **Related Topics** - Cross-references to other guides
4. **Back to Quick Reference** - Link to return to checklist

### Consolidations & Generalizations

This guide consolidates related topics that were previously scattered:

**File I/O** (was 4 separate sections):

- Atomic writes + Parent dirs + NamedTempFile + Testing
- Now: Single comprehensive file I/O guide

**Type Safety** (was 4 separate sections):

- Constants + Enums + Validation + Suggestions
- Now: Journey from strings → type-safe design

**Error Handling** (was 3 separate sections):

- Option patterns + Path footguns + expect/unwrap/?
- Now: General patterns + specific applications

---

## 📖 How to Use This Guide

### For Code Review

1. Check [Quick Reference](quick-reference.md) against the PR
2. Link to specific rules when suggesting changes
3. Reference deep-dives for complex issues

### For Learning

1. Follow a [Learning Path](#-learning-paths) appropriate to your level
2. Read deep-dive guides in order
3. Try examples in your own code
4. Return to quick reference for reinforcement

### For Reference

1. Use [Topic Index](#-topic-index-alphabetical) to find specific topics
2. Jump directly to deep-dive sections
3. Follow cross-references for related topics

### For AI Agents

1. Scan [Quick Reference](quick-reference.md) to check code patterns
2. Jump to deep-dives when issue detected
3. Reference specific sections in feedback

---

## 🔗 Migration from Old Structure

**Previous file:** `RUST_LESSONS_LEARNED.md` (v1.6, 3012 lines)

**What changed:**

- Split into 8 focused documents
- Consolidated redundant content (file I/O, validation, errors)
- Added quick-reference checklist
- Improved navigation and cross-references
- Added learning paths

**Benefits:**

- ✅ Faster lookup (scannable checklist)
- ✅ Better learning (clear progression)
- ✅ Less redundancy (~900 lines saved)
- ✅ Easier maintenance (related topics together)

**Archived version:** `../RUST_LESSONS_LEARNED-v1.6-archive.md`

---

## 📝 Document Metadata

**Current Version:** 2.0 (Restructured)
**Last Updated:** 2025-11-01
**Maintainer:** Catalyst Project Team
**Based on:** 6 months of code reviews (Phases 0-2.6)

**Contributing:**

- **[📖 How to Add New Lessons →](CONTRIBUTING.md)** - Complete guide for adding lessons
- Add new lessons to appropriate deep-dive guide
- Update quick-reference when adding lessons
- Maintain cross-references
- Keep examples concise in quick-reference
- Provide detailed examples in deep-dives

---

**Ready to learn?** Start with the [Quick Reference Checklist →](quick-reference.md)
