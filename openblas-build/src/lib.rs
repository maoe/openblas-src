//! openblas-build
//! ---------------
//!
//! Helper crate for openblas-src/build.rs

use anyhow::{bail, Result};
use std::{
    fs,
    os::unix::io::*,
    path::*,
    process::{Command, Stdio},
};

pub fn openblas_source_dir() -> PathBuf {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("source");
    if !path.join("Makefile").exists() {
        panic!("OpenBLAS repository has not been cloned. Run `git submodule update --init`");
    }
    path
}

#[derive(Debug, Clone, Copy)]
pub enum Interface {
    LP64,
    ILP64,
}

impl Default for Interface {
    fn default() -> Self {
        Interface::LP64
    }
}

/// Target CPU list
/// from https://github.com/xianyi/OpenBLAS/blob/v0.3.10/TargetList.txt
#[derive(Debug, Clone, Copy)]
#[allow(non_camel_case_types)] // to use original identifiers
pub enum Target {
    // X86/X86_64 Intel
    P2,
    KATMAI,
    COPPERMINE,
    NORTHWOOD,
    PRESCOTT,
    BANIAS,
    YONAH,
    CORE2,
    PENRYN,
    DUNNINGTON,
    NEHALEM,
    SANDYBRIDGE,
    HASWELL,
    SKYLAKEX,
    ATOM,

    // X86/X86_64 AMD
    ATHLON,
    OPTERON,
    OPTERON_SSE3,
    BARCELONA,
    SHANGHAI,
    ISTANBUL,
    BOBCAT,
    BULLDOZER,
    PILEDRIVER,
    STEAMROLLER,
    EXCAVATOR,
    ZEN,

    // X86/X86_64 generic
    SSE_GENERIC,
    VIAC3,
    NANO,

    // Power
    POWER4,
    POWER5,
    POWER6,
    POWER7,
    POWER8,
    POWER9,
    PPCG4,
    PPC970,
    PPC970MP,
    PPC440,
    PPC440FP2,
    CELL,

    // MIPS
    P5600,
    MIPS1004K,
    MIPS24K,

    // MIPS64
    SICORTEX,
    LOONGSON3A,
    LOONGSON3B,
    I6400,
    P6600,
    I6500,

    // IA64
    ITANIUM2,

    // Sparc
    SPARC,
    SPARCV7,

    // ARM
    CORTEXA15,
    CORTEXA9,
    ARMV7,
    ARMV6,
    ARMV5,

    // ARM64
    ARMV8,
    CORTEXA53,
    CORTEXA57,
    CORTEXA72,
    CORTEXA73,
    NEOVERSEN1,
    EMAG8180,
    FALKOR,
    THUNDERX,
    THUNDERX2T99,
    TSV110,

    // System Z
    ZARCH_GENERIC,
    Z13,
    Z14,
}

#[derive(Debug, Clone, Default)] // default of bool is false
pub struct BuildOption {
    pub no_static: bool,
    pub no_shared: bool,
    pub no_cblas: bool,
    pub no_lapack: bool,
    pub no_lapacke: bool,
    pub no_fortran: bool,
    pub use_thread: bool,
    pub use_openmp: bool,
    pub dynamic_arch: bool,
    pub interface: Interface,
    pub target: Option<Target>,
}

#[derive(Debug, Clone)]
pub struct Detail {}

impl BuildOption {
    fn make_args(&self) -> Vec<String> {
        let mut args = Vec::new();
        if self.no_static {
            args.push("NO_STATIC=1".into())
        }
        if self.no_shared {
            args.push("NO_SHARED=1".into())
        }
        if self.no_cblas {
            args.push("NO_CBLAS=1".into())
        }
        if self.no_lapack {
            args.push("NO_LAPACK=1".into())
        }
        if self.no_lapacke {
            args.push("NO_LAPACKE=1".into())
        }
        if self.no_fortran {
            args.push("NOFORTRAN=1".into())
        }
        if self.use_thread {
            args.push("USE_THREAD=1".into())
        }
        if self.use_openmp {
            args.push("USE_OPENMP=1".into())
        }
        if matches!(self.interface, Interface::ILP64) {
            args.push("INTERFACE64=1".into())
        }
        if let Some(target) = self.target.as_ref() {
            args.push(format!("TARGET={:?}", target))
        }
        args
    }

    /// Shared or static library will be created
    /// at `out_dir/libopenblas.so` or `out_dir/libopenblas.a`
    ///
    /// - If `out_dir` already exists, it will be removed.
    pub fn build<P: AsRef<Path>>(self, out_dir: P) -> Result<Detail> {
        let out_dir = out_dir.as_ref();
        if out_dir.exists() {
            fs::remove_dir_all(&out_dir)?;
        }
        fs_extra::dir::copy(
            openblas_source_dir(),
            out_dir,
            &fs_extra::dir::CopyOptions {
                overwrite: true,
                skip_exist: false,
                buffer_size: 1_000_000,
                copy_inside: true,
                content_only: false,
                depth: 0,
            },
        )?;

        let out = fs::File::create(out_dir.join("out.log")).expect("Cannot create log file");
        let err = fs::File::create(out_dir.join("err.log")).expect("Cannot create log file");

        Command::new("make")
            .current_dir(out_dir)
            .stdout(unsafe { Stdio::from_raw_fd(out.into_raw_fd()) })
            .stderr(unsafe { Stdio::from_raw_fd(err.into_raw_fd()) })
            .args(&self.make_args())
            .check_call()?;

        todo!()
    }
}

trait CheckCall {
    fn check_call(&mut self) -> Result<()>;
}

impl CheckCall for Command {
    fn check_call(&mut self) -> Result<()> {
        match self.status() {
            Ok(status) => {
                if !status.success() {
                    bail!(
                        "Subprocess returns with non-zero status: `{:?}` ({})",
                        self,
                        status
                    );
                }
                Ok(())
            }
            Err(error) => {
                bail!("Subprocess execution failed: `{:?}` ({})", self, error);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_default() -> Result<()> {
        let opt = BuildOption::default();
        let _detail = opt.build("test_build/build_default")?;
        Ok(())
    }
}
