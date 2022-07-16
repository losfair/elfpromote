use std::fs::File;
use std::io::Write;

use anyhow::Context;
use anyhow::Result;
use atomicwrites::AtomicFile;
use atomicwrites::OverwriteBehavior::AllowOverwrite;
use byteorder::ByteOrder;
use byteorder::LittleEndian;
use byteorder::ReadBytesExt;
use goblin::elf64::dynamic::DT_NEEDED;
use goblin::{
    elf::Elf,
    elf64::{
        dynamic::{Dyn, DT_NULL},
        program_header::PT_DYNAMIC,
    },
};
use memmap2::MmapOptions;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "elfpromote", about = "Promote ELF shared library load order.")]
struct Opt {
    /// Input file
    input: String,

    /// Output file
    #[structopt(short, long)]
    output: String,

    /// Name of the shared library to promote
    #[structopt(short, long)]
    lib: String,
}

fn main() -> Result<()> {
    pretty_env_logger::init_timed();
    let opt = Opt::from_args();
    let input_file = File::open(&opt.input).with_context(|| "Failed to open input file")?;
    let elf_bytes = unsafe { MmapOptions::new().map_copy_read_only(&input_file) }
        .with_context(|| "Failed to map input file for reading")?;
    let mut output = unsafe { MmapOptions::new().map_copy(&input_file) }
        .with_context(|| "Failed to map input file for modifying")?;
    let input_metadata = input_file
        .metadata()
        .with_context(|| "Failed to get input file metadata")?;
    let elf = Elf::parse(&elf_bytes).with_context(|| "Failed to parse ELF")?;
    let dynamic_phdr = elf
        .program_headers
        .iter()
        .find(|x| x.p_type == PT_DYNAMIC)
        .ok_or_else(|| anyhow::anyhow!("No PT_DYNAMIC program header found"))?;
    let offset = dynamic_phdr.p_offset as usize;
    let filesz = dynamic_phdr.p_filesz as usize;
    let mut region = output
        .get_mut(offset..offset.saturating_add(filesz))
        .ok_or_else(|| anyhow::anyhow!("Failed to get dynamic region"))?;
    let mut pairs: Vec<Dyn> = vec![];
    {
        let mut region = std::io::Cursor::new(&*region);
        loop {
            let d_tag = region
                .read_u64::<LittleEndian>()
                .with_context(|| "Failed to read d_tag")?;
            let d_val = region
                .read_u64::<LittleEndian>()
                .with_context(|| "Failed to read d_val")?;
            if d_tag == DT_NULL {
                break;
            }
            pairs.push(Dyn { d_tag, d_val });
        }
    }
    let mut entry: Option<Dyn> = None;
    for (i, d) in pairs.iter().enumerate() {
        if d.d_tag == DT_NEEDED {
            if let Some(name) = elf.dynstrtab.get_at(d.d_val as usize) {
                log::debug!("found library: {}", name);
                if name == opt.lib {
                    entry = Some(pairs.remove(i));
                    break;
                }
            }
        }
    }
    let entry = entry.ok_or_else(|| anyhow::anyhow!("Failed to find the requested library"))?;
    pairs.insert(0, entry);
    for d in &pairs {
        LittleEndian::write_u64(&mut region[..8], d.d_tag);
        LittleEndian::write_u64(&mut region[8..16], d.d_val);
        region = &mut region[16..];
    }
    let output_file = AtomicFile::new(&opt.output, AllowOverwrite);
    output_file
        .write::<_, std::io::Error, _>(|file| {
            file.write_all(&output)?;
            file.set_permissions(input_metadata.permissions())?;
            Ok(())
        })
        .with_context(|| "Failed to write output file")?;
    Ok(())
}
