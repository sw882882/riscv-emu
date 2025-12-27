# RISC-V RV64 Educational Emulator – Project Specification

## 1. Project Goals

This project implements a **RISC-V RV64 system emulator with a timing-aware microarchitectural model**, designed primarily for **educational purposes** and secondarily as a **marketable portfolio project**.

The goals are:
- Gain deep, hands-on understanding of modern CPU architecture
- Learn low-level systems programming in Rust (with C-like discipline)
- Implement MMU, privilege modes, interrupts, and a memory hierarchy
- Be held to an external correctness standard (RISC-V spec + Linux boot)
- Explore architectural trade-offs through measurement, not intuition

This is **not** a cycle-perfect hardware emulator and **not** a game console emulator. Accuracy is prioritized where it affects architectural understanding and software correctness.

---

## 2. Non-Goals (Explicitly Out of Scope)

To control scope, the following are *explicitly excluded*:
- Multi-core support
- Floating point (F/D extensions)
- Vector extensions
- GPU or graphics output
- Disk, filesystem, or networking devices
- Cycle-exact modeling of real hardware
- Security extensions (PMP beyond minimal permissive configuration required for Linux boot)

These may be considered future work but are not part of the core project.

---

## 3. Target ISA and Privilege Model

### 3.1 Instruction Set Architecture

- **ISA**: RISC-V RV64IMAC
  - RV64I: Base integer ISA
  - M: Multiply/divide
  - A: Atomics (required for Linux)
  - C: Compressed instructions (optional but recommended)

### 3.2 Privilege Modes

- **Machine mode (M)**
- **Supervisor mode (S)**

User mode (U) is not required initially; Linux may run user processes in S-mode.
(User mode (U) is not required for initial Linux bring-up; basic U-mode support may be added later if required.)

### 3.3 Endianness

- Little-endian

---

## 4. Execution Model

### 4.1 Core Model

- Single-core, in-order execution
- Single-issue pipeline
- Architecturally precise exceptions

The execution model separates **functional correctness** from **timing modeling**.

### 4.2 Pipeline Stages (Logical)

1. Fetch
2. Decode
3. Execute
4. Memory
5. Writeback

The pipeline is modeled conceptually; instructions may still execute in a step-based loop, with timing accumulated separately.

---

## 5. Timing Model

The emulator maintains a **cycle counter** that advances according to architectural events.

### 5.1 Base Latencies

| Operation | Cycles |
|---------|--------|
| ALU ops | 1 |
| Branch (correct) | 1 |
| Branch mispredict | +3 |
| Load (L1 hit) | 2 |
| Store (L1 hit) | 1 |
| Cache miss | +N (configurable) |
| Page walk | +M (configurable) |

These values are not intended to match real hardware, only to allow meaningful comparison.

### 5.2 Stalls and Exceptions

- Cache misses stall execution
- TLB misses stall until page walk completes
- Exceptions:
    - Flush the conceptual pipeline
    - Do **not** partially commit instruction side effects
    - Charge timing deterministically according to fault point
	    - Timing effects are not architecturally visible and do not affect functional correctness.

---
## 6. Memory System

### 6.1 Physical Memory

- Flat physical address space
- Configurable RAM size (default: 256 MiB)

### 6.2 Virtual Memory

- **Sv39** virtual memory
- 4 KiB pages
- 3-level page tables

### 6.3 TLB (Translation Lookaside Buffers)

- Separate instruction and data TLBs
- 32 entries each
- Fully associative
- LRU replacement
- Flushed on `satp` changes and relevant SFENCE instructions

### 6.4 Cache

- L1 data cache only (initially)
- 32 KiB
- 64-byte cache lines
- Direct-mapped
	- Cache coherence is not modeled; single-core execution assumes no external coherence requirements.
- Write-back, write-allocate
Instruction cache may be added later but is not required initially.

---

## 7. Devices and Platform Model

The emulator models a **QEMU-style RISC-V virt platform**.

### 7.1 Required Devices

#### UART
- 16550-compatible subset
- Console output only
#### CLINT
- Timer interrupts (`mtime`, `mtimecmp`)
- Software interrupts (IPIs)
#### PLIC
- Minimal implementation
- Single interrupt source
- Fixed priority
- Claim/complete semantics sufficient for Linux
### 7.2 Memory Map

- Follows standard virt conventions
- Defined explicitly in code
- Exposed to the kernel via a generated Device Tree Blob (DTB)

---
## 8. Boot Process

### 8.1 Kernel Loading

- Emulator loads a **RISC-V Linux kernel ELF** directly into memory
- OpenSBI may be used temporarily as a debugging aid but is not part of the modeled system.
- Long-term goal: boot Linux without external firmware

### 8.2 Initial CPU State

- hartid set to 0
- a0 = hartid
- a1 = pointer to device tree blob (DTB)
- PC set to kernel entry point
- CPU starts in Machine Mode

### 8.3 Trap Delegation
- Traps originate in M-mode
- Selected interrupts and exceptions are delegated to S-mode using:
    - `medeleg`
    - `mideleg`
Correct delegation behavior is required for Linux boot.
### 8.4 Userland
- BusyBox-based initramfs
- Statically linked
---
## 9. CSR (Control & Status Registers) Coverage

### 9.1 Required CSRs

At minimum:
- `mstatus`, `sstatus`
- `mtvec`, `stvec`
- `mepc`, `sepc`
- `mcause`, `scause`
- `satp`
- `cycle`, `time`
- Interrupt enable/pending CSRs

Unimplemented CSRs either return zero or trap with illegal instruction exceptions, as permitted by the RISC-V spec.
- Linux expects some CSRs to read as zero or WARL not trap (E.G. `mvendorid`, `marchid`, `mimpid`)

---
## 10. Correctness and Testing Strategy

### 10.1 Instruction-Level Tests

- Use official `riscv-tests` for RV64IMAC
- Tests must pass before Linux boot debugging

### 10.2 Integration Testing

- Linux kernel boot acts as a system-level test
- Kernel panics, silent failures or hangs are treated as emulator bugs

### 10.3 Architectural Validation

- Microbenchmarks validate:
    - Cache capacity effects
    - TLB miss penalties
    - Page walk behavior
- Observed timing trends must match architectural expectations

---
## 11. Debug and Bring-Up Support

To support development and debugging:
- Instruction trace logging
- Optional single-step execution
- CSR and register dumps on trap or panic
- Deterministic replay through fixed execution order and seeded randomness (no record/replay system)

---

## 12. Implementation Language and Style

### 12.1 Language

- Primary language: **Rust**
- Optional C for isolated performance-critical components

### 12.2 Rust Style Constraints

- Minimal use of traits
- No async
- Explicit state passing
- Avoid global mutable state
- Interior mutability limited to device models

The code should resemble structured C, with Rust used for safety and clarity.

---

## 13. Project Milestones
1. RV64I + basic CSRs + riscv-tests
2. Traps, delegation, interrupts
3. Sv39 MMU
4. Atomics (A extension)
5. UART + timer (CLINT)
6. Linux boot
7. Cache/TLB + timing

---

## 14. Success Criteria

The project is considered successful if:
- Linux boots and prints console output
- riscv-tests pass consistently
- Cache/TLB effects are observable via benchmarks
- The codebase remains understandable and modular

---

## 15. Future Work (Optional)

- JIT backend
- Branch prediction
- Out-of-order execution (OoO-lite)
- Instruction cache
- Multi-core support

These are explicitly outside the committed scope.

# SPEC FROZEN: DEC 24TH