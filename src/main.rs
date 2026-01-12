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
    machine.max_insns = args.max_insns;

    let entry = riscv_emu::elf::load_elf_into_memory(&args.elf, &mut machine.mem)?;
    machine.cpu.pc = entry;
    // sanity check
    println!("Loaded ELF entry point at 0x{:016x}", entry);

    // Minimal convention: x0 hardwired, others start 0.
    // You can also set up a stack pointer later if you want for your own test programs.
    loop {
        if args.trace {
            riscv_emu::debug::trace(&machine.cpu, machine.executed);
        }

        // fetch-decode-execute
        // handle halting conditions
        match machine.step() {
            Err(riscv_emu::cpu::CpuStepResult::Halt(reason)) => {
                println!("CPU halted: {}", reason);
                break;
            }
            Err(e) => {
                eprintln!("CPU error: {}", e);
                break;
            }
            Ok(()) => {}
        }
    }

    Ok(())
}
