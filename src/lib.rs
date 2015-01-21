#![allow(unstable)]
#![allow(missing_copy_implementations)]
#![feature(asm)]
extern crate libc;
use libc::*;
pub use u_context::*;
pub type InitFn = extern "C" fn();

#[link = "glibc"]
extern "C" {
    // because get/swap context depends rbp/rsp strictly, we cannot provide a binding. Any indirection will break the code(during optimization).
    pub fn getcontext(ucp: *mut UContext) -> c_int;
    pub fn swapcontext(oucp: *mut UContext, ucp: *const UContext) -> c_int;
    
    fn setcontext(ucp: *const UContext);
    fn makecontext(ucp: *const UContext, f: InitFn, args: c_int, argv: *const c_void);
}

extern "C" fn bridge_to_c(func: *mut c_void) {
    use std::mem::transmute;
    use std::thunk::Thunk;
    
    let f: Box<Thunk> = unsafe {transmute(func)};
    f.invoke(());
}

#[cfg(all(target_os="linux", target_arch="x86_64"))]
mod u_context {
    use libc::*;
    use std::mem::*;
    use std::default::Default;
    use std::thunk::Thunk;

    const SIGSET_NWORDS: usize = (1024 / 64);

    #[repr(C)]
    struct Stack {
        pub ss_sp: *const (),
        pub ss_flags: c_int,
        pub ss_size: size_t,
    }
    
    type GReg = c_longlong;
    const NGREG: usize = 23us;
    type GRegSet = [GReg;NGREG];
    // rsp is #15 and rip is #16
    
    #[repr(C)]
    struct FpxReg {
        significand: [c_ushort;4],
        exponent: c_ushort,
        padding: [c_ushort;3],
    }

    #[repr(C)]
    struct XmmReg {
        element: [u32;4],
    }

    #[repr(C)]
    struct FpState {
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

    type FpRegSet = *const FpState;

    #[repr(C)]
    struct MContext {
        g_reg_set: GRegSet,
        fp_regs: FpRegSet,
        reserved1: [c_ulonglong;8],
    }

    #[repr(C)]
    pub struct UContext {
        flags: c_ulong,
        link: *const UContext,
        stack: Stack,
        mcontext: MContext,
        sigmask: [c_ulong;SIGSET_NWORDS],
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
        pub fn new() -> UContext {
            Default::default()
        }

        pub fn make_context<F>(&mut self, f: F) -> () where F: Send + FnOnce() {
            let thk = Thunk::new(f);
            unsafe {
                ::makecontext(self, transmute(::bridge_to_c), 1, transmute(Box::new(thk)));
            }
        }
        
        pub fn set_context(&self) {
            unsafe { ::setcontext(self) };
        }
        
        pub fn set_stack(&mut self, start: *const u8, end: *const u8) {
            let (stack_ptr, stack_len) = (start, end as u64 - start as u64);
            self.stack.ss_sp = stack_ptr as *const ();
            self.stack.ss_size = stack_len * 8;
            self.stack.ss_flags = 0;
        }
        
        pub fn set_link(&mut self, link: &UContext) {
            self.link = link;
        }
        
    }
}

// not complete
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
    struct FpReg {
        significand: [c_ushort;4],
        exponent: c_ushort,
    }
    
    #[repr(C)]
    struct FpState {
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
    struct MContext {
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


