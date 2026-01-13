use crate::mem::{MemError, Memory};
use goblin::elf::{
    Elf,
    header::{self, ELFCLASS64, ELFDATA2LSB, EM_RISCV, ET_DYN, ET_EXEC},
};
use std::fs;

pub fn load_elf_into_memory(
    path: &str,
    mem: &mut Memory,
) -> Result<u64, Box<dyn std::error::Error>> {
    let bytes = fs::read(path)?;
    let elf = Elf::parse(&bytes)?;

    // Basic sanity checks so we fail fast on bad inputs
    if elf.header.e_ident[header::EI_CLASS] != ELFCLASS64 {
        return Err("expected 64-bit ELF".into());
    }
    if elf.header.e_ident[header::EI_DATA] != ELFDATA2LSB {
        return Err("expected little-endian ELF".into());
    }
    if elf.header.e_machine != EM_RISCV {
        return Err("expected RISC-V ELF".into());
    }
    if elf.header.e_type != ET_EXEC && elf.header.e_type != ET_DYN {
        return Err("unsupported ELF type (want ET_EXEC or ET_DYN)".into());
    }

    let ram_end = mem.end_addr();

    // Load PT_LOAD program headers
    for ph in &elf.program_headers {
        if ph.p_type != goblin::elf::program_header::PT_LOAD {
            continue;
        }
        let file_off = ph.p_offset as usize;
        let file_sz = ph.p_filesz as usize;
        let vaddr = ph.p_vaddr as u64;

        let end = file_off
            .checked_add(file_sz)
            .ok_or("program header file range overflow")?;
        if end > bytes.len() {
            return Err(
                format!("segment outside file: off=0x{file_off:x} size=0x{file_sz:x}").into(),
            );
        }
        if ph.p_memsz < ph.p_filesz {
            return Err(
                format!("p_memsz smaller than p_filesz for segment at off=0x{file_off:x}").into(),
            );
        }

        let seg_end = vaddr
            .checked_add(ph.p_memsz)
            .ok_or("segment virtual range overflow")?;
        if vaddr < mem.base || seg_end > ram_end {
            return Err(format!(
                "segment outside RAM: [0x{vaddr:x},0x{seg_end:x}) not within [0x{:x},0x{:x})",
                mem.base, ram_end
            )
            .into());
        }

        let seg = &bytes[file_off..file_off + file_sz];
        mem.write_bytes(vaddr, seg)
            .map_err(|e: MemError| format!("mem write failed: {e}"))?;

        // Zero-fill bss (p_memsz may be larger than p_filesz)
        let mem_sz = ph.p_memsz as usize;
        if mem_sz > file_sz {
            let zeros = vec![0u8; mem_sz - file_sz];
            mem.write_bytes(vaddr + file_sz as u64, &zeros)
                .map_err(|e: MemError| format!("bss write failed: {e}"))?;
        }
    }

    Ok(elf.entry)
}

/// Find the address of the "tohost" symbol in an ELF file.
/// This is used by RISC-V tests to signal completion.
pub fn find_tohost_symbol(path: &str) -> Result<Option<u64>, Box<dyn std::error::Error>> {
    let bytes = fs::read(path)?;
    let elf = Elf::parse(&bytes)?;

    for sym in elf.syms.iter() {
        if let Some(name) = elf.strtab.get_at(sym.st_name) {
            if name == "tohost" {
                return Ok(Some(sym.st_value));
            }
        }
    }

    Ok(None)
}
