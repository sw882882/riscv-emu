use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    /// Path to a RISC-V ELF to load (statically linked is easiest at first)
    #[arg(long)]
    elf: String,

    /// RAM size in MiB
    #[arg(long, default_value_t = 256)]
    ram_mib: usize,

    /// Stop after N instructions (0 = run forever)
    #[arg(long, default_value_t = 0)]
    max_insns: u64,

    /// Enable instruction trace
    #[arg(long, default_value_t = false)]
    trace: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let ram_bytes = args.ram_mib * 1024 * 1024;
    let mut machine = riscv_emu::cpu::Machine::new(ram_bytes);

    let entry = riscv_emu::elf::load_elf_into_memory(&args.elf, &mut machine.mem)?;
    machine.cpu.pc = entry;
    // sanity check
    println!("Loaded ELF entry point at 0x{:016x}", entry);

    // Minimal convention: x0 hardwired, others start 0.
    // You can also set up a stack pointer later if you want for your own test programs.
    let mut executed: u64 = 0;
    loop {
        if args.max_insns != 0 && executed >= args.max_insns {
            break;
        }
        if args.trace {
            riscv_emu::debug::trace(&machine.cpu, executed);
        }

        machine.step()?; // fetch-decode-execute
        executed += 1;
    }

    Ok(())
}
