#![allow(unstable)]
#![allow(missing_copy_implementations)]
#![feature(asm)]
extern crate libc;
use libc::*;
use std::mem::transmute;

pub use u_context::*;

#[derive(Show)]
enum UErr {
    Fail,
}

#[link = "glibc"]
extern "C" {
    pub fn getcontext(ucp: *mut UContext) -> c_int;
    pub fn setcontext(ucp: *const UContext) -> c_int;
    pub fn swapcontext(oucp: *mut UContext, ucp: *const UContext) -> c_int;
    // dunno how to bind var-arg function
    // TODO
    pub fn makecontext(ucp: *const UContext, f: fn(), argc: c_int);
}

#[cfg(all(target_os="linux", target_arch="x86_64"))]
mod u_context {
    use libc::*;
    use std::mem::transmute;
    use std::mem::swap;
    use std::default::Default;
    
    pub const SIGSET_NWORDS: usize = (1024 / 64);

    pub struct Stack {
        pub ss_sp: *const (),
        pub ss_flags: c_int,
        pub ss_size: size_t,
    }
    
    type GReg = c_longlong;
    const NGREG: usize = 23us;
    type GRegSet = [GReg;NGREG];
    // rsp is #15 and rip is #16
    
    #[repr(C)]
    pub struct FpxReg {
        significand: [c_ushort;4],
        exponent: c_ushort,
        padding: [c_ushort;3],
    }

    #[repr(C)]
    pub struct XmmReg {
        element: [u32;4],
    }

    #[repr(C)]
    pub struct FpState {
        cwd: u16,
        swd: u16,
        ftw: u16,
        fop: u16,
        rip: u64,
        rdp: u64,
        mxcsr: u32,
        mxcr_mask: u32,
        st: [FpxReg;8],
        xmm: [XmmReg;16],
        padding: [u32;24],
    }

    pub type FpRegSet = *const FpState;

    #[repr(C)]
    pub struct MContext {
        g_reg_set: GRegSet,
        fp_regs: FpRegSet,
        reserved1: [c_ulonglong;8],
    }

    #[repr(C)]
    pub struct UContext {
        pub flags: c_ulong,
        pub link: *const UContext,
        pub stack: Stack,
        mcontext: MContext,
        pub sigmask: [c_ulong;SIGSET_NWORDS],
        fp_regs_mem: FpState,
    }

    impl Default for UContext {
        fn default() -> Self {
            let a = [0u8;936];
            let ctx: UContext = unsafe { transmute(a) };
            ctx
        }
    }
    
    impl UContext {
        // I dunno how to bind to c's var-arg function,so it is a
        // TODO
        pub fn make_context(f: fn()) -> UContext {
            let u = Default::default();
            unsafe {
                ::makecontext(&u, f, 0);
            }
            u
        }
        pub fn get_context(&mut self) {
            unsafe { ::getcontext(self) };

            // fix the offset, because of the indirection of get_context
            self.mcontext.g_reg_set[15] += 0x50i64; //rsp + 0x50 (0x50)
            
            // rbp@0x50(rsp)
            let mut rbp;
            unsafe {asm!(r"mov 0x50(%rsp), $0":"=r"(rbp));};
            self.mcontext.g_reg_set[10] = rbp; //rbp

            // rip@0x58(rsp)
            let mut rip;
            unsafe {asm!(r"mov 0x58(%rsp), $0":"=r"(rip));};
            self.mcontext.g_reg_set[16] = rip;//rip
        }
        pub fn set_context(&self) {
            unsafe { ::setcontext(self) };
        }
    }
}

#[cfg(all(target_os="linux", not(target_arch="x86_64")))]
mod u_context {
    use libc::*;
    
    const SIGSET_NWORDS: usize = (1024 / 32);
    #[repr(C)]
    struct SigSet {
        val: [c_ulong;SIGSET_NWORDS],
    }

    #[repr(C)]
    struct Stack {
        ss_sp: *const (),
        ss_flags: c_int,
        ss_size: size_t,
    }

    type GReg = c_int;
    const NGREG: usize = 19us;
    type GRegSet = [GReg;NGREG];

    #[repr(C)]
    pub struct FpReg {
        significand: [c_ushort;4],
        exponent: c_ushort,
    }
    
    #[repr(C)]
    pub struct FpState {
        cw: c_long,
        sw: c_long,
        tag: c_long,
        ipoff: c_long,
        cssel: c_long,
        dataoff: c_long,
        datasel: c_long,
        st: [FpReg;8],
        status: c_ulong,
    }
    
    type FpRegSet = *const FpState;

    
    #[repr(C)]
    pub struct MContext {
        g_regs: GRegSet,
        fpregs: FpRegSet,
        oldmask: c_ulong,
        cr2: c_ulong,
    }

    #[repr(C)]
    pub struct UContext {
        uc_flags: c_ulong,
        uc_link: *const UContext,
        uc_stack: Stack,
        uc_mcontext: MContext,
        uc_sigmask: SigSet,
        fpregs_mem: FpState,
    }
}


